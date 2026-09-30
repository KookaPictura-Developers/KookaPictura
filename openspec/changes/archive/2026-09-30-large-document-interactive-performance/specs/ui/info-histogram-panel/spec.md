# Spec Delta

## MODIFIED Requirements

### Requirement: Panels do not re-composite per refresh

The Info panel's pixel sampling SHALL read the document's planar composite
directly rather than building a full-resolution image. The Histogram SHALL be
computed from a view-pyramid level — a bounded reduction of the composite of at
most 512×512 — so that its cost is independent of document size, and it MUST NOT
read `PictureView::image()` or otherwise force a full-resolution image rebuild.
Both SHALL remain visually equivalent to the previous full-resolution results.

#### Scenario: Sampling a large document performs no full composite

- **WHEN** a pixel is sampled in the Info panel on a 4000×4000 document whose
  composite is already current
- **THEN** the sample is read from the cached composite and no full-document
  composite is run

#### Scenario: Histogram refresh does not scan every pixel

- **WHEN** the Histogram panel refreshes on a 4000×4000 document
- **THEN** it bins a downsample of at most 512×512 and the plotted distribution
  remains visually equivalent

#### Scenario: Histogram refresh does not read the full-resolution image

- **WHEN** the Histogram panel refreshes after a region refresh
- **THEN** it reads a view-pyramid level, does not call `PictureView::image()`,
  and no full-resolution image rebuild runs

#### Scenario: Histogram cost is independent of document size

- **WHEN** the Histogram panel refreshes on a document at least twice as large in
  each dimension
- **THEN** the work it does is bounded by the chosen pyramid level and does not
  grow with the document's pixel count
