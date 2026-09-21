## MODIFIED Requirements

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
toolbar, beside another widget panel or column, and beside the workspace. The
new-column line SHALL be drawn above any floating overlay that follows the drag,
so an in-window overlay that tracks the cursor cannot hide it, and it SHALL NOT
be drawn on the atomic Tools column, which never shows a widget drop line. For a
bare workspace edge, the mark SHALL be drawn at the central area's own left or
right edge — the side the new column will occupy — not at an arbitrary column's
edge; the outermost column on that side SHALL own the mark so the line and the
resolved target stay in one column.

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

#### Scenario: The new-column mark stays above a following overlay [fp_edge_mark_above_overlay]

- **WHEN** a floating overlay is dragged over a dock spot while still floating
- **THEN** the new-column mark is drawn above the overlay and remains visible,
  instead of being hidden under the overlay that follows the cursor

#### Scenario: The atomic Tools column draws no widget drop line [fp_edge_mark_not_on_tools]

- **WHEN** a widget panel is dragged over the docked Tools column
- **THEN** no new-column mark is drawn on the Tools column

#### Scenario: A bare workspace-edge mark is drawn on the requested side [fp_edge_mark_workspace_side]

- **WHEN** a column is dragged to the leftmost workspace side while the only
  other column is on the right
- **THEN** the mark is drawn at the central area's left edge, not at the
  right-hand column's edge, and it is committed at the splitter head
