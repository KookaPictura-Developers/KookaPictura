## ADDED Requirements

### Requirement: Column header drag

The top header of a `PanelColumn` SHALL act as a drag handle for the whole
column — the row carrying the `panelColumnToggle`. A left-button press on the
header followed by movement past the drag threshold SHALL move the column and
SHALL show the same single insertion indicator the widget drag grammar uses; on
release the column SHALL land immediately to the left or right of the widget
column under the pointer, or at the workspace outer edge when no column is under
it. A release that resolves no target SHALL leave the column where it is. A click
on the width toggle SHALL still switch the column's normal/icon mode and SHALL
NOT start a column drag.

#### Scenario: The header drags the column beside another [pc_col_header_move]

- **WHEN** the top header of a widget column is pressed and dragged onto another
  column
- **THEN** the insertion indicator is shown and on release the dragged column is
  placed on that side of the target column

#### Scenario: The toggle still toggles [pc_col_header_toggle]

- **WHEN** the column's width toggle is clicked
- **THEN** the column switches between normal and icon mode and no column drag
  starts

### Requirement: Panel group tab overflow

When a panel group's tabs do not fit its tab bar, the tabs SHALL compress and
elide to share the available width rather than showing scroll arrow buttons, and
the per-widget corner control SHALL keep its place.

#### Scenario: Many tabs squeeze instead of scrolling [pc_tabs_compress]

- **WHEN** a group has more visible tabs than fit the column width
- **THEN** the tabs are squeezed and elided and no scroll-arrow buttons appear

### Requirement: Reserved panel-group drag grip

Each panel group SHALL reserve a small blank drag area at the right of its tab
bar, before the per-widget menu control, and a press-drag on it SHALL drag the
whole group through the same drag grammar as the empty tab-bar area. The reserved
area SHALL remain present when the tabs fill or overflow the bar.

#### Scenario: The reserved grip drags the group [pc_group_grip]

- **WHEN** the reserved area to the right of the tabs is pressed and dragged
- **THEN** the whole group is dragged, even when the tab bar is full

### Requirement: Floating group top bar and icon mode

A floating panel group SHALL carry its own top bar with a normal/icon width
toggle and a close control, and the bar SHALL drag the overlay. The toggle SHALL
collapse the group to its icon row and expand it again, and the close control
SHALL remain reachable while collapsed. While collapsed the overlay SHALL keep at
least the height its icon row needs, and the icon row SHALL render on the panel
surface like the docked iconic representation.

#### Scenario: The float top bar toggles and closes [pc_float_header]

- **WHEN** a group floats
- **THEN** its top bar shows a collapse toggle and a close control, toggling
  collapses and expands the group, and dragging the bar moves the overlay

#### Scenario: The floating icon overlay keeps a minimum height [pc_float_icon_min]

- **WHEN** a floating group is collapsed to icons
- **THEN** the overlay is at least the icon-row minimum height and the close
  control remains reachable
