## MODIFIED Requirements

### Requirement: Layout persistence across restart
The system SHALL save the dock and toolbar layout with
`QMainWindow::saveState()` and restore it with `restoreState()` on startup, so
the frame arrangement survives a restart at the same scale factor. The session
SHALL also persist each panel column's width, and a stored width SHALL be applied
and measurable after restart. Every quit path — the window close control,
`File > Exit`, `Ctrl+Q`, and session logout — SHALL persist the session, not only
a window-close event that runs `closeEvent`.

#### Scenario: Layout round-trips
- **WHEN** the layout is changed, saved, and the application restarts
- **THEN** the dock arrangement is restored

#### Scenario: Unknown panel in saved layout
- **WHEN** a saved layout references a panel that no longer exists
- **THEN** restore ignores the unknown entry and keeps the remaining arrangement

#### Scenario: A quit path persists the session [lpr_quit_save]
- **WHEN** the application is quit through `File > Exit` or `Ctrl+Q` rather than
  the window close control
- **THEN** the session is saved before exit and the saved state is restored on
  the next launch

#### Scenario: A column width is applied after restart [lpr_width_restart]
- **WHEN** a panel column is resized, the application is quit, and it is
  relaunched
- **THEN** the column's width after launch equals the stored width, measured
  after the first layout, for every column and for a column that was left in
  iconic mode

### Requirement: Session state store

The system SHALL persist the opaque dock layout, the interface brightness level,
and the panel-column state to `$XDG_STATE_HOME/kooka-pictura/state.json`, writing
atomically through a temporary file so a failed write never leaves a torn store.
The store SHALL carry a schema version, and at schema version 5 SHALL add
`panelRailMode`, `railWidth`, `autoCollapseIconic`, `autoShowHidden`, and the
per-group collapsed, minimized, order, and visibility state. At schema version 6
the system SHALL persist the per-column layout (each column's side and order
with its groups' order, visibility, minimized, and collapsed state nested per
column). At schema version 7 the system SHALL add a width to each per-column layout
entry, and SHALL seed a column's width from the legacy top-level `railWidth` when
a loaded store has no per-column width. A store that is missing a field or older
than the current version SHALL load the defaults, a store with no per-column
layout SHALL load a single right-hand column built from the legacy per-group
state, and unknown keys SHALL survive a load-then-write cycle. The store SHALL be
written on every quit path.

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

#### Scenario: A version-6 store loads a default column width [lpr_v7]

- **WHEN** a schema-6 store whose per-column entries carry no width is loaded
- **THEN** each column loads the default width and the legacy top-level
  `railWidth` seeds the primary column's width
