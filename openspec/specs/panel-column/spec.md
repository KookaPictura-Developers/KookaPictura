# panel-column Specification

## Purpose
TBD - created by archiving change m41-panel-column. Update Purpose after archive.
## Requirements
### Requirement: Panel column host

The system SHALL host the right-hand panels in a custom `PanelColumn` widget
instead of the `QDockWidget` area, laid out as a vertical stack of `PanelGroup`s
whose heights the user can drag with splitter handles. In `normal` mode the
column SHALL enforce a sensible minimum width so it cannot be squeezed to
nothing. The column SHALL impose no hard panel minimum that forces the main
window taller, and when the available height is less than the groups' size hints
the column SHALL scroll rather than grow the window. Panel resize and scroll
state SHALL survive a relayout. The workspace SHALL support an ordered set of
`PanelColumn`s: zero or more on the left of the document tab area, the document
tabs, and zero or more on the right. A `PanelColumn` SHALL be creatable
dynamically as the result of a drop and SHALL be removed whenever it holds no
groups, on every path that can empty it — a committed drop, a group close, a
panel hide, a flyout restore that takes the last panel, or a group removal — not
only on a committed drop; the document tabs SHALL keep the splitter stretch. A
column's side SHALL be determined by its order in the layout relative to the
document tabs, not by its on-screen geometry. A whole widget column SHALL be
dockable to any side of the Tools toolbar, of another widget panel or column,
and of the workspace, not only to the far right, and SHALL use the existing drop
grammar and insertion indicator rather than a second drag system.

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

#### Scenario: A new column is created on a side drop [m43_newcolumn]

- **WHEN** a floating group is dropped beside a column, beside a group at the
  workspace edge, or beside the Tools toolbar
- **THEN** a new `PanelColumn` is created on that side of the workspace and the
  group is placed in it

#### Scenario: An emptied column is removed [m43_newcolumn]

- **WHEN** the last group leaves a dynamically created column
- **THEN** the column is removed from the workspace and the document tabs keep
  the stretch

#### Scenario: A column emptied by any path is removed [m45_empty_column_removed]

- **WHEN** the last widget of a dynamically created column leaves it through a
  committed drop, a group close, or a panel hide
- **THEN** the column is removed from the workspace on every one of those paths,
  not only on a committed drop, and the document tabs keep the stretch

#### Scenario: A whole widget column docks to any side [m44_docksides]

- **WHEN** a whole widget column is dragged to a side of the Tools toolbar, of
  another widget panel or column, or of the workspace
- **THEN** a target on that side is resolved through the existing drop grammar
  and the column is placed there

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
size. Clicking a panel icon SHALL open a frameless `Qt::Popup` flyout that hosts
the **same `PanelGroup`** as the docked group: the group's full tab set, with
the clicked panel set current, styled and behaving exactly as when docked or
floating, with no parity differences between the three presentations, and the
group SHALL be restored to its column position when the flyout closes. The
flyout SHALL open on the inner side of the column: to the left of the strip when
the column is on the right, and to the right of the strip when the column is on
the left. The icon whose flyout is open SHALL render in the active/pressed
state. The flyout SHALL close on click-away and SHALL NOT steal focus
permanently. The strip width SHALL be fixed while the column is iconic, so a
neighbouring pane's splitter handle drag SHALL NOT resize it. The strip icon
button and pixmap sizes SHALL be larger than the M42 sizes. The flyout placement
SHALL be derived from the actual button geometry and SHALL clamp only the
inner-side coordinate, so the flyout can never cross to the outer side of the
column. The compact strip SHALL show each group's icon buttons as one visual
unit, and SHALL expose a small drag-handle affordance above each group that
drags the whole group. The divider between compact groups SHALL be dark grey,
not white; the group container SHALL use the panel surface shade and the drag
dots SHALL be dark gray. Strip labels SHALL elide and appear as soon as any room
exists, rather than only once the full label width is available. A drop onto or
very close above/below a compact group SHALL insert into that group at that
place; a drop on a group's drag-handle grip SHALL create a new group directly
above that group; and a drop between groups, above the top group, or below the
bottom group SHALL create a new group at that boundary. Pressing and dragging a
compact icon SHALL tear that panel into a floating overlay that follows the
cursor until the mouse button is released and is committed on release, exactly
like a tab or group drag.

#### Scenario: Iconic mode collapses to a strip [m41_iconic]

- **WHEN** the column enters iconic mode
- **THEN** it shows a narrow icon strip with group dividers and does not show
  the full panel content

#### Scenario: The strip icons are larger [m42_iconic]

- **WHEN** the iconic strip is built
- **THEN** its icon buttons and icon pixmaps are larger than the M41 strip

#### Scenario: The popup hosts the whole group with the clicked panel active [m45_popup_group]

- **WHEN** a panel icon is clicked in the iconic strip
- **THEN** the popup hosts the whole `PanelGroup` with all of the group's tabs,
  the clicked panel is current, and the group is restored to its column position
  when the popup closes

#### Scenario: The flyout opens on the inner side [m42_flyout]

- **WHEN** a panel icon is clicked while the column is on the right
- **THEN** the flyout opens to the left of the strip, and when the column is on
  the left it opens to the right of the strip

#### Scenario: The compact icon grows again [m43_icon]

- **WHEN** the iconic strip is built
- **THEN** its icon buttons and icon pixmaps are larger than the M42 sizes

#### Scenario: The flyout uses the actual button geometry [m43_flyout]

- **WHEN** a panel icon's flyout is placed
- **THEN** the flyout's inner edge meets the clicked button's actual edge and the
  flyout is never placed on the outer side of the column

#### Scenario: The open panel's icon is active [m42_iconic]

- **WHEN** a panel icon's flyout is open
- **THEN** that icon renders in the active/pressed state

#### Scenario: The compact flyout matches the docked widget [m44_popupstyle]

- **WHEN** a panel is shown in a compact flyout and the same panel is docked
- **THEN** both present the same tab bar, background, and border styling

#### Scenario: The compact divider is dark grey [m44_compactdivider]

- **WHEN** the compact strip is built with more than one group
- **THEN** the divider between groups is dark grey and not white

#### Scenario: Strip labels elide as soon as there is room [m44_elide]

- **WHEN** the compact strip is made slightly wider than the icon button alone
- **THEN** the labels appear and elide to the available width rather than
  staying hidden until the full label width is available

#### Scenario: A compact drop onto a group inserts into it [m44_compactdrop]

- **WHEN** a dragged icon is dropped onto a compact group, or just above or
  below it
- **THEN** the panel is inserted into that group at that place

#### Scenario: A compact drop between or beyond groups creates a group [m44_compactdrop]

- **WHEN** a dragged icon is dropped between two compact groups, above the top
  group, or below the bottom group
- **THEN** a new group is created at that boundary containing the panel

#### Scenario: A group drag handle drags the whole group [m44_draghandle]

- **WHEN** the compact drag-handle above a group's icons is dragged
- **THEN** the whole group is dragged, not a single icon

#### Scenario: A compact icon drag follows the cursor [m47_compact_icon_float]

- **WHEN** a compact strip icon is pressed and dragged away from the strip
- **THEN** a floating overlay appears and follows the cursor until release, and
  the release commits the drop

#### Scenario: A compact panel drop on the grip creates a group above [m47_compact_grip_group]

- **WHEN** a single compact panel is dropped on a group's drag-handle grip
- **THEN** a new one-panel group is created directly above that group

#### Scenario: An iconic column is not resized by its neighbour [m47_iconic_fixed_width]

- **WHEN** a normal column and an iconic column are adjacent and the splitter
  handle between them is dragged
- **THEN** the iconic column keeps its strip width and only the normal column
  resizes

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
content hidden, and a minimized group SHALL occupy only the height its tab bar
needs rather than the expanded height it had. `Collapse to Icons` SHALL reduce
the group to its iconic representation. The two actions SHALL be distinct and
SHALL persist their state per group in the session. The group's tab menu entry
SHALL read `Expand Panel` while the group is minimized and `Minimize` otherwise,
so the label names the action it performs.

#### Scenario: Minimize hides the content [m41_minimize]

- **WHEN** `Minimize` is chosen for a group
- **THEN** the group shows only its tab bar and its content is hidden

#### Scenario: Minimize collapses the group to its tab bar [m45_minimize_collapse]

- **WHEN** a group is minimized and the layout is forced
- **THEN** the group's own height is the tab-bar height and it does not keep the
  expanded height

#### Scenario: The menu reads Expand Panel while minimized [m45_minimize_collapse]

- **WHEN** a group is minimized and its tab menu is shown
- **THEN** the minimize entry reads `Expand Panel`, and a non-minimized group's
  entry reads `Minimize`

#### Scenario: Collapse to icons is distinct from minimize [m41_iconic]

- **WHEN** `Collapse to Icons` is chosen for a group
- **THEN** the group is represented iconically rather than rolled up to its tab
  bar

#### Scenario: The minimized state persists [m41_session]

- **WHEN** a group is minimized, the session is saved, and the store is reloaded
- **THEN** the group is still minimized

### Requirement: Panel drag and drop rules

The system SHALL support dragging a panel tab and dragging a group, and SHALL
distinguish them for the whole drag. Dragging a tab SHALL drag only that panel;
dragging the empty header area to the right of the tabs SHALL drag the whole
group. Dragging a tab within its own group SHALL reorder it; dragging a tab onto
another group's tab bar SHALL move the panel into that group at the target index;
dragging onto the column background or between groups SHALL insert a new group at
that boundary; and dragging past the workspace edge SHALL tear the dragged panel
or group off into an in-window floating overlay rather than an operating-system
window. A single-panel drag SHALL float a one-panel float, and a group drag SHALL
float the group. The same tab-versus-header distinction SHALL hold for a floating
group. A floating overlay SHALL be re-dockable into a column, into another group,
or as a new column by dropping it on the target. When a tab is dragged outward to
float, the drag SHALL continue until the mouse button is released: the float
SHALL keep following the cursor, exactly like a group or column drag, and SHALL
only be committed or re-docked on release. A whole widget column SHALL be
draggable to any side of the Tools toolbar, of another widget panel or column,
and of the workspace. Every drop that moves the last panel or group out of a
widget column SHALL leave no empty column behind, and this SHALL apply to the
primary column as well as to drop-created columns: when the last visible panel is
removed or moved out, the column SHALL be torn down, and a later drop or
`Window`-menu show SHALL recreate a host column rather than leaving a bare strip.

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

#### Scenario: A single-tab drag floats only that panel [m43_tabdrag]

- **WHEN** a tab title is dragged out of the column and torn off
- **THEN** a one-panel float is created for that panel only, and the source group
  keeps its other panels

#### Scenario: A header drag floats the whole group [m43_tabdrag]

- **WHEN** the empty header area to the right of the tabs is dragged out and torn
  off
- **THEN** the whole group floats, containing all of its panels

#### Scenario: A floating group separates a tab drag from a header drag [m43_tabdrag]

- **WHEN** a floating group's tab is dragged
- **THEN** only that panel moves, and when its empty header is dragged the whole
  group moves

#### Scenario: A floated tab keeps following until release [m44_floatdrag]

- **WHEN** a tab is dragged outward until a float is created and the mouse keeps
  moving before the button is released
- **THEN** the float keeps following the cursor, and it is committed or re-docked
  only on release

#### Scenario: The primary column is removed when its last panel closes [m46_primary_empty]

- **WHEN** the primary widget column's last visible panel is closed or moved out
- **THEN** the column is removed, and a later show or drop recreates a host
  column instead of leaving a bare empty strip

### Requirement: Drop insertion indicator

During any panel drag the target column or group SHALL draw a thick blue
insertion line at the candidate drop position: a boundary between groups for a
column/group drop, the target tab index for a tab/regroup drop, or a full-height
line at the workspace edge for a new-column drop. The indicator SHALL be drawn
by the drop target under the pointer and SHALL disappear when the drag leaves or
is cancelled. The M41/M42 indicator look SHALL be kept. The indicator SHALL be
positioned from the same resolved `DropTarget` the commit uses, in the column
that owns that target, so the line and the drop can never disagree: a drop on
the right of a group's tab bar SHALL draw the line at that tab index on the
right, including when the group has hidden tabs so the index maps past them; a
drop into another column's group, group body, or group boundary SHALL draw the
line in that target column; a drop as the rightmost tab SHALL draw the line at
the rightmost tab position and not at the far left; a drop at the bottom boundary
SHALL draw the line at the bottom of the last visible group, clamped inside the
scroll viewport so the line is never entirely clipped; and a whole-group drag in
compact mode SHALL draw the line above the group's drag-handle dots, not inside
the group below them. The column that owns a new-column line SHALL be the column
adjacent to the workspace edge the new column will occupy, not the drag source.
The same single indicator SHALL mark any-side docking targets beside the Tools
toolbar, beside another widget panel or column, and beside the workspace.

#### Scenario: The blue line marks the target boundary [m41_drop]

- **WHEN** a drag hovers over the column between two groups
- **THEN** a thick blue insertion line is drawn at that boundary

#### Scenario: The indicator clears on cancel [m41_drop]

- **WHEN** a drag is cancelled or leaves the column
- **THEN** no insertion line is drawn

#### Scenario: The indicator marks a new-column target [m43_newcolumn]

- **WHEN** a drag hovers beyond the workspace edge on a side
- **THEN** the blue indicator marks the new-column position on that side

#### Scenario: The indicator marks an any-side dock target [m44_docksides]

- **WHEN** a whole widget column is dragged near a side of the Tools toolbar,
  another widget panel or column, or the workspace
- **THEN** the single blue indicator marks the target position on that side

#### Scenario: The indicator side matches the drop [m45_indicator_side]

- **WHEN** a tab is dragged onto the right side of a group's tab bar
- **THEN** the blue line is drawn on the right at the resolved insertion index,
  and the committed drop lands at that same index

#### Scenario: A group with hidden tabs still draws the right line [m46_indicator_hidden_tab]

- **WHEN** a tab is dragged onto the right side of a group that contains a hidden
  tab
- **THEN** the blue line is drawn at the right insertion position and the drop
  lands at that same index

#### Scenario: A cross-column drop shows the indicator in the target column [m45_indicator_cross_column]

- **WHEN** a tab is dragged from one widget column onto another widget column's
  tab bar, group body, or group boundary
- **THEN** the blue line is drawn in the target column at the resolved insertion
  point, and the committed drop places the tab there

#### Scenario: The rightmost-tab drop draws the line at its own index [m45_indicator_rightmost_tab]

- **WHEN** a tab is dropped as the rightmost tab of a group
- **THEN** the blue line is drawn at that tab's insertion position and not at the
  far left of the tab bar

#### Scenario: The bottom-boundary drop draws the line at the bottom [m45_indicator_bottom]

- **WHEN** a group or panel is dragged to the bottom boundary of the column
- **THEN** the blue line is drawn inside the scroll viewport at the bottom of the
  last visible group, not clipped below it

#### Scenario: The compact group drag draws the line above the dots [m45_compact_group_line]

- **WHEN** a whole group is dragged in compact mode and the target is a boundary
  before a group
- **THEN** the blue line is drawn above that group's drag-handle dots rather
  than inside the group below them

### Requirement: Panel column session state

The session store SHALL advance to schema version 6 and SHALL persist the
per-column layout, where each column records its side, its order, and its
groups' order, visibility, minimized state, and collapsed state (the version-5
per-group shape nested per column). The store SHALL also persist
`panelRailMode`, `railWidth`, `autoCollapseIconic`, and `autoShowHidden`. A
store that is missing a field or older than version 6 SHALL load the defaults,
where a store with no per-column layout SHALL load a single right-hand column
built from the legacy per-group state, and the load-then-write path SHALL
preserve unknown keys and the version-4 `toolsColumns` and
`useShiftKeyForToolSwitch` values.

#### Scenario: Panel column state round-trips [m41_session]

- **WHEN** the mode, strip width, and per-group state are changed, the session
  is saved, and the store is reloaded
- **THEN** the changed values are restored

#### Scenario: An older store loads the defaults [m41_session]

- **WHEN** a schema-4 store with none of the v5 fields is loaded
- **THEN** the mode is `normal`, `autoCollapseIconic` and `autoShowHidden` are
  off, and the existing keys are unchanged

#### Scenario: The multi-column layout round-trips [m43_session]

- **WHEN** a left column and a right column with different groups are arranged,
  the session is saved, and the store is reloaded
- **THEN** each column's side, order, and group state are restored

#### Scenario: A version-5 store loads a single right-hand column [m43_session]

- **WHEN** a schema-5 store with a top-level per-group state but no per-column
  layout is loaded
- **THEN** the workspace shows one right-hand column containing those groups and
  the existing keys are preserved

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

A group torn off a column SHALL float in an in-window overlay that follows the
cursor and re-docks on release, and the overlay SHALL be able to cross and be
placed around the docked or pane-hosted Tools panel without being clamped at the
central widget's edge. The overlay SHALL show a close control at the rightmost
side of its header; closing SHALL hide the group's panels while keeping the
group restorable from `Window > Panels`, then remove the overlay.

#### Scenario: A torn-off group floats and re-docks [m41_tearoff]

- **WHEN** a group is dragged out of a column and dropped back on a column
- **THEN** it floats in an in-window overlay and re-docks

#### Scenario: A floating group can be closed [m47_float_close]

- **WHEN** a floating group's header close control is activated
- **THEN** the overlay is removed, its panels are hidden, and the group is still
  present in a column so `Window > Panels` can restore it

#### Scenario: The overlay can cross the Tools panel [m47_float_over_tools]

- **WHEN** a group is dragged toward the docked Tools panel
- **THEN** the overlay follows the cursor over the Tools panel instead of
  stopping at the central area edge

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

### Requirement: Multi-column drop targets

The drop resolver SHALL be extended, not duplicated, to resolve drops against
the ordered column set: a drop left or right of an existing column, of a group
at the workspace edge, or of the Tools toolbar SHALL resolve to a new column on
that side; a whole widget column SHALL resolve to a new-column or boundary target
on any side of the Tools toolbar, another widget panel or column, and the
workspace; a drop inside an existing widget group SHALL resolve to a tab insert
into that group, including in compact mode; and a drop above or below an
existing group SHALL resolve to a boundary insert, including in compact mode. A
cross-column result SHALL carry the column that owns the target, so the
insertion indicator and the commit both act on that column: a tab insert, a
group-body insert, and a boundary insert in another column SHALL each name the
target column. In compact mode a drop onto a group or within a small band just
above or below it SHALL resolve to an insert into that group, and a drop between
groups or beyond the top/bottom group SHALL resolve to a new group at that
boundary. The resolver SHALL cover both normal and iconic modes and both floating
and non-floating drags, and SHALL use one insertion indicator rather than a
second indicator system.

#### Scenario: Dropping left of a column creates a left column [m43_newcolumn]

- **WHEN** a dragged group is dropped to the left of an existing column
- **THEN** a new column is created on the left and the group is placed in it

#### Scenario: Dropping right of a column creates a right column [m43_newcolumn]

- **WHEN** a dragged group is dropped to the right of an existing column
- **THEN** a new column is created on the right and the group is placed in it

#### Scenario: Dropping beside the Tools toolbar creates a column [m43_newcolumn]

- **WHEN** a dragged group is dropped beside the Tools toolbar
- **THEN** a new column is created on that side and the group is placed in it

#### Scenario: Dropping inside a group inserts a tab [m43_intogroup]

- **WHEN** a dragged panel is dropped inside an existing widget group
- **THEN** the panel becomes a tab of that group

#### Scenario: Dropping inside a compact group inserts a tab [m43_intogroup]

- **WHEN** the workspace is in compact mode and a dragged panel is dropped
  inside an existing group
- **THEN** the panel becomes a tab of that group

#### Scenario: Dropping above or below a group inserts at the boundary [m43_boundary]

- **WHEN** a dragged item is dropped above or below an existing group
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: Dropping above or below a compact group inserts at the boundary [m43_boundary]

- **WHEN** the workspace is in compact mode and a dragged item is dropped above
  or below an existing group
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: A whole widget column resolves to any-side targets [m44_docksides]

- **WHEN** a whole widget column is dragged to a side of the Tools toolbar,
  another widget panel or column, or the workspace
- **THEN** the resolver returns a target on that side and the column is docked
  there through the existing commit path

#### Scenario: A compact group-relative drop is resolved [m44_compactdrop]

- **WHEN** the workspace is in compact mode and an icon is dropped onto a group
  or just above/below it, or between/beyond groups
- **THEN** the resolver returns an into-group target for the former and a new
  boundary group for the latter

#### Scenario: A cross-column target names its owning column [m45_indicator_cross_column]

- **WHEN** a drag resolves to a tab, group-body, or boundary target inside
  another widget column
- **THEN** the resolver result names that target column, and the indicator is
  drawn and the drop committed in it

### Requirement: Panel tab bar active colour

The panel-group tab bar SHALL carry an `objectName` (`panelTabBar`) and SHALL be
styled by a selector scoped to that name. The active tab's background SHALL match
the panel/widget background (the `QTabWidget::pane` `${base}` colour), and an
inactive tab's background SHALL NOT match it (using `${window}` with `${hover}`
on hover). The direction of this mapping is a correctness requirement: the
previous implementation painted the active tab with the inactive `${window}`
colour and vice versa, and that inversion SHALL be corrected rather than
reintroduced. The document tab bar SHALL be unaffected unless its styling is
deliberately changed.

#### Scenario: The active tab matches the pane background [m43_tabcolors]

- **WHEN** a panel group's active tab is inspected
- **THEN** its background is the pane `${base}` colour

#### Scenario: An inactive tab does not match the pane background [m43_tabcolors]

- **WHEN** a panel group's inactive tab is inspected
- **THEN** its background is the `${window}` colour and differs from the pane
  `${base}` colour

#### Scenario: The document tabs are unaffected [m43_tabcolors]

- **WHEN** the panel tab styling is applied
- **THEN** the document tab bar keeps its existing styling

#### Scenario: The active and inactive colours are not swapped [m44_tabswap]

- **WHEN** a panel group's active tab and an inactive tab are compared
- **THEN** the active tab's background matches the pane background and the
  inactive tab's does not, with neither using the other's colour

### Requirement: Corner action button visibility

Each panel group's header SHALL always show the full tab bar and the corner
context-action button together, including at the column's minimum width. Tab
text SHALL elide rather than push the corner button out, the header SHALL reserve
the corner button's width, and the column's minimum-width calculation SHALL
account for the corner button. The corner button SHALL NOT be clipped or pushed
outside the group header at the minimum width.

#### Scenario: The corner button stays visible at minimum width [m43_corner]

- **WHEN** the column is at its minimum width
- **THEN** the corner context-action button is fully inside the group's
  tab-header row and is not clipped

#### Scenario: Tab text elides instead of hiding the corner button [m43_corner]

- **WHEN** the group's available tab-bar width is less than the tab text needs
- **THEN** the tab text elides and the corner button remains visible

### Requirement: Panel widget theme and borders

Panels and columns SHALL draw a darker grey border from the shared theme: the
Tools toolbar, and both the normal and the compact widget panels. The divider
between two widget groups SHALL be thicker and darker grey than the default
splitter handle. The column collapse toggle's double-chevron icon SHALL match the
action it performs: the collapse-to-icons icon in `normal` mode and the expand
icon in `iconic` mode, with the two icons not inverted. Border and divider
dimensions SHALL be single named constants rather than per-site literals.

#### Scenario: Panels draw the darker grey border [m44_panelborder]

- **WHEN** the Tools toolbar and a normal-mode widget panel are shown
- **THEN** each draws a darker grey border from the shared theme

#### Scenario: The group divider is thicker and darker [m44_divider]

- **WHEN** two widget groups are stacked in a column
- **THEN** the handle between them is thicker and darker grey than the platform
  default

#### Scenario: The collapse toggle icon matches its action [m44_chevrons]

- **WHEN** the column collapse toggle is inspected in `normal` and in `iconic`
  mode
- **THEN** each mode shows the icon for the action it performs and the two icons
  are not inverted

### Requirement: Widget presentation parity

A widget SHALL use the same colour scheme and style whether it is docked, shown
in a compact flyout, or floating. The three presentations SHALL share the same
scoped stylesheet selectors and the same container chrome (tab bar, background,
borders) rather than per-presentation colour literals.

#### Scenario: The flyout and float match the docked widget [m44_popupstyle]

- **WHEN** the same panel is presented docked, in a compact flyout, and in a
  float
- **THEN** all three use the same background, borders, and tab-bar styling

### Requirement: Default active panel is first

On a fresh session the first panel SHALL be the active one, not the last, in
whichever artefact carries the default active state — the default active tab of
a group, the default current panel of a column, or the default current page. The
first tab of each default group SHALL be current, and the column's current panel
SHALL be the first visible panel.

#### Scenario: The first panel is active by default [m44_defaultactive]

- **WHEN** the frame starts with a fresh session
- **THEN** the first tab of each default group is current and the first visible
  panel is the column's current panel

### Requirement: Panel column minimum width and no clipping

Every widget column in `normal` mode SHALL enforce a single shared minimum width
floor, equal for all widget columns, large enough that the column's content is
always fully visible horizontally, so no column can be resized below it or
disappear and no column ever displays a horizontal scrollbar or hides content on
its right edge. The floor SHALL be derived from the column content's minimum
size plus the scroll chrome, capped at a sane maximum, and SHALL be applied to
the column so a wider column — not internal scrolling — is what keeps content
visible. The column SHALL elide its tab text and keep the header corner action
button inside the header rather than cutting either off. The iconic strip SHALL
keep its own narrow, fixed minimum separate from the shared normal-mode floor.

#### Scenario: All widget columns share one minimum width [m45_min_width_floor]

- **WHEN** two or more widget columns are present and their minimum widths are
  queried
- **THEN** every normal-mode column reports the same shared floor and none can be
  resized below it or vanish

#### Scenario: A column never horizontally scrolls or clips its content [m47_no_hscroll]

- **WHEN** a widget column is at its minimum width
- **THEN** its horizontal scrollbar policy is off, its content fits within the
  viewport, and no right-side content is hidden

### Requirement: Empty columns and ghost groups are cleaned up

A column that has no group with visible content SHALL be removed (dynamic) or
hidden (primary host) after a drop, even while it owns a live floating overlay;
before removal its floats SHALL be re-homed to the primary column so they stay
re-dockable. A group whose tabs are all hidden SHALL be hidden rather than left
as a visible but un-grabbable shell, and SHALL remain restorable from
`Window > Panels`.

#### Scenario: The source column disappears after its last group floats [m47_empty_after_float]

- **WHEN** the last group of a dynamic column is dragged out and left floating
- **THEN** the empty source column is removed and the float remains re-dockable

#### Scenario: A group with no visible tabs is hidden [m47_ghost_group]

- **WHEN** the last visible tab of a group is moved elsewhere
- **THEN** the emptied group is hidden instead of showing an empty tab bar, and
  its hidden panels can be re-shown from `Window > Panels`

