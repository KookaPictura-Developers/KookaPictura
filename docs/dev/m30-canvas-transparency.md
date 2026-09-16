# M30 — Canvas transparency

Goal: the canvas shows the document the way Photoshop does. Two gaps today:
a pixel with `alpha < 255` reveals the plain canvas colour instead of a
transparency checkerboard, and content dragged outside the document — notably
the Move-tool live preview — is painted onto the canvas beyond the document
bounds.

Implemented in `crates/pictura-app/cpp/image_view.{h,cpp}`. `paintEvent` paints
in this order:

1. the plain canvas colour over the whole widget;
2. a screen-space checkerboard clipped to the document rect;
3. the composited image (or the Move preview: cached base plus moved layer)
   under the pan/zoom transform, clipped to the document rect;
4. the overlay polygon.

## Scope

- **Screen-space, document-anchored checkerboard.** A cached 2×2-cell tile
  (`transparencyTile()`) is filled with `QBrush` at
  `setBrushOrigin(docRect.topLeft())`, clipped to the document rect ∩ viewport.
  Cell size is a constant 8 screen pixels (`transparencyCellSize()`), so zoom
  does not scale the pattern and panning does not move it relative to the
  document. The fill is O(1) in tile draws regardless of document size.
- **Two light tones.** `transparencyColorA()` `#FFFFFF` and
  `transparencyColorB()` `#CCCCCC` — the Photoshop "Light" grid. A fully
  transparent document shows the checkerboard inside the document rect and the
  canvas colour outside it; opaque pixels cover the checkerboard.
- **All content clipped to the document rect.** `paintEvent` calls
  `setClipRect(QRectF(0, 0, image_.width(), image_.height()))` after the
  transform, so the composited image, the Move-tool preview layer, and the
  overlay are cropped. A layer dragged past the canvas edge shows the canvas
  colour outside, not the layer.
- **Self-test exit code 74.** `--self-test` opens a 64×64 transparent document,
  renders the canvas, and checks both checker tones appear inside the document
  rect with the canvas colour outside; it then starts a Move preview with a red
  layer at `(-32, -32)` and checks the covered quadrant is red while the pixels
  just outside the document rect stay the canvas colour. Either failure exits 74.

## Out of scope (later milestones)

- The `Transparency & Gamut` preferences pane (grid size None/Small/Medium/Large,
  colours Light/Medium/Dark/Red/Custom) and the
  `View > Show > Transparency Grid` toggle. M30 hard-codes the CS6 default
  (Medium size, Light colours) as an 8 px two-tone tile; the preferences plumbing
  is deferred.
- Gamut warning.
- The GPU/RHI-backed canvas (zero-copy present). The canvas is still the CPU
  `QImage` path; the checkerboard is a Qt brush fill.

## Process

Waves: (1) the `ImageView` checkerboard and clipping; (2) the self-test check;
(3) close-out.

## Verification

- `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` reports
  `m30_canvas checker=1 clipped=1` and exits 0 (exit 74 on failure)
- `cargo test --workspace`; `cargo fmt/clippy`
- `openspec validate --all --strict`; `guard.sh`
