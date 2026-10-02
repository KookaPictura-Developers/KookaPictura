# info-histogram-panel Specification

## Purpose

Extend the Info panel with the CS6 CMYK colour readout and the Ruler tool's
angle/length readout, without adding document-size state.

## ADDED Requirements

### Requirement: CMYK colour readout

The Info panel SHALL show the CMYK components of the colour under the cursor
beside the existing RGB readout, derived from the sampled pixel with no engine
call beyond the sample. Each component SHALL be reported as a 0–100 percentage.
Off-canvas positions SHALL blank the CMYK readout.

#### Scenario: Cursor over a white pixel [info_cmyk_white]

- **WHEN** the cursor is over a white pixel of a document
- **THEN** the Info panel shows `C 0  M 0  Y 0  K 0` beside the RGB readout

#### Scenario: Cursor leaves the canvas [info_cmyk_offcanvas]

- **WHEN** the cursor moves off the canvas
- **THEN** the CMYK readout is blank rather than holding its last value

### Requirement: Ruler readout mode

When the Ruler tool is active, the Info panel SHALL show the ruler's angle (A)
and length (L), and SHALL report the W/H block from the ruler's measurement
rather than the document dimensions. When no measuring line exists, the A/L and
W/H blocks SHALL be blank. Switching back to another tool SHALL restore the
document-dimensions readout.

#### Scenario: A measuring line fills A/L and W/H [info_ruler_measurement]

- **WHEN** the Ruler tool is active and a measuring line runs from (2, 3) to (2, 13)
- **THEN** the panel shows A `-90.0°` and L `10.0`, and the W/H block reads `0.0 × 10.0`

#### Scenario: No measuring line [info_ruler_empty]

- **WHEN** the Ruler tool is active and no measuring line has been drawn
- **THEN** the A/L and W/H blocks are blank

#### Scenario: Leaving ruler mode restores document dimensions [info_ruler_off]

- **WHEN** the active tool changes away from the Ruler
- **THEN** the W/H block reads the document dimensions again
