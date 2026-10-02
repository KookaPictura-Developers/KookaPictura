# info-histogram-panel Specification

## Purpose

Rework the Info panel into CS6's 2×2 readout grid with per-block colour-mode and
measurement-unit menus, a selection-driven W/H block, a ruler-mode block swap,
and a `Doc:` memory line.

## ADDED Requirements

### Requirement: Info readout grid

The Info panel SHALL present four readout blocks in a 2×2 grid, each block an
auto-raise `QToolButton` (opening its menu on a single click) beside rows of
`Key : value`. The top-left and top-right blocks SHALL be the colour blocks; the
bottom-left SHALL report the cursor position as X/Y; the bottom-right SHALL report
the size as W/H. Each colour block SHALL carry a static `8-bit` footer label.

#### Scenario: The grid shows four blocks

- **WHEN** a document is open
- **THEN** the panel shows a colour block with an R/G/B readout, a colour block
  with a C/M/Y/K readout, an X/Y position block, and a W/H size block

#### Scenario: No document

- **WHEN** no document is open
- **THEN** every block's values are blank and the panel does not crash

### Requirement: Colour-mode menu

Each colour block's button SHALL open a colour-mode menu offering Grayscale, RGB,
HSB, CMYK, and Lab. Selecting a mode SHALL rebuild that block's key rows and
format its values: Grayscale one `K` row (0–255, `qGray`); RGB `R`,`G`,`B`
(0–255); HSB `H` (0–360), `S`,`B` (0–100%); CMYK `C`,`M`,`Y`,`K` (0–100%); Lab
`L` (0–100), `a`,`b` (−128..127). The top-left block SHALL default to RGB and the
top-right to CMYK. The two blocks' modes SHALL be independent.

#### Scenario: Switching the top-left block to CMYK [info_color_mode_cmyk]

- **WHEN** the top-left block's menu selection changes from RGB to CMYK
- **THEN** its rows become `C`,`M`,`Y`,`K` and the values are 0–100 percentages

#### Scenario: Grayscale uses the grey value [info_color_mode_gray]

- **WHEN** the top-left block is set to Grayscale over a white pixel
- **THEN** its single `K` row reads `255`

### Requirement: Measurement-unit menu

Each measurement block's button SHALL open a measurement-unit menu offering
Pixels, Inches, Centimeters, Millimeters, Points, Picas, and Percent, and SHALL
apply the chosen unit to that block only. Conversion SHALL use 72 PPI: inches =
pixels / 72, centimeters = inches × 2.54, millimeters = inches × 25.4, points =
inches × 72, picas = inches × 6, and percent = the value against the relevant
document dimension (X and W against the width, Y and H against the height).
Non-pixel units SHALL be formatted to one or two decimals; Pixels SHALL be whole.

#### Scenario: Milliseconds to millimetres [info_unit_mm]

- **WHEN** the position block is set to Millimeters with the cursor at 2 px
- **THEN** X and Y each read `0.71` (2 / 72 × 25.4)

#### Scenario: Each block keeps its own unit [info_unit_per_block]

- **WHEN** the position block is set to Millimeters and the size block to Pixels
- **THEN** the position values are converted and the size values stay whole pixels

### Requirement: Document size line

The Info panel SHALL show a `Doc:` line reporting the document's pixel-plane
memory footprint and its on-disk file size, read from the `document_size_bytes`
bridge (`[memory, disk]`). The memory SHALL sum the composite, every layer's
channel planes (recursing groups), layer masks, and the document extra channels;
the disk size SHALL be the file's byte length, or 0 for an unsaved document.

#### Scenario: An open document shows a Doc line [info_doc_line]

- **WHEN** a document is open
- **THEN** the panel shows a non-empty `Doc:` line

## MODIFIED Requirements

### Requirement: Info panel readouts

The system SHALL show, for the active document, the cursor position in image
coordinates and the colour under the cursor in the selected colour mode. The size
block SHALL report the active selection's width and height, and SHALL be blank
when nothing is selected. The readouts SHALL update as the cursor moves and the
document changes. The document-dimensions row SHALL be removed.

#### Scenario: Cursor readouts update

- **WHEN** the pointer moves over the canvas
- **THEN** the Info panel shows the image coordinates and the colour at that point

#### Scenario: Selection readout

- **WHEN** a rectangular selection exists
- **THEN** the W/H block shows the selection's width and height

#### Scenario: No selection

- **WHEN** nothing is selected
- **THEN** the W/H block is blank

#### Scenario: No document

- **WHEN** no document is open
- **THEN** the readouts show placeholders and do not crash

### Requirement: CMYK colour readout

The Info panel SHALL show the CMYK components of the colour under the cursor in
the top-right block by default, derived from the sampled pixel with no engine call
beyond the sample. Each component SHALL be reported as a 0–100 percentage.
Off-canvas positions SHALL blank the colour blocks.

#### Scenario: Cursor over a white pixel [info_cmyk_white]

- **WHEN** the cursor is over a white pixel of a document
- **THEN** the top-right block shows `C : 0  M : 0  Y : 0  K : 0`

#### Scenario: Cursor leaves the canvas [info_cmyk_offcanvas]

- **WHEN** the cursor moves off the canvas
- **THEN** the colour blocks are blank rather than holding their last values

### Requirement: Ruler readout mode

When the Ruler tool is active, the Info panel SHALL swap the top-right block to
the ruler's angle (A) and length (L) with a protractor glyph, and SHALL report
the W/H block from the ruler's measurement rather than the selection. When no
measuring line exists, the A/L and W/H blocks SHALL be blank. Switching away from
the Ruler SHALL restore the top-right colour block and the selection-size W/H.

#### Scenario: A measuring line fills A/L and W/H [info_ruler_measurement]

- **WHEN** the Ruler tool is active and a measuring line runs from (2, 3) to (2, 13)
- **THEN** the top-right block shows A `-90.0°` and L `10.0`, and the W/H block
  reads the ruler deltas

#### Scenario: No measuring line [info_ruler_empty]

- **WHEN** the Ruler tool is active and no measuring line has been drawn
- **THEN** the A/L and W/H blocks are blank

#### Scenario: Leaving ruler mode restores document dimensions [info_ruler_off]

- **WHEN** the active tool changes away from the Ruler
- **THEN** the top-right block is the CMYK readout again and the W/H block follows
  the selection
