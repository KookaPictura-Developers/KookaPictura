# info-histogram-panel Specification

## Purpose
Info panel readouts, a histogram view, and the Info and Histogram dock, without per-refresh recompositing.
## Requirements
### Requirement: Info panel readouts
The system SHALL show, for the active document, the cursor position in image
coordinates, the colour under the cursor, the active selection's pixel count (or
"none"), and the document dimensions. The readouts SHALL update as the cursor
moves and the document changes.

#### Scenario: Cursor readouts update
- **WHEN** the pointer moves over the canvas
- **THEN** the Info panel shows the image coordinates and the colour at that point

#### Scenario: Selection readout
- **WHEN** a selection exists
- **THEN** the Info panel shows its pixel count

#### Scenario: No document
- **WHEN** no document is open
- **THEN** the readouts show placeholders and do not crash

### Requirement: Histogram view
The system SHALL display a 256-bin histogram of the composite image with a
selector for the red, green, blue, and luminance channels, and SHALL recompute it
when the document changes.

#### Scenario: Channel selection
- **WHEN** the user selects a channel in the Histogram panel
- **THEN** the plotted distribution is for that channel

#### Scenario: Recompute on edit
- **WHEN** the document changes
- **THEN** the histogram is recomputed from the new composite

### Requirement: Info and Histogram dock and toggle
The system SHALL host the Info and Histogram panels in registered docks with
stable `objectName`s and SHALL expose `Window > Panels > Info` and
`Window > Panels > Histogram` toggles.

#### Scenario: Toggle the Histogram panel
- **WHEN** the user toggles Histogram from the Window menu
- **THEN** the panel is shown or hidden

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

