# Tasks

## 1. Docked Tools column fixed width

- [x] 1.1 In `PicturaMainWindow::reapplyColumnStretch`, disable the splitter
  handle(s) adjacent to the Tools column; leave handles between widget panes
  enabled.
- [x] 1.2 Re-apply it in `applyPanelSession` after the Tools column is
  re-inserted, because `insertWidget` creates fresh, enabled handles.

## 2. Floating Tools overlay re-fit on a mode flip

- [x] 2.1 `PanelFloat::syncToContent`: handle a hosted tools column by snapping
  both axes to the content minimum (`minimumWidth` / `minimumSizeHint`), with
  zero minimums so the overlay may shrink.
- [x] 2.2 `PanelColumn::refreshToolsWidth`: when the column is floating, call
  `columnFloat_->syncToContent()` instead of only resizing a splitter pane.

## 3. Edge mark above the overlay

- [x] 3.1 Add a frame-parented `edgeIndicator_` to `PanelColumn` (deleted with
  the column) and a `showColumnEdgeIndicator(bool left)` that draws the
  full-height `#2a7fff` mark at the column edge and raises it.
- [x] 3.2 `showIndicatorFor` routes the new-column kinds to
  `showColumnEdgeIndicator` and hides the edge mark for every other target;
  `clearIndicator` hides it too.
- [x] 3.3 Suppress the mark on the atomic Tools column (`isToolsColumn()`), which
  never shows a widget drop line.

## 4. Coverage

- [x] 4.1 `docked_tools_fixed_width` (441): the Tools column is a splitter pane
  and the handle beside it is disabled.
- [x] 4.2 `floating_tools_toggle_resize` (442): a floating Tools overlay grows
  in width to match the two-column grid on a 1->2 flip.
- [x] 4.3 `float_column_drag_indicator` (443): in forced child (Wayland) mode a
  second drag of an already-floating column resolves the target, the edge mark
  is a frame-level widget, and the release re-docks.
- [x] 4.4 `dropIndicatorVisibleForTest` / `dropIndicatorGlobalGeometryForTest`
  also account for the frame-level edge mark; add `edgeIndicatorOnFrameForTest`.
- [x] 4.5 Existing checks 118, 126, 143, 167-169, 420-424, 433-440 pass
  unchanged.

## 5. Verification

- [x] 5.1 `openspec validate tools-column-fixed-width-and-float-fit --strict` and
  `openspec validate --all --strict`.
- [x] 5.2 `cmake --build build --parallel`.
- [x] 5.3 `QT_QPA_PLATFORM=offscreen ./build/pictura --headless --self-test` —
  zero failures, 378 checks.
- [x] 5.4 `bash scripts/verify-full.sh` — OK.
