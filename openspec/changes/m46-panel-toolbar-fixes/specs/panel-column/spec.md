## MODIFIED Requirements

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

### Requirement: Panel column minimum width and no clipping

Every widget column in `normal` mode SHALL enforce a single shared minimum width
floor, equal for all widget columns, so no column can be resized below it or
disappear. The central splitter and the per-column group splitter SHALL NOT be
child-collapsible: dragging a handle SHALL clamp at a pane's minimum instead of
collapsing that pane to zero. The column SHALL NOT clip or hide its content on
the right: when the available width is less than the content needs, the column
SHALL elide its tab text, scroll horizontally, and keep the header corner action
button inside the header rather than cutting either off. The iconic strip SHALL
keep its own narrow minimum separate from the shared normal-mode floor.

#### Scenario: All widget columns share one minimum width [m45_min_width_floor]

- **WHEN** two or more widget columns are present and their minimum widths are
  queried
- **THEN** every normal-mode column reports the same shared floor and none can be
  resized below it or vanish

#### Scenario: A splitter pane cannot be collapsed to zero [m46_no_collapse]

- **WHEN** the central splitter or a column's group splitter handle is dragged
  past a pane's minimum
- **THEN** the pane is clamped at its minimum and remains visible

#### Scenario: The right side is never clipped [m45_no_clip]

- **WHEN** a widget column is at its minimum width
- **THEN** its tab text elides, its corner action button is inside the header,
  and any content wider than the column is reachable by scrolling rather than
  cut off
