# info-histogram-panel Specification

## Purpose

The Info readout grid's table styling and menu placement: theme-shaded inner
separators instead of a hard-coded outer-framed box, and readout menus that
open beside their button rather than in front of it.

## MODIFIED Requirements

### Requirement: Info readout grid

The Info panel SHALL present four readout blocks in a 2×2 grid, each block an
auto-raise `QToolButton` (opening its menu on a single click) beside rows of
`Key : value`. The top-left and top-right blocks SHALL be the colour blocks; the
bottom-left SHALL report the cursor position as X/Y; the bottom-right SHALL
report the size as W/H. The grid SHALL use zero horizontal and vertical spacing.
Each block SHALL draw a single 1px separator on its right edge in grid column 0
and on its bottom edge in grid row 0, so the four blocks share one inner cross
and the grid draws no outer border — the panel pane is the outer edge. The
separator colour SHALL come from the theme's frame shade (the pane colour
darkened), never a hard-coded constant, so the lines stay darker than the pane
at every brightness level. Each colour block's bit-depth footer SHALL sit in the
content area to the right of the icon and SHALL be left- and bottom-aligned. A
1px rule in the same frame shade SHALL separate the grid from the `Doc:` line.
A block's menu SHALL open beside its button — to the right, or to the left when
the right side does not fit on the screen — and SHALL never cover the button
that opened it.

#### Scenario: The grid shows four blocks

- **WHEN** a document is open
- **THEN** the panel shows a colour block with an R/G/B readout, a colour block
  with a C/M/Y/K readout, an X/Y position block, and a W/H size block

#### Scenario: No document

- **WHEN** no document is open
- **THEN** every block's values are blank and the panel does not crash

#### Scenario: Shared table separators

- **WHEN** the panel is shown
- **THEN** the four blocks are divided by single theme-shade lines forming the
  inner cross, with no outer border, and a same-shade rule above the `Doc:` line

#### Scenario: Bit-depth footer placement

- **WHEN** a colour block shows its bit-depth footer
- **THEN** the footer is left- and bottom-aligned in the content area to the
  right of the icon

#### Scenario: Menu opens beside its button [info_menu_beside_button]

- **WHEN** a readout button's menu opens
- **THEN** the menu sits to the right of the button (or to its left when the
  right side does not fit) and does not overlap the button
