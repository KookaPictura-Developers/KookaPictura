## MODIFIED Requirements

### Requirement: Column width toggle

The top of the column SHALL carry one thin width-toggle control with
`objectName` `panelColumnToggle` that switches the whole column between `normal`
and `iconic` modes, using the same visual language as the Tools panel's
`toolsColumnToggle`. Switching to `iconic` SHALL set the column to the smallest
width that shows the icon strip rather than keeping the prior splitter width,
and switching back to `normal` SHALL restore the normal-mode width. The control
SHALL replace the M24 `PanelRail` far-right toolbar. The normal-mode width SHALL
be recorded when the column leaves normal mode and SHALL be seeded from the
session's stored `railWidth` when the column is constructed, so a column that
starts in or is left in iconic mode still restores its real normal-mode width.

#### Scenario: The toggle switches modes [m41_iconic]

- **WHEN** the column width toggle is activated
- **THEN** the column switches from `normal` to `iconic` and back, and the
  control's state reflects the current mode

#### Scenario: Switching to iconic uses the smallest width [m42_iconic]

- **WHEN** the column is wide in `normal` mode and the toggle switches it to
  `iconic`
- **THEN** the column collapses to the smallest width that shows the icon strip
  and does not keep the prior splitter width

#### Scenario: The toggle replaces the rail [m41_rail]

- **WHEN** the frame is built
- **THEN** there is no `PanelRail` and no far-right panel rail toolbar

#### Scenario: The normal width survives an iconic round-trip and restart [lpr_iconic_width]

- **WHEN** a column is set to a normal width, switched to iconic, and the
  application is restarted while it is iconic
- **THEN** switching it back to normal restores the stored normal-mode width
  measured after the first layout

### Requirement: Panel column session state

The session store SHALL advance to schema version 7 and SHALL persist the
per-column layout, where each column records its side, its order, its width, and
its groups' order, visibility, minimized state, and collapsed state (the
version-5 per-group shape nested per column). The store SHALL also persist
`panelRailMode`, `railWidth`, `autoCollapseIconic`, and `autoShowHidden`. A
store that is missing a field or older than version 7 SHALL load the defaults,
where a store with no per-column layout SHALL load a single right-hand column
built from the legacy per-group state, a version-6 per-column entry with no
width SHALL load the default width (seeded from the legacy `railWidth` for the
primary column), and the load-then-write path SHALL preserve unknown keys and
the version-4 `toolsColumns` and `useShiftKeyForToolSwitch` values.

#### Scenario: Panel column state round-trips [m41_session]

- **WHEN** the mode, strip width, and per-group state are changed, the session
  is saved, and the store is reloaded
- **THEN** the changed values are restored

#### Scenario: An older store loads the defaults [m41_session]

- **WHEN** a schema-4 store with none of the v5 fields is loaded
- **THEN** the mode is `normal`, `autoCollapseIconic` and `autoShowHidden` are
  off, and the existing keys are unchanged

#### Scenario: The multi-column layout round-trips [m43_session]

- **WHEN** a left column and a right column with different groups are arranged,
  the session is saved, and the store is reloaded
- **THEN** each column's side, order, width, and group state are restored

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
