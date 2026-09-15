# tool-framework Specification

## Purpose
TBD - created by archiving change m18-toolbox-tools. Update Purpose after archive.
## Requirements
### Requirement: Tool registry and active tool
The system SHALL define a fixed set of tools identified by a stable `ToolId`
(Move, Marquee, Lasso, Quick Selection, Crop, Eyedropper, Hand, Zoom), SHALL
track exactly one active tool, and SHALL select a tool from a single-character
keyboard shortcut.

#### Scenario: Activating a tool by shortcut
- **WHEN** the user presses a tool's letter shortcut
- **THEN** that tool becomes active and the Tools panel shows it as selected

#### Scenario: One active tool
- **WHEN** a tool is activated
- **THEN** no other tool remains active

### Requirement: Tools panel
The system SHALL present the tools in a dockable Tools panel with one button per
tool, showing the active tool distinctly, and SHALL expose the panel through the
`Window` menu.

#### Scenario: Tools panel reflects the active tool
- **WHEN** a tool becomes active by shortcut or by clicking its button
- **THEN** the Tools panel marks that tool as active

### Requirement: Options bar
The system SHALL present an options bar whose controls change with the active
tool, and SHALL allow the options bar to be shown or hidden. Each selection tool
SHALL expose its combine mode; Quick Selection SHALL expose a tolerance.

#### Scenario: Options follow the active tool
- **WHEN** the active tool changes
- **THEN** the options bar shows that tool's controls

#### Scenario: Options bar toggle
- **WHEN** the user toggles the options bar
- **THEN** it is shown or hidden without changing the active tool

### Requirement: Canvas pointer routing to the active tool
The system SHALL route canvas pointer press, move, and release events to the
active tool with image-space coordinates. When the Hand tool is active the canvas
SHALL pan on left-drag as before; other tools SHALL receive the events.

#### Scenario: Hand tool pans
- **WHEN** the Hand tool is active and the user left-drags the canvas
- **THEN** the canvas pans

#### Scenario: A drawing tool receives the drag
- **WHEN** a selection or crop tool is active and the user presses on the canvas
- **THEN** that tool begins its drag and is given the pressed image coordinates

### Requirement: Tool status and cursor hints
The system SHALL show a hint for the active tool in the status bar and SHALL set
a tool-appropriate cursor while the tool is active.

#### Scenario: Status hint reflects the tool
- **WHEN** a tool becomes active
- **THEN** the status bar hint names the active tool

