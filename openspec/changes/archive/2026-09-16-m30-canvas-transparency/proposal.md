## Why

The document canvas does not match Photoshop's transparency display:

- A pixel with `alpha < 255` reveals the plain canvas colour. Photoshop instead
  shows the transparency checkerboard behind the document, so the user can see
  which pixels are transparent and which are opaque.
- Content that extends past the document bounds is drawn onto the canvas. The
  Move-tool live preview in particular paints the dragged layer beyond the
  document rect, so the layer appears to spill over the canvas colour instead of
  being cropped at the document edge.

M30 fixes both in the canvas paint path: a transparency checkerboard behind the
document, and document-rect clipping of every piece of canvas content.

## What Changes

- **Transparency checkerboard.** `ImageView::paintEvent` draws a two-tone
  checkerboard (`#FFFFFF` / `#CCCCCC`, Photoshop "Light") behind the document,
  clipped to the document rect, so a pixel with `alpha < 255` reveals it. A
  fully transparent document shows the checkerboard; opaque pixels cover it; the
  checkerboard never appears outside the document rect.
- **Screen-space, document-anchored.** The cell size is a constant 8 screen
  pixels, independent of zoom, and the pattern is anchored to the document origin
  via a brush origin, so panning does not move it relative to the document. A
  cached 2×2-cell tile is filled as a `QBrush`, so the cost is O(1) in tile
  draws regardless of document size.
- **Clip all content to the document rect.** The composited image, the Move-tool
  preview layer (cached base plus moved layer at the live delta), and the
  overlay polygon are clipped to the document rect. A layer dragged outside the
  canvas is cropped; the pixels just outside the rect show the canvas colour.
- **Self-test.** `--self-test` renders a 64×64 transparent document and checks
  both checker tones inside the document rect with the canvas colour outside,
  then checks a red Move-preview layer at `(-32, -32)` fills the covered
  quadrant while the pixels outside the document rect stay the canvas colour.
  Failure exits 74.

## Capabilities

### New Capabilities

None. M30 extends an existing capability.

### Modified Capabilities

- `application-shell`: the document canvas SHALL draw a two-tone transparency
  checkerboard behind the document image, clipped to the document rect; SHALL
  clip the composited document and the live move preview to the document rect;
  and the checkerboard SHALL be constant in screen space and anchored to the
  document origin.

## Impact

- `crates/pictura-app/cpp/image_view.{h,cpp}` — `transparencyCellSize()`,
  `transparencyColorA()`, `transparencyColorB()`, the cached tile, the
  checkerboard fill with a document-origin brush origin, and the
  `setClipRect` around the document content.
- `crates/pictura-app/cpp/main.cpp` — the M30 self-test block (checkerboard
  reveal, document-rect clipping) with exit code 74.
- No new dependency; the CPU `QImage` canvas is unchanged otherwise. The
  `Transparency & Gamut` preferences pane, the `View > Show > Transparency Grid`
  toggle, gamut warning, and the GPU/RHI canvas are deferred, not part of M30.
