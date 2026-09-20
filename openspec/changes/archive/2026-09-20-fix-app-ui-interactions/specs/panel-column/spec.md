## MODIFIED Requirements

### Requirement: Panel column session state

The session store SHALL advance to schema version 8 and SHALL persist the
per-column layout, where each column records its side, its order, its width, its
rail mode (`normal` or `iconic`), and its groups' order, visibility, minimized
state, and collapsed state (the version-5 per-group shape nested per column). The
store SHALL also persist `panelRailMode`, `railWidth`, `autoCollapseIconic`, and
`autoShowHidden` as legacy values. A store that is missing a field or older than
version 8 SHALL load the defaults, where a store with no per-column layout SHALL
load a single right-hand column built from the legacy per-group state, a
version-7 per-column entry with no rail mode SHALL load the mode from the legacy
top-level `panelRailMode` for every column, a version-6 per-column entry with no
width SHALL load the default width (seeded from the legacy `railWidth` for the
primary column), and the load-then-write path SHALL preserve unknown keys and the
version-4 `toolsColumns` and `useShiftKeyForToolSwitch` values. The system SHALL
restore each column's own rail mode, and SHALL NOT apply one column's mode to
every column.

#### Scenario: Panel column state round-trips [m41_session]

- **WHEN** the mode, strip width, and per-group state are changed, the session
  is saved, and the store is reloaded
- **THEN** the changed values are restored

#### Scenario: An older store loads the defaults [m41_session]

- **WHEN** a schema-4 store with none of the v5 fields is loaded
- **THEN** the mode is `normal`, `autoCollapseIconic` and `autoShowHidden` are
  off, and the existing keys are unchanged

#### Scenario: The multi-column layout round-trips [m43_session]

- **WHEN** a left column and a right column with different groups and different
  rail modes are arranged, the session is saved, and the store is reloaded
- **THEN** each column's side, order, width, rail mode, and group state are
  restored

#### Scenario: A version-5 store loads a single right-hand column [m43_session]

- **WHEN** a schema-5 store with a top-level per-group state but no per-column
  layout is loaded
- **THEN** the workspace shows one right-hand column containing those groups and
  the existing keys are preserved

#### Scenario: A version-6 store loads a default width [lpr_v7_width]

- **WHEN** a schema-6 store with per-column entries that carry no width and a
  legacy top-level `railWidth` is loaded
- **THEN** the primary column's width is the legacy `railWidth` and every other
  column loads the default width

#### Scenario: A version-7 store seeds every column's rail mode [lpr_v7_railmode]

- **WHEN** a schema-7 store with per-column widths but no per-column rail mode
  and a legacy top-level `panelRailMode` is loaded
- **THEN** every column loads the legacy `panelRailMode` value

#### Scenario: A non-primary column's mode round-trips [lpr_column_railmode]

- **WHEN** a non-primary column is toggled from `iconic` to `normal`, the session
  is saved, and the store is reloaded
- **THEN** that column restores as `normal` while the other columns keep their
  own modes
