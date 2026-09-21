# Docked Tools width pin and workspace-edge mark side

## Why

Two follow-ups on the Tools column and the drag edge mark:

- The docked Tools column's separator handle is disabled, but its width was
  still a free splitter size between a minimum and an unbounded maximum, so a
  stray layout pass or any resize that does not go through the handle can still
  change it. The fixed-width contract should be enforced by the widget's own
  size range.
- A bare workspace-edge drag drew its line on an arbitrary column's edge. The
  outer left edge is commonly the document tabs, not a `PanelColumn`, so
  dragging the Tools column to the leftmost side drew the line on the right-hand
  column instead of at the left edge.

## What Changes

- `PanelColumn::updateMinimumWidth` pins the docked Tools column to its content
  width (maximum equal to minimum) in addition to the disabled handle.
- `PanelColumn::updateColumnDrag` marks a bare workspace edge at the central
  area's own left/right edge (`showWorkspaceEdgeIndicator`), owned by the
  outermost column so the line and the resolved target stay in one column,
  instead of at that column's own edge.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `application-shell`: the docked Tools width range is pinned, not only the
  handle disabled.
- `panel-column`: a bare workspace-edge mark is drawn at the workspace edge on
  the requested side.

## Impact

- **C++ app**: `panels/panel_column.cpp`, `panels/panel_column.h`,
  `panels/panel_column_float.cpp`, `panels/panel_column_indicator.cpp`, and the
  round-4 shell self-test.
- **No document-format change, no new dependency, no `docs/` edit.**
