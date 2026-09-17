## MODIFIED Requirements

### Requirement: Panel column host

The system SHALL host the right-hand panels in a custom `PanelColumn` widget
instead of the `QDockWidget` area, laid out as a vertical stack of `PanelGroup`s
whose heights the user can drag with splitter handles. In `normal` mode the
column SHALL enforce a sensible minimum width so it cannot be squeezed to
nothing. The column SHALL impose no hard panel minimum that forces the main
window taller, and when the available height is less than the groups' size hints
the column SHALL scroll rather than grow the window. Panel resize and scroll
state SHALL survive a relayout.

#### Scenario: The column hosts panel groups [m41_tabs]

- **WHEN** the frame starts with a fresh session
- **THEN** the right-hand area contains a `PanelColumn` of `PanelGroup`s and no
  right-hand panel `QDockWidget`

#### Scenario: A short window scrolls instead of forcing a minimum [m41_minwidth]

- **WHEN** the main window is shrunk below the column's size hint
- **THEN** the window shrinks and the column scrolls, and no panel forces a
  larger height minimum

#### Scenario: Group heights are draggable [m41_tabs]

- **WHEN** the user drags the handle between two groups
- **THEN** the groups' heights change accordingly

#### Scenario: Normal mode enforces a minimum width [m42_minwidth]

- **WHEN** the splitter handle is dragged to squeeze the column in `normal` mode
- **THEN** the column does not shrink below its normal-mode minimum width

### Requirement: Column width toggle

The top of the column SHALL carry one thin width-toggle control with
`objectName` `panelColumnToggle` that switches the whole column between `normal`
and `iconic` modes, using the same visual language as the Tools panel's
`toolsColumnToggle`. Switching to `iconic` SHALL set the column to the smallest
width that shows the icon strip rather than keeping the prior splitter width,
and switching back to `normal` SHALL restore the normal-mode width. The control
SHALL replace the M24 `PanelRail` far-right toolbar.

#### Scenario: The toggle switches modes [m41_iconic]

- **WHEN** the column width toggle is activated
- **THEN** the column switches from `normal` to `iconic` and back, and the
  control's state reflects the current mode

#### Scenario: Switching to iconic uses the smallest width [m42_iconic]

- **WHEN** the column is wide in `normal` mode and the toggle switches it to
  `iconic`
- **THEN** the column collapses to the smallest width that shows the icon strip
  and does not keep the prior splitter width

#### Scenario: The toggle replaces the rail [m41_rail]

- **WHEN** the frame is built
- **THEN** there is no `PanelRail` and no far-right panel rail toolbar

### Requirement: Compact and iconic mode

In `iconic` mode the column SHALL collapse to a narrow vertical strip
containing one icon button per panel with group dividers between groups, and
SHALL show the panel labels when the strip is widened. Each strip icon button
SHALL be larger than the M41 strip and SHALL render its icon at a larger pixmap
size. Clicking a panel icon SHALL open a frameless `Qt::Popup` flyout
containing that panel's content. The flyout SHALL open on the inner side of the
column: to the left of the strip when the column is on the right, and to the
right of the strip when the column is on the left. The icon whose flyout is open
SHALL render in the active/pressed state. The flyout SHALL be styled as a panel
group: a header naming the panel followed by the panel content, with a close
icon button (a double right chevron) at the right end of the header. The flyout
SHALL close on click-away and SHALL NOT steal focus permanently. The strip width
SHALL be a stored value and SHALL be user-resizable.

#### Scenario: Iconic mode collapses to a strip [m41_iconic]

- **WHEN** the column enters iconic mode
- **THEN** it shows a narrow icon strip with group dividers and does not show
  the full panel content

#### Scenario: The strip icons are larger [m42_iconic]

- **WHEN** the iconic strip is built
- **THEN** its icon buttons and icon pixmaps are larger than the M41 strip

#### Scenario: An icon opens a popup flyout [m41_iconic]

- **WHEN** a panel icon in the iconic strip is clicked
- **THEN** a `Qt::Popup` flyout opens with that panel's content

#### Scenario: The flyout opens on the inner side [m42_flyout]

- **WHEN** a panel icon is clicked while the column is on the right
- **THEN** the flyout opens to the left of the strip, and when the column is on
  the left it opens to the right of the strip

#### Scenario: The open panel's icon is active [m42_iconic]

- **WHEN** a panel icon's flyout is open
- **THEN** that icon renders in the active/pressed state

#### Scenario: The flyout is group-styled with a header close button [m42_flyout]

- **WHEN** a panel icon's flyout is shown
- **THEN** it presents a header naming the panel above the panel content and a
  close icon button at the right end of the header

#### Scenario: The flyout closes on click-away [m41_iconic]

- **WHEN** a popup flyout is open and the user clicks outside it
- **THEN** the flyout closes

### Requirement: Panel drag and drop rules

The system SHALL support dragging a panel tab and dragging a group. Dragging a
tab within its own group SHALL reorder it; dragging a tab onto another group's
tab bar SHALL move the panel into that group at the target index; dragging onto
the column background or between groups SHALL insert a new group at that
boundary; and dragging past the column edge SHALL tear the dragged panel or
group off into an in-window floating overlay rather than an operating-system
window. A floating overlay SHALL be re-dockable into the column or into another
group by dropping it on the target.

#### Scenario: A tab reorders within its group [m41_drag]

- **WHEN** a tab is dragged to a different index in the same group
- **THEN** the panel order in that group changes

#### Scenario: A tab regroups [m41_drag]

- **WHEN** a tab is dropped on another group's tab bar
- **THEN** the panel joins that group at the drop index and leaves its old group

#### Scenario: A drop between groups inserts a new group [m41_drag]

- **WHEN** a dragged item is dropped on the column background between two groups
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: A group tears off into an overlay and re-docks [m41_tearoff]

- **WHEN** a group is dragged past the column edge and then dropped back on the
  column
- **THEN** it floats in an in-window overlay and, on the drop, re-docks into the
  column

## ADDED Requirements

### Requirement: Iconic strip drag and reorder

The iconic strip SHALL support dragging a panel icon to reorder it within the
strip, and SHALL support dropping a dragged icon onto the normal-mode group
stack to move the panel into the target group. The strip drag SHALL reuse the
`PanelColumn` drag machinery used by tabs and groups rather than a second drag
system, and a strip reorder SHALL persist through the existing per-group panel
order.

#### Scenario: Dragging a strip icon reorders it [m42_dragstrip]

- **WHEN** a panel icon in the iconic strip is dragged to another position in
  the strip
- **THEN** the strip and the stored panel order reflect the new order

#### Scenario: Dropping a strip icon on the group stack moves the panel [m42_dragstrip]

- **WHEN** a panel icon is dragged from the strip onto a normal-mode group's tab
  bar or the space between groups
- **THEN** the panel moves into that group, and the column returns to displaying
  the group stack

### Requirement: Floating panel overlay

A torn-off panel group SHALL float as a child overlay inside the main window,
not as an operating-system top-level window. The overlay SHALL move with its tab
bar within the main window, SHALL be clipped to the main window's bounds so it
cannot be dragged outside them, SHALL render above the columns and the canvas,
and SHALL re-dock into the `PanelColumn` when dropped back on it. The overlay
SHALL NOT appear as its own window in the window manager or task list.

#### Scenario: A torn-off group is an in-window overlay [m42_float_overlay]

- **WHEN** a group is torn off the column
- **THEN** it floats as a child overlay inside the main window and does not open
  a separate operating-system window

#### Scenario: The overlay is clipped to the main window [m42_float_overlay]

- **WHEN** the floating overlay is dragged toward the main window's edge
- **THEN** it stays within the main window's bounds and is not drawn outside it

#### Scenario: The overlay re-docks on drop [m42_float_overlay]

- **WHEN** the floating overlay is dropped back on the column
- **THEN** the group re-docks into the column and the overlay disappears

### Requirement: Per-widget panel header action menu

Each panel group's tab header SHALL carry a context-action button at its far
right. Activating it SHALL show the menu of the panel that is current in the
group, so the menu is per-widget rather than per-group. Each menu SHALL contain
that panel's researched CS6 entries as recorded in `docs/dev/m42-panel-menus.md`,
in a stable order. An entry that is not implemented SHALL be shown disabled with
the tooltip `<label> — not implemented yet` and SHALL NOT fake behaviour. `Close`
and `Close Panel Group` SHALL NOT appear in the per-widget menu; they remain on
the tab context menu. The Layers panel's `Panel Options…` entry SHALL open the
existing Panel Options behaviour.

#### Scenario: The header button shows the current panel's menu [m42_widgetmenu]

- **WHEN** the tab-header action button is activated for a group whose current
  panel is Layers
- **THEN** the Layers per-widget menu is shown

#### Scenario: Unimplemented entries are disabled with a tooltip [m42_widgetmenu]

- **WHEN** a per-widget menu contains an entry whose feature is not implemented
- **THEN** that entry is disabled and its tooltip is
  `<label> — not implemented yet`

#### Scenario: Close entries stay off the per-widget menu [m42_widgetmenu]

- **WHEN** a per-widget menu is shown
- **THEN** it contains neither `Close` nor `Close Panel Group`

#### Scenario: Layers Panel Options still opens [m42_widgetmenu]

- **WHEN** `Panel Options…` is chosen from the Layers per-widget menu
- **THEN** the existing Layers Panel Options behaviour runs
