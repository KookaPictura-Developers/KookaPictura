## MODIFIED Requirements

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be hosted as an atomic column — a tabless, ungroupable
`PanelColumn` variant — and SHALL NOT be a `QDockWidget` in the main-window dock
areas. It SHALL support being moved and floated as an in-window overlay and SHALL
NOT be grouped with other panels in a tab group. A drag or drop that would
combine the Tools panel with a widget panel or group SHALL NOT tabify or merge
them. When floated it SHALL size to the minimum its content needs and SHALL
remain an in-window overlay rather than an independent operating-system window.
The panel's column header SHALL remain draggable so the column can be moved and
floated. The panel's content size SHALL be fixed along the column's major axis: a
fixed content width in both one- and two-column modes. Dragging the column
separator SHALL NOT resize it. The panel SHALL be placeable on either side of any
widget panel or column, wherever the columns are placed, through the same column
drop grammar and the same single insertion indicator the widget columns use,
without breaking the fixed-size rule or the no-tabification contract.

#### Scenario: The panel is an atomic column, not a dock [las_tools_column]

- **WHEN** the frame is built
- **THEN** the Tools panel is a tabless `PanelColumn` variant with no tab bar and
  no `PanelGroup`, and no Tools `QDockWidget` exists

#### Scenario: The panel floats in-window [las_tools_inwindow_float]

- **WHEN** the Tools column is dragged out of the workspace
- **THEN** it floats as an in-window overlay and can be re-placed, not as an
  independent operating-system window

#### Scenario: The floating overlay hugs its content height [m42_tools]

- **WHEN** the Tools column floats in-window
- **THEN** its height is the minimum its content needs and it does not expand to
  fill the window

#### Scenario: The floating height cannot be dragged [m44_toolsfloat]

- **WHEN** a resize is attempted on the floating Tools column
- **THEN** it keeps its fixed content height and does not change

#### Scenario: Tabification is refused [m40_dock]

- **WHEN** the Tools column is dragged onto another panel's tab bar
- **THEN** it does not become a tab in that group and no insertion indicator is
  shown for the forbidden combination

#### Scenario: The width cannot be dragged [m43_tools]

- **WHEN** the column separator beside the Tools column is dragged
- **THEN** the Tools column's width does not change and stays at its content
  width

#### Scenario: The panel is placed beside a widget column [m45_tools_beside_column]

- **WHEN** the floating Tools column is dragged to a side of a widget column,
  including the left of a right-hand column or between two columns
- **THEN** the single blue indicator marks that boundary and the panel is placed
  there without being tabified and without losing its fixed content width

### Requirement: Tools column fills its column height

The Tools column SHALL fill its column height while it is hosted in the
workspace, and its content minimum SHALL NOT force the main window's central
workspace shorter than the window. Only the in-window floating overlay SHALL pin
its height to the content it needs; re-placing a floating column into the
workspace SHALL release the pinned height and floating it again SHALL re-pin it.
The floating pinned height SHALL NOT be drag-resizable.

#### Scenario: The column fills its column height [las_tools_free_height]

- **WHEN** the Tools column is hosted in the workspace
- **THEN** the column fills the height it is given, the central splitter keeps
  its height, and the document pane fills the splitter

#### Scenario: Floating pins and re-placing releases [las_tools_float_height]

- **WHEN** the Tools column is floated in-window and then re-placed into the
  workspace
- **THEN** the floating overlay is pinned to its content height and the
  re-placed column is free to fill its column height again

## REMOVED Requirements

### Requirement: Central splitter hosts the Tools pane

**Reason**: The Tools panel is now a first-class `PanelColumn` variant hosted by
the central splitter in its own right, not a special fixed-width pane re-hosted
from a dock. The dock-versus-pane distinction no longer exists.

**Migration**: The Tools column is placed as a sibling column through the widget
column grammar. Its content is a single plain child instead of a pane host, and
the atomic-target rule replaces the pane-specific tabification guard.

## RENAMED Requirements

- FROM: `### Requirement: Tools panel height is pinned only while floating`
- TO: `### Requirement: Tools column fills its column height`
