## Why

A docked or pane-hosted Tools panel kept its content-derived minimum/fixed
height. When it was dropped beside a widget column, its tall content minimum was
pushed onto the central splitter, so the main window's minimum height grew and
the workspace (document area) was squeezed. Only a floating panel should pin the
content height.

## What Changes

- `Toolbox` pins the content height only while floating; docked and pane-hosted
  it keeps a free height (min 0, max unbounded).
- Release the dock layout's size constraint when not floating so its content
  minimum cannot be re-imposed on the dock, and override `minimumSizeHint()` to a
  free height when not floating.
- Recompute the height contract on float/re-dock (`topLevelChanged`).
- Update the sizing check to the new contract and add a regression check that a
  pane-hosted toolbar does not shrink the workspace.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `application-shell`: the Tools panel's height is pinned only while floating;
  docked/pane-hosted it never forces the workspace shorter.

## Impact

- **C++ app**: `toolbox.{h,cpp}`, `selftest.cpp`, `selftest_shell_round3.cpp`.
- **No document-format change, no new dependency, no `docs/` edit.**
