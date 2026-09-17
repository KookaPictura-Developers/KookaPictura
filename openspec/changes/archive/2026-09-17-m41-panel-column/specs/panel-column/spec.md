## ADDED Requirements

### Requirement: Panel column host

The system SHALL host the right-hand panels in a custom `PanelColumn` widget
instead of the `QDockWidget` area, laid out as a vertical stack of `PanelGroup`s
whose heights the user can drag with splitter handles. The column SHALL impose
no hard panel minimum that forces the main window wider or taller, and when the
available height is less than the groups' size hints the column SHALL scroll
rather than grow the window. Panel resize and scroll state SHALL survive a
relayout.

#### Scenario: The column hosts panel groups [m41_tabs]

- **WHEN** the frame starts with a fresh session
- **THEN** the right-hand area contains a `PanelColumn` of `PanelGroup`s and no
  right-hand panel `QDockWidget`

#### Scenario: A short window scrolls instead of forcing a minimum [m41_minwidth]

- **WHEN** the main window is shrunk below the column's size hint
- **THEN** the window shrinks and the column scrolls, and no panel forces a
  larger minimum size

#### Scenario: Group heights are draggable [m41_tabs]

- **WHEN** the user drags the handle between two groups
- **THEN** the groups' heights change accordingly

### Requirement: Panel groups and top tabs

Each `PanelGroup` SHALL present its panels in a `QTabWidget` with the tab
position explicitly set to `QTabWidget::North`, so the tabs render on top of the
group on every platform and stylesheet. The tab text SHALL be the panel's title,
there SHALL be no separate group-title label widget, and a group containing a
single panel SHALL still show that panel's tab.

#### Scenario: Tabs are on top [m41_tabs]

- **WHEN** any group is shown
- **THEN** its tab position is `QTabWidget::North`

#### Scenario: The tab is the only group chrome [m41_tabs]

- **WHEN** a group's tab bar is inspected
- **THEN** the group has no title label beyond the tabs, and each tab's text is
  the panel title

#### Scenario: A single-panel group shows its tab [m41_tabs]

- **WHEN** a group contains exactly one panel
- **THEN** that panel's tab is shown

### Requirement: Default panel groups

The system SHALL open with the CS6 Essentials groups: `Color | Swatches |
Styles`; `Adjustments`; `Layers | Channels | Paths`; `Navigator | Histogram |
Info`; and the iconic `History` and `Actions`. `Styles` SHALL occupy the former
Gradients/Patterns tab slot, and the Properties content SHALL fold into
`Adjustments`. The Gradients, Patterns, Properties, and Libraries panels SHALL
remain registered and reachable from `Window > Panels` but SHALL NOT be in a
default visible group.

#### Scenario: Default groups match CS6 Essentials [m41_groups]

- **WHEN** the frame starts with a fresh session
- **THEN** the visible groups are exactly Color/Swatches/Styles, Adjustments,
  Layers/Channels/Paths, Navigator/Histogram/Info, and iconic History and Actions

#### Scenario: A folded panel stays reachable [m41_groups]

- **WHEN** the `Window > Panels > Properties` toggle is invoked
- **THEN** the Properties panel can be shown even though it is not in a default
  visible group

### Requirement: Panel registry and content widgets

Each panel SHALL be a plain `QWidget` content widget with a stable `objectName`
and SHALL be registered in a frame-owned registry mapping the panel to its
title, icon asset id, group id, visibility, iconic state, minimized state, and
order. The `objectName`, `setView`, `refresh`, and existing test hooks of every
panel SHALL be preserved where possible. The `Window > Panels` toggles SHALL
remain the single command path for panel visibility and SHALL reflect and drive
the registry.

#### Scenario: Panels are content widgets [m41_tabs]

- **WHEN** a panel object is queried
- **THEN** it is a `QWidget` content widget with its documented `objectName` and
  is not a `QDockWidget`

#### Scenario: The Window menu drives the registry [m41_groups]

- **WHEN** a `Window > Panels` toggle is invoked
- **THEN** the panel's registry visibility changes and the column reflects it

### Requirement: Column width toggle

The top of the column SHALL carry one thin width-toggle control with
`objectName` `panelColumnToggle` that switches the whole column between `normal`
and `iconic` modes, using the same visual language as the Tools panel's
`toolsColumnToggle`. The control SHALL replace the M24 `PanelRail` far-right
toolbar.

#### Scenario: The toggle switches modes [m41_iconic]

- **WHEN** the column width toggle is activated
- **THEN** the column switches from `normal` to `iconic` and back, and the
  control's state reflects the current mode

#### Scenario: The toggle replaces the rail [m41_rail]

- **WHEN** the frame is built
- **THEN** there is no `PanelRail` and no far-right panel rail toolbar

### Requirement: Compact and iconic mode

In `iconic` mode the column SHALL collapse to a narrow vertical strip
containing one icon button per panel with group dividers between groups, and
SHALL show the panel labels when the strip is widened. Clicking a panel icon
SHALL open a frameless `Qt::Popup` flyout containing that panel's content; the
flyout SHALL close on click-away and SHALL NOT steal focus permanently. The
strip width SHALL be a stored value and SHALL be user-resizable.

#### Scenario: Iconic mode collapses to a strip [m41_iconic]

- **WHEN** the column enters iconic mode
- **THEN** it shows a narrow icon strip with group dividers and does not show
  the full panel content

#### Scenario: An icon opens a popup flyout [m41_iconic]

- **WHEN** a panel icon in the iconic strip is clicked
- **THEN** a `Qt::Popup` flyout opens with that panel's content

#### Scenario: The flyout closes on click-away [m41_iconic]

- **WHEN** a popup flyout is open and the user clicks outside it
- **THEN** the flyout closes

### Requirement: Panel tab context menu

Right-clicking a group's tab bar SHALL show exactly, in order, `Close`,
`Close Panel Group`, `Minimize`, `Collapse to Icons`, `Auto-Collapse Iconic
Panels` (checkable), `Auto-Show Hidden Panels` (checkable), and
`Interface Options…`. The two checkable items SHALL persist in the session.
`Interface Options…` SHALL open the Preferences dialog on its Interface pane.
An action that cannot apply SHALL be disabled rather than shown as an inert
stub.

#### Scenario: The tab menu has exactly the seven items [m41_menu]

- **WHEN** a group's tab bar is right-clicked
- **THEN** the menu contains exactly the seven named items in order

#### Scenario: The checkable items persist [m41_session]

- **WHEN** `Auto-Collapse Iconic Panels` is toggled and the session is saved and
  reloaded
- **THEN** the setting is restored

#### Scenario: Interface Options opens the Interface pane [m41_prefs]

- **WHEN** `Interface Options…` is chosen
- **THEN** the Preferences dialog opens on the Interface pane

### Requirement: Minimize and collapse to icons

`Minimize` SHALL roll a group up so only its tab bar remains visible with the
content hidden. `Collapse to Icons` SHALL reduce the group to its iconic
representation. The two actions SHALL be distinct and SHALL persist their state
per group in the session.

#### Scenario: Minimize hides the content [m41_minimize]

- **WHEN** `Minimize` is chosen for a group
- **THEN** the group shows only its tab bar and its content is hidden

#### Scenario: Collapse to icons is distinct from minimize [m41_iconic]

- **WHEN** `Collapse to Icons` is chosen for a group
- **THEN** the group is represented iconically rather than rolled up to its tab
  bar

#### Scenario: The minimized state persists [m41_session]

- **WHEN** a group is minimized, the session is saved, and the store is reloaded
- **THEN** the group is still minimized

### Requirement: Panel drag and drop rules

The system SHALL support dragging a panel tab and dragging a group. Dragging a
tab within its own group SHALL reorder it; dragging a tab onto another group's
tab bar SHALL move the panel into that group at the target index; dragging onto
the column background or between groups SHALL insert a new group at that
boundary; and dragging past the column edge SHALL tear the dragged panel or
group off into a floating window. A floating group SHALL be re-dockable into the
column or into another group by dropping it on the target.

#### Scenario: A tab reorders within its group [m41_drag]

- **WHEN** a tab is dragged to a different index in the same group
- **THEN** the panel order in that group changes

#### Scenario: A tab regroups [m41_drag]

- **WHEN** a tab is dropped on another group's tab bar
- **THEN** the panel joins that group at the drop index and leaves its old group

#### Scenario: A drop between groups inserts a new group [m41_drag]

- **WHEN** a dragged item is dropped on the column background between two groups
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: A group tears off and re-docks [m41_tearoff]

- **WHEN** a group is dragged past the column edge and then dropped back on the
  column
- **THEN** it floats in its own window and, on the drop, re-docks into the column

### Requirement: Drop insertion indicator

During any panel drag the target column or group SHALL draw a thick blue
insertion line at the candidate drop position: a boundary between groups for a
column/group drop, or the target tab index for a tab/regroup drop. The indicator
SHALL be drawn by the drop target under the pointer and SHALL disappear when the
drag leaves or is cancelled.

#### Scenario: The blue line marks the target boundary [m41_drop]

- **WHEN** a drag hovers over the column between two groups
- **THEN** a thick blue insertion line is drawn at that boundary

#### Scenario: The indicator clears on cancel [m41_drop]

- **WHEN** a drag is cancelled or leaves the column
- **THEN** no insertion line is drawn

### Requirement: Panel column session state

The session store SHALL advance to schema version 5 and SHALL persist
`panelRailMode` (`normal` or `iconic`), `railWidth`, `autoCollapseIconic`,
`autoShowHidden`, and per-group collapsed, minimized, order, and visibility
state. A store that is missing a field or older than version 5 SHALL load the
defaults, and the load-then-write path SHALL preserve unknown keys and the v4
`toolsColumns` and `useShiftKeyForToolSwitch` values.

#### Scenario: Panel column state round-trips [m41_session]

- **WHEN** the mode, strip width, and per-group state are changed, the session
  is saved, and the store is reloaded
- **THEN** the changed values are restored

#### Scenario: An older store loads the defaults [m41_session]

- **WHEN** a schema-4 store with none of the v5 fields is loaded
- **THEN** the mode is `normal`, `autoCollapseIconic` and `autoShowHidden` are
  off, and the existing keys are unchanged

### Requirement: Preferences dialog

The system SHALL provide a Preferences dialog with `General` and `Interface`
panes, reachable from the tab menu's `Interface Options…` and, when the command
tree's `Edit > Preferences > General` leaf is implemented, from
`Edit → Preferences…`. The Interface pane SHALL expose `Use Shift Key For Tool
Switch`, `Auto-Collapse Iconic Panels`, and the column width-toggle default;
applying a change SHALL persist it and apply it to the toolbox and column
without a restart. The remaining CS6 preference panes SHALL stay disabled and
unimplemented.

#### Scenario: The Interface pane carries the preferences [m41_prefs]

- **WHEN** the Preferences dialog is opened on the Interface pane
- **THEN** it shows `Use Shift Key For Tool Switch`, `Auto-Collapse Iconic
  Panels`, and the column width-toggle default

#### Scenario: A preference round-trips and applies [m41_prefs]

- **WHEN** `Use Shift Key For Tool Switch` is toggled off in the dialog and the
  session is saved and reloaded
- **THEN** the toolbox honours the off value and the store reads it back

#### Scenario: Unimplemented panes stay disabled [m41_prefs]

- **WHEN** the Preferences dialog is shown
- **THEN** only the General and Interface panes are enabled
