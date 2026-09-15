# info-histogram-panel Specification

## Purpose
TBD - created by archiving change m20-panels. Update Purpose after archive.
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

