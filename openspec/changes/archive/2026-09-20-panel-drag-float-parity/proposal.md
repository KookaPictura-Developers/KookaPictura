## Why

The Tools panel still cannot be hosted beside an arbitrary widget column: with a
custom title bar, Qt's own dock drag competes with the custom gesture, so the
panel ends up at an outer dock edge instead of the resolved splitter boundary.
The column-header drag has the same fragile gesture. Several panel-chrome
details also fall short of the docked presentation: the floating group's toggle
placement, icon grouping, height, resize affordance, single-panel tear-off, drag
opacity, the group context-menu contents, and the corner button styling.

## What Changes

- Drive the Tools title-bar drag entirely from the custom gesture (consume the
  mouse events so Qt's dock drag cannot re-dock it), host it at the resolved
  splitter boundary, and float it only when the release resolves no target.
- Allow a widget column to be dragged to the workspace's outermost left/right
  position, including left of the Tools panel, and show the blue insertion line.
- Add a left-click menu on the column header: `Collapse to Icons`,
  `Auto-Collapse Iconic Panels`, `Auto-show Hidden Panels`, `Interface Options…`.
- Floating group: mode toggle immediately left of the close control; a compact
  top in icon mode; grouped icon rendering matching the docked icon strip;
  resize grip with docked-like minimums; a single-panel float dragged by its tab
  moves the whole overlay instead of leaving a ghost; and reduced opacity while
  hovering a valid drop target.
- Group context-menu button: match the header background and centre it, and add
  `Close` (active tab) and `Close Group`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `panel-column`: column placement at the workspace extreme, the column header
  context menu, the floating-group bar/icon/resize/opacity behaviour, and the
  per-widget menu contents.
- `tool-framework` (implementation): the Tools panel is placed through the
  custom title-bar gesture at the resolved boundary.

## Impact

- **C++ app**: `toolbox.{h,cpp}`, `frame_build.cpp`, `frame_columns.cpp`,
  `frame.h`, `panels/panel_column*.{h,cpp}`, `panels/panel_group*.{h,cpp}`,
  `panels/panel_float.cpp`, `theme.cpp`, `selftest_shell_round3.cpp`.
- **No document-format change, no new dependency, no `docs/` edit.**
