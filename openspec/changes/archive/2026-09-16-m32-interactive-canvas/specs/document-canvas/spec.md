## ADDED Requirements

### Requirement: Region-composited move-preview base

Starting a Move-tool preview (`PictureView::begin_move_preview`) MUST NOT
composite the whole document. It SHALL build the preview base by region-
compositing only the moved layer's clamped document rectangle with that layer
hidden (`visible = false`) against the current cached canvas, and MUST NOT modify
the document's stored `composite` or the persistent cached canvas. The resulting
base image MUST be byte-identical to a full recomposite of the document with that
layer hidden. The moved layer's image, its document-space origin, and its opacity
that the preview exposes SHALL be unchanged.

#### Scenario: The move-preview base equals a full composite with the layer hidden

- **WHEN** `begin_move_preview` runs on a document whose cached canvas is current
- **THEN** the returned base image is byte-identical to a full composite of the
  document with the topmost pixel layer hidden

#### Scenario: The preview start does not composite the whole document

- **WHEN** a Move-tool drag starts on a 4000×4000 document
- **THEN** only the moved layer's rectangle is composited, and the document's
  stored composite and the persistent cached canvas are unchanged

### Requirement: Region-composited layer visibility toggle

Toggling a layer's visibility (`PictureView::set_layer_visible`) SHALL refresh
only the toggled layer's influence rectangle when that is provably equivalent to
a full recomposite — a raster layer's clamped `rect`, or an adjustment layer's
mask rectangle only when the mask is enabled, carries pixel data, and has a zero
`default_color` — rather than recompositing the whole document. When the toggle
is not provably confined to such a rectangle — an adjustment layer with an
unmasked mask or a non-zero `default_color`, a group, or any layer whose effect
cannot be bounded — it MUST fall back to a full recomposite. The refreshed canvas
MUST be byte-identical to a full recomposite of the document after the toggle.

#### Scenario: Toggling a raster layer updates only its rectangle

- **WHEN** a raster layer that covers a sub-rectangle of a 4000×4000 document is
  hidden or shown
- **THEN** only that layer's clamped rectangle is composited, the rest of the
  canvas keeps its previous value, and the result equals a full recomposite

#### Scenario: A bounded adjustment toggle is byte-identical

- **WHEN** an adjustment layer whose enabled mask carries pixel data with a zero
  `default_color` is hidden or shown
- **THEN** only its mask rectangle is composited and the canvas equals a full
  recomposite of the document

#### Scenario: An unboundable toggle falls back to a full recomposite

- **WHEN** a group layer, or an adjustment layer with an unmasked mask or a
  non-zero `default_color`, is hidden or shown
- **THEN** the canvas is recomposited in full and equals a full recomposite of the
  document

### Requirement: Zoom-cached canvas present

The canvas SHALL present the document from an image cached at the current zoom
instead of re-scaling the full-resolution document on every paint. The cache SHALL
be invalidated when the document image or the zoom changes and rebuilt on the next
paint; while the cache is valid a pan or hover repaint MUST NOT resample the
full-resolution document. The presented pixels MUST match the previous
transform-based paint. When the scaled image would exceed a fixed pixel bound the
canvas MAY fall back to the transform draw, which MUST be visually identical.

#### Scenario: A pan repaint does not resample the document

- **WHEN** the view is panned without changing the zoom
- **THEN** the paint uses the zoom cache and does not rescale the full-resolution
  document

#### Scenario: A zoom change invalidates the cache

- **WHEN** the zoom changes
- **THEN** the next paint rebuilds the cache at the new zoom

#### Scenario: Cached present matches the transform draw

- **WHEN** the canvas paints a document at a given zoom and pan with and without
  the zoom cache
- **THEN** the two rendered images are pixel-identical
