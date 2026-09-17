## ADDED Requirements

### Requirement: Central splitter keeps every pane

The application shell SHALL keep every pane of its central splitter — the
ordered set of left widget columns, the document tab area, and right widget
columns — at or above its minimum size, and SHALL NOT allow a handle drag to
collapse a pane to zero. The splitter SHALL clamp a handle drag at the pane's
minimum size, and a widget column SHALL remain visible at its minimum floor
instead of disappearing. The per-column group splitter SHALL enforce the same
non-collapsible invariant for its panel groups.

#### Scenario: Dragging the central splitter handle cannot hide a pane [m46_center_splitter]

- **WHEN** the central splitter handle is dragged past a widget column's minimum
  width
- **THEN** the column is clamped at its minimum and stays visible rather than
  collapsing to zero

#### Scenario: The group splitter cannot collapse a group to zero [m46_no_collapse]

- **WHEN** a column's group splitter handle is dragged past a panel group's
  minimum height
- **THEN** the group is clamped at its minimum and its tab bar stays visible
