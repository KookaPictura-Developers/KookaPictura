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
permanently. The strip width SHALL be a stored value and SHALL be user-resizable.
The strip icon button and pixmap sizes SHALL be larger than the M42 sizes. The
flyout placement SHALL be derived from the actual button geometry and SHALL
clamp only the inner-side coordinate, so the flyout can never cross to the outer
side of the column. The compact strip SHALL show each group's icon buttons as
one visual unit, and SHALL expose a small drag-handle affordance above each
group that drags the whole group. The divider between compact groups SHALL be
dark grey, not white. Strip labels SHALL elide and appear as soon as any room
exists, rather than only once the full label width is available. A drop onto or
very close above/below a compact group SHALL insert into that group at that
place, and a drop between groups, above the top group, or below the bottom group
SHALL create a new group at that boundary.

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

#### Scenario: The popup, docked, and floating presentations have no parity differences [m45_popup_group]

- **WHEN** the same group is presented docked, in a compact popup, and floating
- **THEN** all three present the same `PanelGroup` with the same tabs, styling,
  and behaviour

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
- **THEN** it presents the group's tab bar and content and a close affordance

#### Scenario: The flyout closes on click-away [m41_iconic]

- **WHEN** a popup flyout is open and the user clicks outside it
- **THEN** the flyout closes

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
right; a drop into another column's group SHALL draw the line in that target
column; a drop as the rightmost tab SHALL draw the line at the rightmost tab
position and not at the far left; a drop at the bottom boundary SHALL draw the
line at the bottom of the last visible group; and a whole-group drag in compact
mode SHALL draw the line above the group's drag-handle dots, not inside the
group below them. The same single indicator SHALL mark any-side docking targets
beside the Tools toolbar, beside another widget panel or column, and beside the
workspace.

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

#### Scenario: A cross-column drop shows the indicator in the target column [m45_indicator_cross_column]

- **WHEN** a tab is dragged from one widget column into a group of another
  widget column
- **THEN** the blue line is drawn in the target column at the resolved insertion
  point, and the committed drop places the tab there

#### Scenario: The rightmost-tab drop draws the line at its own index [m45_indicator_rightmost_tab]

- **WHEN** a tab is dropped as the rightmost tab of a group
- **THEN** the blue line is drawn at that tab's insertion position and not at the
  far left of the tab bar

#### Scenario: The bottom-boundary drop draws the line at the bottom [m45_indicator_bottom]

- **WHEN** a group or panel is dragged to the bottom boundary of the column
- **THEN** the blue line is drawn at the bottom of the last visible group

#### Scenario: The compact group drag draws the line above the dots [m45_compact_group_line]

- **WHEN** a whole group is dragged in compact mode and the target is a boundary
  before a group
- **THEN** the blue line is drawn above that group's drag-handle dots rather
  than inside the group below them

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
dynamic column SHALL leave no empty column behind.

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

## ADDED Requirements

### Requirement: Panel column minimum width and no clipping

Every widget column in `normal` mode SHALL enforce a single shared minimum width
floor, equal for all widget columns, so no column can be resized below it or
disappear. The column SHALL NOT clip or hide its content on the right: when the
available width is less than the content needs, the column SHALL elide its tab
text, scroll horizontally, and keep the header corner action button inside the
header rather than cutting either off. The iconic strip SHALL keep its own
narrow minimum separate from the shared normal-mode floor.

#### Scenario: All widget columns share one minimum width [m45_min_width_floor]

- **WHEN** two or more widget columns are present and their minimum widths are
  queried
- **THEN** every normal-mode column reports the same shared floor and none can be
  resized below it or vanish

#### Scenario: The right side is never clipped [m45_no_clip]

- **WHEN** a widget column is at its minimum width
- **THEN** its tab text elides, its corner action button is inside the header,
  and any content wider than the column is reachable by scrolling rather than
  cut off
