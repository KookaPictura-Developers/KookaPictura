## MODIFIED Requirements

### Requirement: Options bar

The system SHALL present an options bar whose controls change with the active
tool, and SHALL allow the options bar to be shown or hidden. Each selection tool
SHALL expose its combine mode; Quick Selection SHALL expose a tolerance. Brush
and Pencil SHALL expose size, hardness, opacity, flow, and paint mode; Pencil
SHALL additionally expose Auto Erase. The options bar's numeric controls SHALL
use the shared numeric field control, so each offers a scrubbing label and a
slider popup, and a change to one control SHALL stay in sync with any other
control bound to the same tool value.

#### Scenario: Options follow the active tool

- **WHEN** the active tool changes
- **THEN** the options bar shows that tool's controls

#### Scenario: Options bar toggle

- **WHEN** the user toggles the options bar
- **THEN** it is shown or hidden without changing the active tool

#### Scenario: A numeric option can be scrubbed and popup-edited [ltf_options_numeric]

- **WHEN** the user interacts with a numeric options-bar control such as the
  brush size or a selection tolerance
- **THEN** it offers a scrubbing label and a slider popup and applies the change
  to the next operation

### Requirement: Tool status and cursor hints
The system SHALL show a hint for the active tool in the status bar and SHALL set
a tool-appropriate cursor while the tool is active. When the active tool would
edit pixels and the target layer is pixel-locked, the cursor SHALL indicate that
the action is not allowed, and a refusal SHALL be reported to the user rather
than the action silently doing nothing.

#### Scenario: Status hint reflects the tool
- **WHEN** a tool becomes active
- **THEN** the status bar hint names the active tool

#### Scenario: A locked target marks the cursor [ltf_locked_cursor]
- **WHEN** a pixel-editing tool is active over a pixel-locked layer
- **THEN** the canvas cursor indicates the action is not allowed and an attempted
  edit reports a refusal
