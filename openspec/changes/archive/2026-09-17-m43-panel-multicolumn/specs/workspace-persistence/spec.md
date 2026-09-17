## MODIFIED Requirements

### Requirement: Session state store

The system SHALL persist the opaque dock layout, the interface brightness level,
and the panel-column state to `$XDG_STATE_HOME/kooka-pictura/state.json`, writing
atomically through a temporary file so a failed write never leaves a torn store.
The store SHALL carry a schema version, and at schema version 5 SHALL add
`panelRailMode`, `railWidth`, `autoCollapseIconic`, `autoShowHidden`, and the
per-group collapsed, minimized, order, and visibility state. At schema version 6
the system SHALL persist the per-column layout (each column's side and order
with its groups' order, visibility, minimized, and collapsed state nested per
column). A store that is missing a field or older than the current version SHALL
load the defaults, a store with no per-column layout SHALL load a single
right-hand column built from the legacy per-group state, and unknown keys SHALL
survive a load-then-write cycle.

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

#### Scenario: The per-column layout round-trips [m43_session]

- **WHEN** a multi-column layout is saved and the store is reloaded
- **THEN** each column's side and order plus its groups' order, visibility,
  minimized state, and collapsed state are restored

#### Scenario: A version-5 store loads a single right-hand column [m43_session]

- **WHEN** a schema-5 store with a top-level per-group state and no per-column
  layout is loaded
- **THEN** the workspace shows one right-hand column containing those groups and
  the existing keys are preserved
