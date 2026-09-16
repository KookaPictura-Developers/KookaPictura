## MODIFIED Requirements

### Requirement: Dirty-region canvas refresh

The canvas SHALL treat the document's planar `composite` buffer as the
authoritative full-document canvas and SHALL NOT maintain a full-resolution
`QImage` as a per-pixel region-patch target. When a mutation reports a dirty
rectangle, the view SHALL composite only that rectangle through the active
backend, write the result into the document's `composite` at the rectangle's
origin with a plane `copy_from_slice` (no per-pixel FFI), convert only the
rectangle to a `QImage`, mark the display dirty, and emit a
`regionBlitted(region, x, y)` signal. It MUST NOT emit `changed`, and it MUST NOT
require rebuilding the full-resolution image. A Move commit SHALL invalidate
`old_layer_rect ∪ new_layer_rect`; a paint dab SHALL invalidate the dab's bounding
box. A canvas updated only through such region refreshes SHALL be byte-identical
to a full recomposite after any sequence of those mutations. There SHALL be no
area budget: a dirty rectangle of any size SHALL take the region-blit path and
MUST NOT fall back to a full-document recomposite on account of its size. A
mutation that does not report a dirty rectangle SHALL keep the full recomposite
and SHALL emit `changed`.

#### Scenario: After several Move commits the region-refreshed canvas equals a full recomposite

- **WHEN** several Move commits are applied to a document, with GPU compute
  enabled or falling back to the CPU oracle
- **THEN** the document composite and the rebuilt full image are byte-identical to
  a full recomposite of the final document

#### Scenario: A region update does not disturb pixels outside the region

- **WHEN** a dirty rectangle is refreshed through the active backend
- **THEN** every pixel outside the rectangle keeps its previous value and every
  pixel inside the rectangle equals the corresponding pixel of a full recomposite

#### Scenario: A large dirty rectangle does not fall back to a full composite

- **WHEN** a 1024×1024 paint dab is refreshed on a 4000×4000 document
- **THEN** only the rectangle is composited and blitted, the view does not run a
  full-document composite because of the rectangle's size, and the result equals a
  full recomposite

#### Scenario: A region refresh does not emit `changed`

- **WHEN** a region refresh runs on a document
- **THEN** it emits `regionBlitted(region, x, y)` and the panel refresh is not
  driven more frequently than by a full recomposite

### Requirement: Region-composited move-preview base

Starting a Move-tool preview (`PictureView::begin_move_preview`) MUST NOT
composite the whole document. It SHALL build the preview base by region-
compositing only the moved layer's clamped document rectangle with that layer
hidden (`visible = false`) against the current cached canvas, and MUST NOT modify
the document's stored `composite` or the persistent cached canvas. The resulting
base image MUST be byte-identical to a full recomposite of the document with that
layer hidden. The base SHALL be derived from the authoritative document composite,
never from a cached full image that a region refresh has left stale. The moved
layer's image, its document-space origin, and its opacity that the preview exposes
SHALL be unchanged.

#### Scenario: The move-preview base equals a full composite with the layer hidden

- **WHEN** `begin_move_preview` runs on a document whose cached canvas is current
- **THEN** the returned base image is byte-identical to a full composite of the
  document with the topmost pixel layer hidden

#### Scenario: The preview start does not composite the whole document

- **WHEN** a Move-tool drag starts on a 4000×4000 document
- **THEN** only the moved layer's rectangle is composited, and the document's
  stored composite and the persistent cached canvas are unchanged

#### Scenario: A stale cached image is not used for the preview base

- **WHEN** a region refresh has marked the display dirty and a Move-tool drag then
  starts
- **THEN** the preview base is built from the document composite, not from the
  stale cached image, and equals a full composite with the layer hidden

## ADDED Requirements

### Requirement: C++ region blit and present-cache coherence

The view SHALL provide a C++ region blit
(`ImageView::blitRegion(const QImage& region, int x, int y)`) that overwrites the
canvas at `(x, y)` using `QPainter` with `CompositionMode_Source`. The blit SHALL
keep the transparency checkerboard, the document clip, and the pan/zoom behaviour
unchanged, SHALL invalidate the zoom-level present cache so the next paint rebuilds
it at the current zoom, and SHALL schedule a repaint. There SHALL be no per-pixel
FFI blit of a region into the canvas. The blitted canvas SHALL be pixel-identical
to a full recomposite of the document. `PictureView::image()` SHALL return the
cached full image when the display is clean and SHALL rebuild it from the
document's planar composite when the display is dirty; `sample_argb` SHALL read the
document's planar composite directly without building a full image; and
`move_preview_base` SHALL be derived from the current composite, never from a stale
cached image.

#### Scenario: The C++ blit path is exercised

- **WHEN** a region refresh is emitted while its document is active
- **THEN** `ImageView::blitRegion` applies it, the canvas image equals the
  document composite, and the present cache is invalidated and rebuilt on the next
  paint

#### Scenario: The on-screen canvas after a region refresh matches a full recomposite

- **WHEN** a region refresh is followed by a forced full recomposite of the same
  document
- **THEN** the canvas image before the recomposite is pixel-identical to the image
  after it

#### Scenario: `image()` rebuilds from the composite only when dirty

- **WHEN** `image()` is called after a region refresh and again after a full
  recomposite
- **THEN** the first call returns an image rebuilt from the document composite and
  byte-identical to a full recomposite, and the second returns the cached image

#### Scenario: `sample_argb` reads the current composite

- **WHEN** `sample_argb(x, y)` is called after a region refresh
- **THEN** it returns the document composite's pixel at `(x, y)`, not a stale
  displayed value, and does not build a full image

#### Scenario: The existing M31/M32 canvas-equality checks still hold

- **WHEN** the M31 region-move and M32 move-preview/visibility self-tests run after
  a region refresh
- **THEN** the canvas they compare is byte-identical to a full recomposite, as
  before
