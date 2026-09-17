## MODIFIED Requirements

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
dynamically as the result of a drop and SHALL be removed when it holds no groups;
the document tabs SHALL keep the splitter stretch. A column's side SHALL be
determined by its order in the layout relative to the document tabs, not by its
on-screen geometry.

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
SHALL be a stored value and SHALL be user-resizable. The strip icon button and
pixmap sizes SHALL be larger than the M42 sizes. The flyout placement SHALL be
derived from the actual button geometry and SHALL clamp only the inner-side
coordinate, so the flyout can never cross to the outer side of the column.

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

#### Scenario: The flyout is group-styled with a header close button [m42_flyout]

- **WHEN** a panel icon's flyout is shown
- **THEN** it presents a header naming the panel above the panel content and a
  close icon button at the right end of the header

#### Scenario: The flyout closes on click-away [m41_iconic]

- **WHEN** a popup flyout is open and the user clicks outside it
- **THEN** the flyout closes

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
or as a new column by dropping it on the target.

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

### Requirement: Drop insertion indicator

During any panel drag the target column or group SHALL draw a thick blue
insertion line at the candidate drop position: a boundary between groups for a
column/group drop, the target tab index for a tab/regroup drop, or a full-height
line at the workspace edge for a new-column drop. The indicator SHALL be drawn
by the drop target under the pointer and SHALL disappear when the drag leaves or
is cancelled. The M41/M42 indicator look SHALL be kept.

#### Scenario: The blue line marks the target boundary [m41_drop]

- **WHEN** a drag hovers over the column between two groups
- **THEN** a thick blue insertion line is drawn at that boundary

#### Scenario: The indicator clears on cancel [m41_drop]

- **WHEN** a drag is cancelled or leaves the column
- **THEN** no insertion line is drawn

#### Scenario: The indicator marks a new-column target [m43_newcolumn]

- **WHEN** a drag hovers beyond the workspace edge on a side
- **THEN** the blue indicator marks the new-column position on that side

### Requirement: Floating panel overlay

A torn-off panel or group SHALL float as a child overlay inside the main window,
not as an operating-system top-level window. The overlay SHALL move with its tab
bar within the main window, SHALL be clipped to the main window's bounds so it
cannot be dragged outside them, SHALL render above the columns and the canvas,
and SHALL re-dock into the panel column when dropped back on it. A single-panel
float SHALL hold only its one panel and a group float its group; the float's own
tab bar SHALL route a tab drag to a one-panel move and its empty header to a
group move. The overlay SHALL NOT appear as its own window in the window manager
or task list.

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

#### Scenario: A one-panel float carries only its panel [m43_singlefloat]

- **WHEN** a tab drag tears off one panel
- **THEN** the float contains that panel only, and re-docking it restores the
  group with the other panels

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

## ADDED Requirements

### Requirement: Multi-column drop targets

The drop resolver SHALL be extended, not duplicated, to resolve drops against
the ordered column set: a drop left or right of an existing column, of a group
at the workspace edge, or of the Tools toolbar SHALL resolve to a new column on
that side; a drop inside an existing widget group SHALL resolve to a tab insert
into that group, including in compact mode; and a drop above or below an
existing group SHALL resolve to a boundary insert, including in compact mode.
The resolver SHALL cover both normal and iconic modes and both floating and
non-floating drags, and SHALL use one insertion indicator rather than a second
indicator system.

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

### Requirement: Panel tab bar active colour

The panel-group tab bar SHALL carry an `objectName` (`panelTabBar`) and SHALL be
styled by a selector scoped to that name. The active tab's background SHALL match
the panel/widget background (the `QTabWidget::pane` `${base}` colour), and an
inactive tab's background SHALL NOT match it (using `${window}` with `${hover}`
on hover). The document tab bar SHALL be unaffected unless its styling is
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
