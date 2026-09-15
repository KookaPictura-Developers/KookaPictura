## ADDED Requirements

### Requirement: SVG cursor per tool
The system SHALL provide an original SVG cursor for each implemented tool under
`assets/cursors/`, named by the tool's `tool.<tool>` id, and SHALL expose
`QCursor cursor(const QString& id)` that resolves it to a non-null cursor.

#### Scenario: A known cursor resolves
- **WHEN** `cursor(id)` is called for a bundled tool cursor id
- **THEN** it returns a non-null `QCursor`

#### Scenario: An unknown cursor is null
- **WHEN** `cursor(id)` is called for an id with no asset
- **THEN** it returns a default/null cursor without crashing

### Requirement: Cursor hotspots
Each SVG cursor SHALL define an action hotspot, and the cursor art SHALL place
its action point at that hotspot. The eye-dropper tip SHALL be at the lower-left;
the remaining tools SHALL use the cursor centre.

#### Scenario: Hotspot matches the artwork
- **WHEN** a tool cursor is created
- **THEN** the hotspot is at the tool's defined action point (the eye-dropper at lower-left, others centred)

### Requirement: Active tool applies its cursor
The system SHALL set the canvas cursor to the active tool's SVG cursor whenever
the active tool changes or a canvas is bound to the controller.

#### Scenario: Switching tools changes the cursor
- **WHEN** the active tool changes
- **THEN** the canvas cursor becomes that tool's SVG cursor
