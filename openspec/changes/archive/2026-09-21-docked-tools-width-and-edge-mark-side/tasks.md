# Tasks

## 1. Docked Tools width pin

- [x] 1.1 `PanelColumn::updateMinimumWidth`: for the tools column, set the maximum
  width equal to the minimum (content width), in addition to the disabled
  separator handle.

## 2. Workspace-edge mark side

- [x] 2.1 Add `PanelColumn::showWorkspaceEdgeIndicator(bool left)` drawing the
  full-height mark at the central widget's left/right edge, on the frame and
  raised like the column-edge mark.
- [x] 2.2 `PanelColumn::updateColumnDrag`: for a bare workspace edge (no resolved
  anchor), mark the workspace edge on the requested side and let the outermost
  column on that side own the mark, instead of drawing at that column's own edge.

## 3. Coverage

- [x] 3.1 `docked_tools_fixed_width` (441) also asserts the tools column's
  minimum width equals its maximum width.
- [x] 3.2 `float_column_drag_indicator` (443) also asserts the left-side mark's
  geometry is at the central area's left edge (a tall, 3 px line), not on the
  right-hand column.
- [x] 3.3 Existing checks 118, 143, 167-169, 420-424, 439 pass unchanged.

## 4. Verification

- [x] 4.1 `openspec validate docked-tools-width-and-edge-mark-side --strict` and
  `openspec validate --all --strict`.
- [x] 4.2 `cmake --build build --parallel`.
- [x] 4.3 `QT_QPA_PLATFORM=offscreen ./build/pictura --headless --self-test` —
  zero failures, 378 checks.
- [x] 4.4 `bash scripts/verify-full.sh` — OK.
