## ADDED Requirements

### Requirement: Panels registered as named docks
The system SHALL host each panel in a `QDockWidget` registered with a unique,
stable `objectName` before any layout is saved or restored. Registration SHALL
reject a duplicate `objectName`.

#### Scenario: Panel registration assigns a stable name
- **WHEN** a panel dock is created
- **THEN** it has a unique `objectName` used by layout persistence

#### Scenario: Duplicate names are rejected
- **WHEN** two docks are registered with the same `objectName`
- **THEN** registration fails rather than silently corrupting layout restore

### Requirement: Layout persistence across restart
The system SHALL save the dock and toolbar layout with
`QMainWindow::saveState()` and restore it with `restoreState()` on startup, so
the frame arrangement survives a restart at the same scale factor.

#### Scenario: Layout round-trips
- **WHEN** the layout is changed, saved, and the application restarts
- **THEN** the dock arrangement is restored

#### Scenario: Unknown panel in saved layout
- **WHEN** a saved layout references a panel that no longer exists
- **THEN** restore ignores the unknown entry and keeps the remaining arrangement

### Requirement: Window panels toggle and hide-all
The system SHALL expose a `Window > Panels` entry that toggles each panel's
visibility, and SHALL hide and restore all panels with `Tab`. `Shift+Tab` SHALL
hide all panels except the Tools panel and options bar; until those exist it
SHALL behave as `Tab`.

#### Scenario: Toggle a panel from the Window menu
- **WHEN** the user selects a panel from `Window > Panels`
- **THEN** that panel's visibility toggles without producing a history state

#### Scenario: Tab hides all panels
- **WHEN** the user presses `Tab`
- **THEN** all panels are hidden, and pressing `Tab` again restores them

### Requirement: Session state store
The system SHALL persist the opaque dock layout and the interface brightness
level to `$XDG_STATE_HOME/kooka-pictura/state.json`, writing atomically through a
temporary file so a failed write never leaves a torn store. The store SHALL carry
a schema version.

#### Scenario: Session state is restored on launch
- **WHEN** the application starts and a valid session state exists
- **THEN** the saved layout and brightness are applied

#### Scenario: Missing session state
- **WHEN** no session state file exists
- **THEN** the application starts with the default layout and default brightness

#### Scenario: Corrupt session state
- **WHEN** the session state file cannot be parsed
- **THEN** the application starts with defaults and does not crash

#### Scenario: Atomic write
- **WHEN** the session state is saved
- **THEN** the store is written to a temporary file and renamed over the previous store
