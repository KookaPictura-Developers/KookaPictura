## ADDED Requirements

### Requirement: Active layer resolution for tool edits

The application SHALL maintain exactly one active layer for the active document
and SHALL resolve it through one shared active-layer resolver used by every
tool-edit entry point (paint, filter, Free Transform, and content move). The
Layers panel SHALL push its selection into the view when the selection changes,
and a non-PSD raster import SHALL make its imported layer active. A tool edit
SHALL be refused, with the document unchanged and a user-visible refusal, when no
layer is active or when more than one layer is selected. No tool-edit entry point
SHALL fall back to the topmost raster layer.

#### Scenario: Exactly one active layer receives the edit [ltf_active_layer]

- **WHEN** a tool edit runs with exactly one layer active
- **THEN** that layer is the edit target and the other layers are unchanged

#### Scenario: The panel pushes its selection [ltf_active_layer_push]

- **WHEN** the user changes the selected layer in the Layers panel
- **THEN** the view's active layer follows the panel selection and the next tool
  edit targets it

#### Scenario: Zero or multiple selection refuses [ltf_active_layer_refuse]

- **WHEN** a tool edit runs with no layer active or with more than one layer
  selected
- **THEN** the edit is refused with a user-visible refusal, the document is
  unchanged, and no history state is added

### Requirement: Active-layer scope of the resolver

The resolver SHALL be the single source of the edit target and SHALL be consulted
by paint, filter, Free Transform, and content move. The resolver SHALL NOT be
used to block structural layer operations, and a position lock SHALL remain the
only reason a move is refused. When the active layer changes, the tool status and
cursor SHALL reflect the new target without a separate per-tool lookup.

#### Scenario: Every tool edit uses the same resolver [ltf_active_layer_shared]

- **WHEN** paint, a filter, Free Transform, and a content move each run with the
  same single active layer
- **THEN** all four target that layer through the same resolver

## MODIFIED Requirements

### Requirement: Tool status and cursor hints

The system SHALL show a context hint for the active tool in the bottom status bar
as a hint bar of bordered keycaps with descriptions, and SHALL set a
tool-appropriate cursor while the tool is active. For Brush and Pencil the cursor
SHALL be blank so the drawn brush-size ring is the pointer affordance. When the
active tool would edit pixels and the target layer is pixel-locked, the cursor
SHALL indicate that the action is not allowed, and a refusal SHALL be reported to
the user rather than the action silently doing nothing.

#### Scenario: Status hint reflects the tool

- **WHEN** a tool becomes active
- **THEN** the status-bar hint bar shows that tool's context keycaps

#### Scenario: Brush uses a blank cursor [ltf_blank_cursor]

- **WHEN** Brush or Pencil is active over the canvas
- **THEN** the canvas cursor is blank and the brush-size ring is drawn

#### Scenario: A locked target marks the cursor [ltf_locked_cursor]

- **WHEN** a pixel-editing tool is active over a pixel-locked layer
- **THEN** the canvas cursor indicates the action is not allowed and an attempted
  edit reports a refusal
