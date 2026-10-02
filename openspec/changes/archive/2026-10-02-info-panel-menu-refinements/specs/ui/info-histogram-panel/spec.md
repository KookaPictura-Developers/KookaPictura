# info-histogram-panel Specification

## Purpose

Refine the Info panel's 2×2 readout grid to CS6 behaviour: the full eyedropper
colour menu with bit-depth selection, a W/H block that inherits its unit from
the X/Y block, a left-aligned bit-depth footer in the content area, and
table-style dark-gray section lines.

## MODIFIED Requirements

### Requirement: Info readout grid

The Info panel SHALL present four readout blocks in a 2×2 grid, each block an
auto-raise `QToolButton` (opening its menu on a single click) beside rows of
`Key : value`. The top-left and top-right blocks SHALL be the colour blocks; the
bottom-left SHALL report the cursor position as X/Y; the bottom-right SHALL
report the size as W/H. The grid SHALL use zero horizontal and vertical spacing.
Each block SHALL draw a single 1px `#3a3a3a` border on its right and bottom
edges, plus a top edge in grid row 0 and a left edge in grid column 0, so
adjacent cells share one internal line and the grid has one outer border.
Each colour block's bit-depth footer SHALL sit in the content area to the right
of the icon and SHALL be left- and bottom-aligned. A 1px `#3a3a3a` line SHALL
separate the grid from the `Doc:` line.

#### Scenario: The grid shows four blocks

- **WHEN** a document is open
- **THEN** the panel shows a colour block with an R/G/B readout, a colour block
  with a C/M/Y/K readout, an X/Y position block, and a W/H size block

#### Scenario: No document

- **WHEN** no document is open
- **THEN** every block's values are blank and the panel does not crash

#### Scenario: Shared table separators

- **WHEN** the panel is shown
- **THEN** the four blocks are divided by single dark-gray lines with a single
  outer border and a `#3a3a3a` rule above the `Doc:` line

#### Scenario: Bit-depth footer placement

- **WHEN** a colour block shows its bit-depth footer
- **THEN** the footer is left- and bottom-aligned in the content area to the
  right of the icon

### Requirement: Colour-mode menu

Each colour block's button SHALL open a menu listing, in order, Actual Color,
Proof Color, a separator, Grayscale, RGB, HSB, CMYK, Lab, a separator, Total
Ink, Opacity, a separator, and the 8-bit, 16-bit, and 32-bit depth entries. The
colour-mode entries and the three depth entries SHALL be checkable, and their
checked state SHALL reflect the block's current mode and depth when the menu
opens. The top-left block SHALL default to Actual Color, the top-right block to
CMYK, both at 8-bit, and the two blocks' modes and depths SHALL be independent.

Selecting a mode SHALL rebuild that block's key rows and format its values.
Actual Color and Proof Color SHALL show R, G, B; Proof Color SHALL read the same
sRGB values as Actual Color. Grayscale SHALL show one `K` row (`qGray`). RGB
SHALL show R, G, B. HSB SHALL show `H` (0–360), `S`, `B` (0–100%). CMYK SHALL
show `C`,`M`,`Y`,`K` (0–100%). Lab SHALL show `L` (0–100), `a`,`b` (rounded
integers). Total Ink SHALL show one `Ink` row equal to the sum of the CMYK
percentages (0–400). Opacity SHALL show one `Op` row equal to the sampled
alpha as a percentage (`qAlpha / 255 × 100`). Off-canvas positions SHALL blank
the colour blocks.

#### Scenario: Switching the top-left block to CMYK [info_color_mode_cmyk]

- **WHEN** the top-left block's menu selection changes to CMYK
- **THEN** its rows become `C`,`M`,`Y`,`K` and the values are 0–100 percentages

#### Scenario: Grayscale uses the grey value [info_color_mode_gray]

- **WHEN** the top-left block is set to Grayscale over a white pixel
- **THEN** its single `K` row reads `255`

#### Scenario: Total Ink and Opacity rows

- **WHEN** the top-left block is set to Total Ink and then Opacity over an
  opaque white pixel
- **THEN** Total Ink reads `Ink : 0` and Opacity reads `Op : 100`

#### Scenario: Top-left defaults to Actual Color

- **WHEN** the panel is first constructed
- **THEN** the top-left block's mode is Actual Color and the top-right block's
  mode is CMYK

### Requirement: Measurement-unit menu

The position (X/Y) block's button SHALL open a measurement-unit menu offering
Pixels, Inches, Centimeters, Millimeters, Points, Picas, and Percent. The chosen
unit SHALL apply to both the position and size blocks; the W/H block SHALL NOT
have its own menu and SHALL inherit the X/Y unit. Conversion SHALL use 72 PPI:
inches = pixels / 72, centimeters = inches × 2.54, millimeters = inches × 25.4,
points = inches × 72, picas = inches × 6, and percent = the value against the
relevant document dimension (X and W against the width, Y and H against the
height). Non-pixel units SHALL be formatted to one or two decimals; Pixels SHALL
be whole.

#### Scenario: Milliseconds to millimetres [info_unit_mm]

- **WHEN** the position block is set to Millimeters with the cursor at 2 px
- **THEN** X and Y each read `0.71` (2 / 72 × 25.4)

#### Scenario: Each block keeps its own unit [info_unit_per_block]

- **WHEN** the X/Y block is set to Millimeters with a selection present
- **THEN** the position values convert and the W/H block is reformatted in the
  same unit

#### Scenario: W/H has no menu [info_unit_size_no_menu]

- **WHEN** the size block is inspected
- **THEN** its button carries no menu and its unit follows the position block

## ADDED Requirements

### Requirement: Colour bit depth

Each colour block SHALL keep an independent bit depth of 8, 16, or 32,
selectable from the depth entries of its colour menu, and its footer SHALL read
`8-bit`, `16-bit`, or `32-bit`. Depth SHALL scale the channel values of Actual
Color, Proof Color, RGB, and Grayscale: 8-bit raw 0–255, 16-bit `v × 257`
(0–65535), and 32-bit `v / 255.0` to three decimals. HSB, CMYK, Lab, Total Ink,
and Opacity SHALL be depth-independent. `ponytail:` the engine samples 8-bit, so
the 16- and 32-bit readouts are scaled from the 8-bit sample rather than read
from a true deep-colour document.

#### Scenario: 16-bit scales an RGB channel [info_bit_depth_16]

- **WHEN** the top-left block is at 16-bit over a white pixel
- **THEN** the `R` value is `65535` and the footer reads `16-bit`

#### Scenario: Depth does not change percentages [info_bit_depth_independent]

- **WHEN** a block is set to CMYK at 16-bit
- **THEN** the C/M/Y/K values stay 0–100 percentages and only the footer changes
