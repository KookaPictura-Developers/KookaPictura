## Why

Five panel-chrome affordances that the specs already promise (or that CS6 users
expect) were missing or broken:

- the Tools panel could only be dropped at the workspace's outer edges, not on
  either side of an arbitrary widget column;
- the widget column's top header (where the normal/icon toggle lives) was inert,
  so a column could not be dragged and placed from it;
- a group with many tabs showed scroll arrows instead of squeezing the tabs;
- a group had no reserved drag area at the right of its tab bar, so a full or
  overflowing tab bar left nowhere to grab;
- a floating group had no top bar (no mode toggle, close buried in the tab
  corner) and its collapse-to-icons state stranded a small icon row in a large
  empty box with no way to move, expand, or close it.

## What Changes

- Resolve a Tools-panel drop over the whole central area: the outer workspace
  band and a nearest-column fallback, so the panel can land on either side of any
  widget column wherever it is docked.
- Make the column header a drag handle that moves the whole column beside another
  column or to a workspace edge, reusing the single insertion indicator.
- Turn off tab-bar scroll buttons so overflow compresses and elides the tabs.
- Reserve a small blank drag grip at the right of the tab bar that drags the
  whole group, present even when the bar is full.
- Give a floating group a top bar with a normal/icon toggle and the close
  control, draggable, and give its icon mode a minimum height and the panel
  surface styling so it reads like the docked iconic representation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `panel-column`: new requirements for whole-column header drag, tab-overflow
  compression, a reserved group drag grip, and the floating group's top bar and
  icon mode.

The Tools-panel placement is an implementation fix against the existing
`tool-framework`/`application-shell` contract ("placeable on any side of any
widget panel or column, wherever the columns are docked"), so no requirement text
changes there; new regression coverage locks it.

## Impact

- **C++ app**: `frame.{h}`, `frame_columns.cpp`, `frame_test.cpp`,
  `panels/panel_column.{h,cpp}`, `panels/panel_column_drag.cpp`,
  `panels/panel_column_test.cpp`, `panels/panel_group.{h,cpp}`,
  `panels/panel_group_menu.cpp`, `panels/panel_group_test.cpp`,
  `panels/panel_float.cpp`, `theme.cpp`, `selftest_shell_round3.cpp`.
- **No document-format change, no new dependency, no `docs/` edit.**
