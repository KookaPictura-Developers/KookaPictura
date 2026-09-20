## Why

Suppressing Qt's own `QDockWidget` title-bar drag (the fix that made placement
beside widget columns deterministic) removed the other two placements: the Tools
panel could no longer be docked to the left/right workspace edge as a normal
dock, and it could no longer float. The resolver treated the workspace outer band
as a splitter-pane target, so an edge release always became a pane.

## What Changes

- A release in the workspace outer band docks the Tools panel to the left or
  right dock area instead of hosting it as a splitter pane.
- A release outside the main window floats the panel at the cursor.
- A title-bar double-click toggles the panel between floating and its dock.
- Pane placement beside or among widget columns is unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tool-framework`: the title-bar gesture's three placement outcomes (pane,
  edge dock, float) and the double-click float toggle.

## Impact

- **C++ app**: `toolbox.{h,cpp}`, `frame_build.cpp`, `frame_columns.cpp`,
  `frame.h`, `frame_test.cpp`, `selftest_shell_round3.cpp`.
- **No document-format change, no new dependency, no `docs/` edit.**
