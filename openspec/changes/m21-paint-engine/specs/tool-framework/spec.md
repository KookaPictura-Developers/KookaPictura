## MODIFIED Requirements

### Requirement: Tool registry and active tool

The system SHALL define a fixed set of tools identified by a stable `ToolId`
(Move, Marquee, Lasso, Quick Selection, Crop, Eyedropper, Brush, Pencil, Hand,
Zoom), SHALL track exactly one active tool, and SHALL select a tool from a
single-character keyboard shortcut. The Brush and Pencil SHALL share the `B`
shortcut, with a modified press cycling between them.

#### Scenario: Activating a tool by shortcut

- **WHEN** the user presses a tool's letter shortcut
- **THEN** that tool becomes active and the Tools panel shows it as selected

#### Scenario: One active tool

- **WHEN** a tool is activated
- **THEN** no other tool remains active

#### Scenario: Cycling the paint tool slot

- **WHEN** the `B` shortcut is pressed while Brush is active
- **THEN** Pencil becomes active

### Requirement: Options bar

The system SHALL present an options bar whose controls change with the active
tool, and SHALL allow the options bar to be shown or hidden. Each selection tool
SHALL expose its combine mode; Quick Selection SHALL expose a tolerance. Brush
and Pencil SHALL expose size, hardness, opacity, flow, and paint mode; Pencil
SHALL additionally expose Auto Erase.

#### Scenario: Options follow the active tool

- **WHEN** the active tool changes
- **THEN** the options bar shows that tool's controls

#### Scenario: Options bar toggle

- **WHEN** the user toggles the options bar
- **THEN** it is shown or hidden without changing the active tool
