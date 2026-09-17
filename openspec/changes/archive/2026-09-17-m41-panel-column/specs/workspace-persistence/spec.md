## MODIFIED Requirements

### Requirement: Panels registered as named docks

The system SHALL host each right-hand panel as a plain `QWidget` content widget
with a unique, stable `objectName`, registered in the frame's panel registry
before any layout is saved or restored. The Tools panel and the options bar SHALL
remain `QDockWidget`/toolbar docks outside the column. Registration SHALL reject
a duplicate `objectName`.

#### Scenario: Panel registration assigns a stable name

- **WHEN** a panel content widget is created
- **THEN** it has a unique `objectName` used by layout and registry persistence

#### Scenario: Duplicate names are rejected

- **WHEN** two panels are registered with the same `objectName`
- **THEN** registration fails rather than silently corrupting layout restore

#### Scenario: The Tools panel stays a dock

- **WHEN** the frame is built
- **THEN** the Tools panel remains a `QDockWidget` and the column panels do not

### Requirement: Session state store

The system SHALL persist the opaque dock layout, the interface brightness level,
and the panel-column state to `$XDG_STATE_HOME/kooka-pictura/state.json`, writing
atomically through a temporary file so a failed write never leaves a torn store.
The store SHALL carry a schema version, and at schema version 5 SHALL add
`panelRailMode`, `railWidth`, `autoCollapseIconic`, `autoShowHidden`, and the
per-group collapsed, minimized, order, and visibility state. A store that is
missing a field or older than version 5 SHALL load the defaults, and unknown keys
SHALL survive a load-then-write cycle.

#### Scenario: Session state is restored on launch

- **WHEN** the application starts and a valid session state exists
- **THEN** the saved layout, brightness, and panel-column state are applied

#### Scenario: Missing session state

- **WHEN** no session state file exists
- **THEN** the application starts with the default layout and default brightness

#### Scenario: Corrupt session state

- **WHEN** the session state file cannot be parsed
- **THEN** the application starts with defaults and does not crash

#### Scenario: Atomic write

- **WHEN** the session state is saved
- **THEN** the store is written to a temporary file and renamed over the previous store

#### Scenario: An older store loads the panel-column defaults [m41_session]

- **WHEN** a schema-4 store with none of the version-5 fields is loaded
- **THEN** the panel-column mode is `normal`, auto-collapse and auto-show are
  off, and the existing keys are preserved
