# history-panel Specification

## Purpose
TBD - created by archiving change m20-panels. Update Purpose after archive.
## Requirements
### Requirement: Labeled history states
The system SHALL record a label with every captured history state and SHALL list
the states oldest-first with their labels in the History panel, highlighting the
current state. States after the current one (redoable) SHALL be shown as not
current.

#### Scenario: States are listed with labels
- **WHEN** several edits have been made
- **THEN** the panel lists one labeled entry per state in order, with the current state highlighted

#### Scenario: A new edit truncates redo states
- **WHEN** a new edit is made after undoing
- **THEN** the panel no longer lists the previously undone states

### Requirement: Jump to a history state
The system SHALL restore the document and selection to the state selected in the
History panel.

#### Scenario: Jump backwards
- **WHEN** the user selects an earlier state in the panel
- **THEN** the document and selection match that state and it becomes current

#### Scenario: Jump forwards
- **WHEN** the user selects a later state
- **THEN** the document and selection match that state

### Requirement: History snapshots
The system SHALL let the user capture the current state as a named snapshot and
SHALL list snapshots separately from the linear states; selecting a snapshot
SHALL restore it.

#### Scenario: Capture a snapshot
- **WHEN** the user captures a snapshot with a name
- **THEN** it appears in the panel and can be selected later to restore that state

### Requirement: History panel docking and toggle
The system SHALL host the History panel in a registered dock with a stable
`objectName` and SHALL expose a `Window > Panels > History` toggle.

#### Scenario: Toggle the History panel
- **WHEN** the user toggles History from the Window menu
- **THEN** the panel is shown or hidden

