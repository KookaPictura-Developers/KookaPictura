## MODIFIED Requirements

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be hosted as an atomic column — a tabless, ungroupable
`PanelColumn` variant — and SHALL NOT be a `QDockWidget` in the main-window dock
areas. It SHALL support being moved and floated as a frameless `Qt::Tool`
top-level window parented to (transient for) the main window, and SHALL NOT be
grouped with other panels in a tab group. A drag or drop that would combine the
Tools panel with a widget panel or group SHALL NOT tabify or merge them. When
floated it SHALL size to the minimum its content needs, SHALL offer no resize
grip, and SHALL be a frameless `Qt::Tool` top-level window — no title bar, no
window decorations, and no taskbar entry — that may be moved outside the main
window, its movement clamped to the available screen geometry, rather than an
operating-system window with a frame. The panel's column header SHALL remain
draggable so the column can be moved and floated. The panel's content size SHALL
be fixed along the column's major axis: a fixed content width in both one- and
two-column modes. Dragging the column separator SHALL NOT resize it. The panel
SHALL be placeable on either side of any widget panel or column, wherever the
columns are placed, through the same column drop grammar and the same single
insertion indicator the widget columns use, without breaking the fixed-size rule
or the no-tabification contract.

#### Scenario: The panel is an atomic column, not a dock [las_tools_column]

- **WHEN** the frame is built
- **THEN** the Tools panel is a tabless `PanelColumn` variant with no tab bar and
  no `PanelGroup`, and no Tools `QDockWidget` exists

#### Scenario: The panel floats as a frameless tool window [las_tools_inwindow_float]

- **WHEN** the Tools column is dragged out of the workspace
- **THEN** it floats as a frameless `Qt::Tool` top-level window parented to the
  main window, movable outside the main window and clamped to the screen, and can
  be re-placed

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
