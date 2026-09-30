# Spec Delta

## MODIFIED Requirements

### Requirement: C++ region blit and present-cache coherence

The view SHALL provide a C++ region blit
(`ImageView::blitRegion(const QImage& region, int x, int y)`) that draws the
region at `(x, y)` using `QPainter` with `CompositionMode_Source`. The blit SHALL
keep the transparency checkerboard, the document clip, and the pan/zoom behaviour
unchanged, and SHALL schedule a repaint. The view pyramid SHALL be updated from a
region refresh by the app's Rust view through the damage account, so the next
paint crops the updated levels; the C++ blit MUST NOT be responsible for updating
the pyramid. There SHALL be no per-pixel FFI blit of a region into the canvas.
The displayed canvas SHALL be pixel-identical to a full recomposite of the
document. `PictureView::image()` SHALL build a full-resolution image on demand
from the cached level-0 frame (or the document's planar composite when there is
no level-0 frame) and MUST NOT hold a persistent full-resolution `QImage`; the
repaint path MUST NOT depend on it. `sample_argb` SHALL read the document's
planar composite directly without building a full image; and `move_preview_base`
SHALL be derived from the current composite, never from a stale cached image.

#### Scenario: The C++ blit path is exercised

- **WHEN** a region refresh is emitted while its document is active
- **THEN** `ImageView::blitRegion` draws it, the app's Rust view updates the
  affected pyramid levels, and the next paint crops those updated levels

#### Scenario: The on-screen canvas after a region refresh matches a full recomposite

- **WHEN** a region refresh is followed by a forced full recomposite of the same
  document
- **THEN** the canvas image before the recomposite is pixel-identical to the image
  after it

#### Scenario: `image()` rebuilds from the composite only when dirty

- **WHEN** `image()` is called after a region refresh and again after a full
  recomposite
- **THEN** both calls return an image built from the cached level-0 frame and
  byte-identical to a full recomposite, and no persistent full-resolution `QImage`
  is held between calls

#### Scenario: `sample_argb` reads the current composite

- **WHEN** `sample_argb(x, y)` is called after a region refresh
- **THEN** it returns the document composite's pixel at `(x, y)`, not a stale
  displayed value, and does not build a full image

#### Scenario: The existing M31/M32 canvas-equality checks still hold

- **WHEN** the M31 region-move and M32 move-preview/visibility self-tests run
  after a region refresh
- **THEN** the canvas they compare is byte-identical to a full recomposite, as
  before

#### Scenario: image() builds the full image on demand

- **WHEN** `image()` is called for an open document
- **THEN** it returns an image built from the current level-0 frame and
  byte-identical to a full recomposite

#### Scenario: No persistent full-resolution QImage is held

- **WHEN** a document is open and no caller has consumed `image()`
- **THEN** the view holds no persistent full-resolution `QImage`

#### Scenario: The repaint path does not depend on `image()`

- **WHEN** the canvas repaints after a region refresh at any zoom, before any
  caller has consumed `image()`
- **THEN** it presents a view-pyramid level crop and does not build or read a
  full-resolution image

## ADDED Requirements

### Requirement: Mid-stroke canvas present from the view pyramid

While a paint stroke is in progress the canvas SHALL present the same
zoom-selected view-pyramid level crop it presents when idle, rather than
rescaling the full-resolution document. The pyramid levels SHALL reflect the
stroke's in-progress pixels after every dab. The presented canvas after any dab
MUST be pixel-identical to the canvas a full recomposite of the stroke's working
document would produce at the same zoom and sampling filter. A document of any
size SHALL present through a level crop mid-stroke, and the repaint MUST NOT
resample the full-resolution document.

#### Scenario: A dab repaints through a level crop

- **WHEN** the canvas repaints after a dab while a stroke is in progress
- **THEN** it presents the level crop chosen for the zoom and does not rescale the
  full-resolution document

#### Scenario: The mid-stroke present equals a full recomposite

- **WHEN** a dab has been applied and the canvas is presented mid-stroke
- **THEN** the presented pixels are identical to a full recomposite of the
  stroke's working document at the same zoom and sampling filter

#### Scenario: A large document presents through a crop mid-stroke

- **WHEN** a stroke is painted on a 16000²-class document
- **THEN** each repaint crops a level rather than drawing the full-resolution
  source
