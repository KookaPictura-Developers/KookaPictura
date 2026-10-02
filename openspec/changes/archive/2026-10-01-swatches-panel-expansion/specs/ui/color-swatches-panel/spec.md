# color-swatches-panel Specification

## Purpose

Extend the Swatches panel from a fixed grid to a reflowing, named, editable
swatch grid with CS6's click modifiers, keeping the existing default palette.

## ADDED Requirements

### Requirement: Reflowing swatch grid

The Swatches panel SHALL present its swatches in a grid whose column count
follows the width it is given, so the panel shows more swatches per row as it is
made wider and its height follows from the resulting row count. A point SHALL
map to the swatch whose cell contains it, and to no swatch in a gap or past the
end.

#### Scenario: Width decides the columns [sw_reflow_columns]

- **WHEN** the grid is given a width that fits five swatch cells
- **THEN** its column count is five and its height is the rows that implies

#### Scenario: A point maps to a swatch [sw_reflow_index_at]

- **WHEN** a point inside the first swatch's cell is tested
- **THEN** `indexAt` returns the first swatch's index

#### Scenario: A point past the end maps to nothing [sw_reflow_past_end]

- **WHEN** a point below the last row of swatches is tested
- **THEN** `indexAt` returns -1

### Requirement: Swatch click modifiers

A plain click on a swatch SHALL set the foreground colour, a ctrl-click SHALL set
the background colour, and an alt-click SHALL remove the swatch. Each named
swatch SHALL show its name as a tooltip.

#### Scenario: Plain click sets the foreground [sw_click_foreground]

- **WHEN** a swatch is clicked without a modifier
- **THEN** the foreground colour becomes that swatch's colour

#### Scenario: Ctrl-click sets the background [sw_click_background]

- **WHEN** a swatch is ctrl-clicked
- **THEN** the background colour becomes that swatch's colour

#### Scenario: Alt-click deletes [sw_click_alt_delete]

- **WHEN** a swatch is alt-clicked
- **THEN** the swatch is removed from the grid

### Requirement: Swatch footer and context menu

The panel SHALL provide New and Delete actions in a footer and SHALL provide
Reset in the context menu. New SHALL add a swatch from the current foreground
colour, Delete SHALL remove the marked swatch, and Reset SHALL restore the
default palette. Clicking the empty space past the last swatch SHALL also
request a new swatch.

#### Scenario: New adds a foreground swatch [sw_footer_new]

- **WHEN** the New action is invoked with a foreground colour set
- **THEN** a swatch of that colour is appended to the grid

#### Scenario: Reset restores the palette [sw_footer_reset]

- **WHEN** the Reset action is invoked after swatches were added or removed
- **THEN** the grid holds the default palette again

### Requirement: Existing default palette preserved

The default swatch library SHALL preserve the colours of the previous fixed grid.

#### Scenario: Default palette matches [sw_default_palette]

- **WHEN** a new Swatches panel is constructed
- **THEN** the grid holds the forty-eight colours of the previous palette
