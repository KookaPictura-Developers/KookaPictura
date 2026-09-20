## ADDED Requirements

### Requirement: Tools panel placement outcomes

A left-button drag on the Tools panel's custom title bar SHALL resolve to one of
three placements on release, through the panel's own gesture rather than Qt's
dock drag, so the outcome is deterministic. A release over or beside a widget
column SHALL host the panel as a central-splitter pane at that column's boundary.
A release in the workspace's outer left or right band SHALL dock the panel to the
corresponding left or right dock area. A release outside the main window SHALL
float the panel at the cursor. A title-bar double-click SHALL toggle the panel
between a floating overlay and its current left/right dock. The panel SHALL
remain re-draggable from each state, and the implemented tool set and active-tool
contract SHALL be unchanged.

#### Scenario: The title-bar drag keeps all three placements [ltf_tools_placements]

- **WHEN** the Tools panel's title bar is dragged and released over a widget
  column, in the workspace outer band, or outside the main window
- **THEN** it becomes a splitter pane beside that column, docks to that side, or
  floats at the cursor respectively

#### Scenario: Double-click floats and re-docks the panel [ltf_tools_dblclick]

- **WHEN** the Tools panel's title bar is double-clicked while docked or hosted
  as a splitter pane, then double-clicked again while floating
- **THEN** it floats on the first double-click and returns to a normal left/right
  dock on the second
