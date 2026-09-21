# Tools column fixed width and floating re-fit

## Why

Three Tools-toolbar defects remain after the column refactor:

- The docked Tools column can be widened by dragging the splitter handle beside
  it. The spec already says the separator drag "SHALL NOT resize it"; the handle
  was simply left enabled, so the contract was not enforced in code.
- A floating Tools overlay does not re-fit when the tool grid flips between one
  and two columns. `PanelFloat::syncToContent` returned early for the tools
  column, and `PanelColumn::refreshToolsWidth` only resized a splitter pane, so a
  two-column grid stayed clipped inside a one-column-wide overlay.
- Dragging a floating column over a dock spot resolved the target but the blue
  new-column mark was drawn inside the target column's viewport. The floating
  overlay follows the cursor above the splitter and hides the line exactly where
  the user is looking.

## What Changes

- `PicturaMainWindow::reapplyColumnStretch` disables the splitter handle(s)
  adjacent to the Tools column, and `applyPanelSession` re-applies that after it
  re-inserts the column (a fresh insert creates enabled handles).
- `PanelColumn::refreshToolsWidth` re-fits a floating tools overlay through
  `PanelFloat::syncToContent`, and `syncToContent` handles the tools column by
  snapping both axes to the content minimum.
- The new-column/workspace-edge mark is drawn by a frame-level widget
  (`edgeIndicator_`) that is raised, so an in-window floating overlay cannot
  cover it; the mark is suppressed on the atomic Tools column, which never shows
  a widget drop line.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `application-shell`: the standalone Tools column's fixed-width rule is
  enforced by disabling the separator handle, and a 1<->2 column flip re-fits a
  floating Tools overlay to its new content size.
- `panel-column`: the new-column edge mark is drawn above any following
  floating overlay and is never drawn on the atomic Tools column.

## Impact

- **C++ app**: `frame_columns.cpp`, `frame_session.cpp`,
  `panels/panel_column.cpp`, `panels/panel_column.h`,
  `panels/panel_column_indicator.cpp`, `panels/panel_column_test.cpp`,
  `panels/panel_float.cpp`, and the round-4 shell self-test.
- **No document-format change, no new dependency, no `docs/` edit.**
