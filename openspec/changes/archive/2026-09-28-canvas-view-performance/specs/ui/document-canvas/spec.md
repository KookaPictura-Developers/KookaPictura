# Spec Delta

## MODIFIED Requirements

### Requirement: Zoom-cached canvas present

The canvas SHALL present the document by cropping a view-pyramid level instead
of scaling the full-resolution document on every paint. The canvas SHALL choose
the coarsest level whose scale is still at least as fine as the screen: level 0
for zoom > 0.5; level 1 for 0.25 < zoom <= 0.5; level 2 for 0.125 < zoom <= 0.25;
and so on. A pan or hover repaint MUST NOT resample the full-resolution document.
There SHALL be no pixel-count bound that disables the crop path: a document of
any size SHALL present through a level crop.

#### Scenario: A pan repaint does not resample the document

- **WHEN** the view is panned without changing the zoom
- **THEN** the paint crops the level chosen for the zoom and does not rescale the
  full-resolution document

#### Scenario: A zoom change invalidates the cache

- **WHEN** the zoom changes across a level boundary
- **THEN** the next paint chooses the corresponding level and presents it

#### Scenario: Cached present matches the transform draw

- **WHEN** the canvas paints a document at zoom 100 % or above (level 0) through
  the level crop and through a direct full-resolution transform draw, with the
  same sampling filter
- **THEN** the two rendered images are pixel-identical

#### Scenario: A coarse-level present is a single reduction

- **WHEN** the canvas paints at a zoom that selects a level below 0
- **THEN** the source is that level's crop, a single reduction of the composite,
  and the paint does not rescale the full-resolution document

#### Scenario: A document past the former cache bound still presents through a crop

- **WHEN** a document whose scaled present would exceed 64 megapixels is painted
  at a high zoom
- **THEN** the paint still crops a level rather than drawing the full-resolution
  source, and does not fall back to the transform draw

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
document. `PictureView::image()` SHALL return a cached full image when the display
is clean and SHALL rebuild it from the document's planar composite when the
display is dirty; `sample_argb` SHALL read the document's planar composite directly
without building a full image; and `move_preview_base` SHALL be derived from the
current composite, never from a stale cached image. A full-resolution image MAY be
built on demand for the panels and self-tests that consume `image()`; the repaint
path MUST NOT depend on it.

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
- **THEN** the first call returns an image rebuilt from the document composite and
  byte-identical to a full recomposite, and the second returns the cached image

#### Scenario: `sample_argb` reads the current composite

- **WHEN** `sample_argb(x, y)` is called after a region refresh
- **THEN** it returns the document composite's pixel at `(x, y)`, not a stale
  displayed value, and does not build a full image

#### Scenario: The existing M31/M32 canvas-equality checks still hold

- **WHEN** the M31 region-move and M32 move-preview/visibility self-tests run
  after a region refresh
- **THEN** the canvas they compare is byte-identical to a full recomposite, as
  before

## ADDED Requirements

### Requirement: Damage account for partial repaint

The app's Rust view SHALL track outstanding display damage as an account that
distinguishes described regions from undescribed edits. A mutation that does not
describe its changed region SHALL mark the whole canvas as damaged. A described
mark SHALL only add to the damage and MUST NOT clear or mask an earlier
undescribed edit. Taking the damage SHALL clip it to the canvas and clear the
account. Any sequence of described and undescribed mutations SHALL leave the
presented composite byte-identical to a full recomposite.

#### Scenario: Nothing changed means nothing to redraw

- **WHEN** no mutation has occurred since the account was taken
- **THEN** taking the damage yields an empty region

#### Scenario: Described changes add up

- **WHEN** two described regions are marked without an intervening take
- **THEN** taking the damage yields their union, clipped to the canvas

#### Scenario: An undescribed change is the whole canvas

- **WHEN** a described region is marked and then an undescribed edit occurs
- **THEN** taking the damage yields the whole canvas

#### Scenario: A mark cannot hide an earlier undescribed change

- **WHEN** an undescribed edit occurs and then a small described region is marked
  before the canvas catches up
- **THEN** taking the damage yields the whole canvas, not the small region

#### Scenario: Taking starts afresh

- **WHEN** damage is taken and then taken again without any new mutation
- **THEN** the second take yields an empty region

### Requirement: Premultiplied display pixels

The Rust-to-Qt display conversion that hands canvas pixels to Qt SHALL produce
premultiplied RGBA, so Qt paints them without a per-frame conversion and averages
transparent edges correctly. The shared straight-alpha conversions used for
export encoding and for layer, mask, and histogram thumbnails MUST remain straight
alpha and MUST produce byte-identical output to before this change.

#### Scenario: A displayed pixel is premultiplied

- **WHEN** a straight-alpha composite pixel with alpha below 255 is converted for
  display
- **THEN** its color components equal `round(color * alpha / 255)` and its alpha
  is unchanged

#### Scenario: A transparent display pixel has zero color

- **WHEN** a composite pixel with alpha 0 is converted for display
- **THEN** its color components are 0

#### Scenario: Export and thumbnail bytes are unchanged

- **WHEN** the same composite is written through the export encoder or reduced for
  a layer, mask, or histogram thumbnail
- **THEN** the resulting bytes are identical to those produced before this change,
  not premultiplied

### Requirement: Zoom-dependent canvas sampling filter

When presenting a level crop below 200 % zoom the canvas SHALL sample it with
smooth (interpolated) filtering; at and above 200 % zoom it SHALL sample with
nearest-neighbour filtering so individual pixels stay crisp. The pixel-identity
guarantee compares two draws made with the same sampling filter; changing the
filter is an intended change of presented pixels, not a violation of it.

#### Scenario: Smooth below 200 %

- **WHEN** the canvas paints a document at a zoom below 200 %
- **THEN** the present draw uses smooth filtering

#### Scenario: Nearest at 200 % and above

- **WHEN** the canvas paints a document at 200 % zoom or greater
- **THEN** the present draw uses nearest-neighbour filtering

### Requirement: View-pyramid crop bridge

The view SHALL expose, across the Rust–Qt bridge, a crop of a chosen level and
queries for the level count and a level's size. Rectangles SHALL use the project's
`"x y w h"` string convention, levels SHALL be integers, and a level's size SHALL
use the project's `"w h"` string; no new marshalled rectangle type is introduced.
A crop MUST be premultiplied and sized to the requested rectangle. An empty
rectangle, an out-of-range level, or a rectangle that does not intersect the level
MUST return an empty image and MUST NOT panic. A rectangle that only partially
overlaps the level SHALL be returned at its requested size, transparent where it
falls outside the level.

#### Scenario: A bridge crop is a premultiplied rectangle

- **WHEN** a level and an in-range rectangle are requested across the bridge
- **THEN** the returned image has the rectangle's size, its pixels are that
  level's, premultiplied, and the reported level count and size match the pyramid

#### Scenario: An out-of-range request is empty, not a panic

- **WHEN** a crop is requested for a level index at or past the level count, with
  an empty rectangle, or with a rectangle that does not intersect the level
- **THEN** the bridge returns an empty image and does not panic

#### Scenario: A partially overlapping crop is transparent outside the level

- **WHEN** a rectangle that only partially overlaps the level is requested
- **THEN** the bridge returns a rectangle-sized image whose overlapping pixels are
  the level's, premultiplied, and whose outside pixels are transparent
