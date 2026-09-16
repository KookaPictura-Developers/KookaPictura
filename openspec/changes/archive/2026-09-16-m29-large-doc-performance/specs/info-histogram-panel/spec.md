## ADDED Requirements

### Requirement: Panels do not re-composite per refresh

The Info panel's pixel sampling SHALL read the already-current composited image
(or cached document composite) rather than running a fresh composite for each
sample. The Histogram SHALL be computed from a bounded downsample of at most
512×512 so that its cost is independent of document size. Both SHALL remain
visually equivalent to the previous full-resolution results.

#### Scenario: Sampling a large document performs no full composite

- **WHEN** a pixel is sampled in the Info panel on a 4000×4000 document whose
  composite is already current
- **THEN** the sample is read from the cached composite and no full-document
  composite is run

#### Scenario: Histogram refresh does not scan every pixel

- **WHEN** the Histogram panel refreshes on a 4000×4000 document
- **THEN** it bins a downsample of at most 512×512 and the plotted distribution
  remains visually equivalent
