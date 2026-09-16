## Context

The central canvas is `ImageView` (`crates/pictura-app/cpp/image_view.{h,cpp}`),
a plain `QWidget` that paints a `QImage` under a pan/zoom transform. `paintEvent`
fills the whole widget with `canvasColor_` and, when a document is open, draws the
image (or the Move-tool preview: a cached base plus the moved layer) after
`translate(offset_)` and `scale(zoom_)`.

Two display gaps:

- The canvas colour shows through any pixel with `alpha < 255`. There is no
  transparency checkerboard, so transparent regions read as canvas colour rather
  than as transparent.
- The transformed image and the Move-tool preview are drawn without a clip, so
  content dragged past the document edge is painted onto the canvas beyond the
  document rect.

Photoshop CS6 draws a two-tone checkerboard behind the document (`Transparency &
Gamut` preferences: grid size None/Small/Medium/Large, colours
Light/Medium/Dark/Red/Custom; `docs/02-ui-ux/preferences.md`), with the default
Medium size and Light colours. The grid is screen-space — it does not scale with
zoom — and anchored to the document; everything in the document window is cropped
at the document edges.

Constraints: the canvas is still the CPU `QImage` path (the GPU/RHI zero-copy
present is a later milestone); no new dependency; `docs/` is the long-form
contract.

## Goals / Non-Goals

**Goals:**

- Draw a two-tone checkerboard behind the document, clipped to the document rect,
  so pixels with `alpha < 255` reveal it; a fully transparent document shows the
  checkerboard and opaque pixels cover it.
- Keep the checkerboard constant in screen space (8 px cells, independent of
  zoom) and anchored to the document origin, so panning does not move the pattern
  relative to the document.
- Clip the composited image, the live Move-tool preview, and the overlay to the
  document rect, so content outside the canvas is cropped.
- Keep the fill O(1) in the number of tile draws by caching a 2×2-cell tile and
  using a brush origin.
- Ship a `--self-test` check (exit 74 on failure) that verifies both the
  checkerboard reveal and the document-rect clipping.

**Non-Goals:**

- The `Transparency & Gamut` preferences pane (grid size None/Small/Medium/Large
  and colours Light/Medium/Dark/Red/Custom). M30 hard-codes the CS6 default
  (Medium, Light) as an 8 px two-tone tile.
- The `View > Show > Transparency Grid` toggle.
- Gamut warning.
- The GPU/RHI-backed canvas (zero-copy present).

## Decisions

### 1. Screen-space, document-anchored checkerboard (frozen)

`paintEvent` computes the document rect
`QRectF docRect(offset_, QSizeF(image_.width() * zoom_, image_.height() * zoom_))`
and clips it to the widget:

```
const QRectF checkerRect = docRect.intersected(QRectF(rect()));
painter.setBrushOrigin(docRect.topLeft().toPoint());
painter.fillRect(checkerRect, QBrush(transparencyTile()));
```

`transparencyTile()` is a lazily-built `QPixmap` of `2 * transparencyCellSize()`
square, filled `transparencyColorA()` with the opposite two cells
`transparencyColorB()`.

- *Why:* filling a brush tile is O(1) in tile draws, so the checkerboard costs
  the same at 64² and 4000²; the brush origin ties the pattern to the document
  top-left, so it moves with the image under pan; the fill rect is the document
  rect, so the pattern never lands on the canvas.
- *Why constant cells:* a screen-space grid is what Photoshop shows — cells stay
  legible at any zoom and the pattern does not become a moiré at high zoom.
- *Alternatives:* drawing every cell as a `fillRect` pair (rejected — O(cells)
  and duplicates the tile); scaling the tile with zoom (rejected — wrong
  appearance and cell size would change with zoom); clipping with
  `setClipRect` before the transform (equivalent, but the fill-rect +
  intersection form also avoids painting the clipped-away region at all).

### 2. Two light tones (frozen)

`transparencyCellSize()` returns 8; `transparencyColorA()` returns `#FFFFFF`
and `transparencyColorB()` returns `#CCCCCC` — the Photoshop "Light" grid. These
are `static` on `ImageView` so the self-test and any future preferences wiring can
read the single source of truth.

- *Why:* matches the CS6 default and gives a visible two-tone grid on both dark
  and light canvases.
- *Ceiling:* the grid size and colour set are fixed at the CS6 default; a future
  preferences pane replaces the constants with prefs-backed values (ponytail:
  hard-coded CS6 Light/Medium, wire to `TransparencyPrefs` when the pane lands).

### 3. Clip all content to the document rect (frozen)

After `translate(offset_)` and `scale(zoom_, zoom_)`, `paintEvent` calls
`painter.setClipRect(QRectF(0, 0, image_.width(), image_.height()))` before
drawing. The clip applies to the composited image, the Move-preview base and
moved layer, and the overlay polygon. It is set before the branch, so the
preview and the normal path share it.

- *Why:* the document rect is the only visible content area; a dragged layer is
  cropped at the document edge, matching Photoshop and the `canvas-operations`
  behaviour of resizing the canvas.
- *Alternatives:* clipping only the preview layer (rejected — the
  composited image and overlay have the same contract).

### 4. Self-test check (frozen)

`--self-test` opens a 64×64 transparent document, renders the canvas to an
`ARGB32` image, and probes a grid of points inside `docRect`:

- both checker tones must appear inside the document rect,
- the canvas colour must NOT appear inside the document rect (fully transparent
  document), and
- the pixel 4 px outside the document rect must be the canvas colour.

Then it starts a Move preview with a red 64×64 layer at `(-32, -32)`, renders,
and checks the covered top-left quadrant near the document centre is red while
the pixels outside the document rect stay the canvas colour. Any mismatch prints
`FAIL: M30 transparency/clipping wrong` and exits 74; success prints
`m30_canvas checker=1 clipped=1`.

- *Why:* rendering the widget and sampling it is the existing self-test idiom
  (M23 chrome, M28 heavy filters); it exercises the real `paintEvent` rather than
  a reimplementation.

### 5. Process (frozen)

Waves: (1) the `ImageView` checkerboard and clipping; (2) the self-test check;
(3) close-out.

## Risks / Trade-offs

- **Brush origin drifts under fractional offsets** → `setBrushOrigin` takes the
  integer `docRect.topLeft().toPoint()`, so the pattern can shift by <1 px as the
  pan crosses a pixel boundary. The grid stays aligned to the document; the
  sub-pixel jitter is invisible at an 8 px cell and matches Qt's integer brush
  origin behaviour.
- **Checkerboard at extreme zoom** → cells are constant in screen space, so the
  pattern never scales into a moiré or collapses to a single tone; the fill rect
  remains bounded by the viewport.
- **Clipping hides intended content** → the overlay polygon is also clipped;
  selection overlays are document-space by construction, so clamping them to the
  document rect is correct. A future out-of-document overlay (e.g. a crop overlay
  extending past the canvas) would need its own clip, noted here.
- **Hard-coded grid** → M30 does not honor the `Transparency & Gamut`
  preferences; a user who wants None/Small/Large or a different colour set must
  wait for that pane. The constants are the single source of truth for the wiring.

## Migration Plan

Additive. `paintEvent` gains the checkerboard fill and the `setClipRect`; three
`static` accessors and a cached tile are added to `ImageView`. No public API
change, no dependency. Rollback: remove the fill and the clip, and the previous
canvas-colour behaviour returns. The deferred preferences pane can later read
`TransparencyPrefs` in place of the constants without touching the geometry.

## Open Questions

- Whether the preferences pane should store the grid as a screen-pixel size or a
  Photoshop size enum (None/Small/Medium/Large) — decided when the pane lands.
- Whether the checkerboard should also back the Move-tool preview when the
  preview is the only content drawn — M30 keeps the checkerboard under both
  paths; no evidence yet that a preview-only variant is wanted.
