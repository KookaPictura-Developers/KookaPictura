## MODIFIED Requirements

### Requirement: Per-widget panel header action menu

Each panel group's tab header SHALL carry a context-action button at its far
right. Activating it SHALL show the menu of the panel that is current in the
group, so the menu is per-widget rather than per-group. Each menu SHALL contain
that panel's researched CS6 entries as recorded in `docs/dev/m42-panel-menus.md`,
in a stable order, and SHALL also offer `Close` (close the group's active tab)
and `Close Group` (close the whole group) at the end. An entry that is not
implemented SHALL be shown disabled with the tooltip `<label> — not implemented
yet` and SHALL NOT fake behaviour. `Close` and `Close Group` SHALL close the
active panel and the group through the same close path the tab context menu
uses. The Layers panel's `Panel Options…` entry SHALL open the existing Panel
Options behaviour.

#### Scenario: The header button shows the current panel's menu [m42_widgetmenu]

- **WHEN** the tab-header action button is activated for a group whose current
  panel is Layers
- **THEN** the Layers per-widget menu is shown

#### Scenario: Unimplemented entries are disabled with a tooltip [m42_widgetmenu]

- **WHEN** a per-widget menu contains an entry whose feature is not implemented
- **THEN** that entry is disabled and its tooltip is
  `<label> — not implemented yet`

#### Scenario: Close entries are present and act [m42_widgetmenu]

- **WHEN** a per-widget menu is shown and its `Close` or `Close Group` entry is
  chosen
- **THEN** the menu contains both entries, and choosing `Close` hides the active
  tab while `Close Group` hides the whole group

#### Scenario: Layers Panel Options still opens [m42_widgetmenu]

- **WHEN** `Panel Options…` is chosen from the Layers per-widget menu
- **THEN** the existing Layers Panel Options behaviour runs

### Requirement: Column header drag

The top header of a `PanelColumn` SHALL act as a drag handle for the whole
column — the row carrying the `panelColumnToggle`. A left-button press on the
header followed by movement past the drag threshold SHALL move the column and
SHALL show the same single insertion indicator the widget drag grammar uses; on
release the column SHALL land immediately to the left or right of the widget
column under the pointer, or at the workspace's outermost left or right position
when no column is under it — including to the left of the Tools panel when the
Tools panel is hosted as a splitter pane. A release that resolves no target SHALL
leave the column where it is. A left-button press and release on the header that
does not exceed the drag threshold SHALL open a header menu containing
`Collapse to Icons` (toggling the column's icon mode), `Auto-Collapse Iconic
Panels` and `Auto-show Hidden Panels` (checkable and bound to the column's
flags), and `Interface Options…` (opening the Interface preferences pane). A
click on the width toggle SHALL still switch the column's normal/icon mode and
SHALL NOT start a column drag or open the header menu.

#### Scenario: The header drags the column beside another [pc_col_header_move]

- **WHEN** the top header of a widget column is pressed and dragged onto another
  column
- **THEN** the insertion indicator is shown and on release the dragged column is
  placed on that side of the target column

#### Scenario: The column lands at the outermost position [pc_col_header_edge]

- **WHEN** the header is dragged to the workspace's outer left band with the
  Tools panel hosted as a splitter pane at that edge
- **THEN** the column lands at the splitter's outermost position, to the left of
  the Tools panel

#### Scenario: The header menu toggles the column options [pc_col_header_menu]

- **WHEN** the column header is clicked without dragging and a menu entry is
  chosen
- **THEN** `Collapse to Icons` toggles the column's icon mode, the two auto
  toggles flip their column flags, and `Interface Options…` opens the Interface
  preferences pane

#### Scenario: The toggle still toggles [pc_col_header_toggle]

- **WHEN** the column's width toggle is clicked
- **THEN** the column switches between normal and icon mode, no column drag
  starts, and no header menu opens

### Requirement: Floating group top bar and icon mode

A floating panel group SHALL carry its own top bar with a normal/icon width
toggle and a close control, the toggle placed immediately to the left of the
close control on the bar's right side, and the bar SHALL drag the overlay. The
toggle SHALL collapse the group to its icon row and expand it again, and the
close control SHALL remain reachable while collapsed. While collapsed the overlay
SHALL snap to the height its icon row needs, at least the icon-row minimum, and
the icon row SHALL render as the docked icon strip's group box with the same
container styling, grip divider, and icon size. The overlay SHALL offer a resize
grip and keep a minimum size like a docked column. When the floating group holds
a single visible panel, dragging its tab SHALL move the whole overlay rather than
tear off a second overlay and leave a ghost. While a dragged overlay is over a
valid drop target it SHALL be dimmed, returning to full opacity when the target
is invalid or the drag ends.

#### Scenario: The float top bar toggles and closes [pc_float_header]

- **WHEN** a group floats
- **THEN** its top bar shows a collapse toggle immediately left of a close
  control on the right, toggling collapses and expands the group, and dragging
  the bar moves the overlay

#### Scenario: The floating icon overlay snaps to the icon row [pc_float_icon_min]

- **WHEN** a floating group is collapsed to icons
- **THEN** the overlay height snaps to the icon-row height (at least the
  icon-row minimum), the icon row renders as a grouped panel-icon box, and the
  close control remains reachable

#### Scenario: The floating overlay is resizable [pc_float_resize]

- **WHEN** a floating group is shown
- **THEN** it carries a resize grip and a minimum size comparable to a docked
  column

#### Scenario: A single-panel float moves as a whole [pc_float_single_drag]

- **WHEN** the tab of a floating group that holds one visible panel is dragged
- **THEN** the whole overlay moves and no second overlay or empty source overlay
  is left behind

#### Scenario: A dragged overlay dims over a target [pc_float_dim]

- **WHEN** an overlay is dragged over a valid drop target
- **THEN** it is dimmed, and it returns to full opacity when the target is
  invalid or the drag ends

### Requirement: Widget presentation parity

A widget SHALL use the same colour scheme and style whether it is docked, shown
in a compact flyout, or floating. The three presentations SHALL share the same
scoped stylesheet selectors and the same container chrome (tab bar, background,
borders) rather than per-presentation colour literals. Within a group header, the
reserved drag area, the corner container, and the corner action button SHALL
share one header background, and the corner action button SHALL be vertically
centred in the header row. A collapsed group's icon row SHALL use the docked icon
strip's group-box chrome.

#### Scenario: The flyout and float match the docked widget [m44_popupstyle]

- **WHEN** the same panel is presented docked, in a compact flyout, and in a
  float
- **THEN** all three use the same background, borders, and tab-bar styling

#### Scenario: The corner button matches the header [m44_cornerstyle]

- **WHEN** a group header is shown
- **THEN** the reserved drag area, the corner container, and the corner action
  button share one background and the button is vertically centred
