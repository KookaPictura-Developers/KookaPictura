## 1. Column-float infrastructure

- [x] 1.1 Add `panels/panel_column_float.cpp` and implement whole-column
  tear-off into an in-window `PanelFloat` overlay: follow the cursor, minimum
  width, resize grip, commit on release, and re-place on a valid target.
- [x] 1.2 Drive the float through the existing `PanelColumn` drag lifecycle and
  the single insertion indicator; do not add a second drag system.
- [x] 1.3 Register the new translation unit in `CMakeLists.txt`.
- [x] 1.4 Add a self-test (next free code 420, new `selftest_*.cpp`):
  a whole column floats in-window, keeps its minimum width, shows a resize grip,
  and re-places on release.

## 2. Tools column

- [x] 2.1 Represent the Tools panel as a `PanelColumn` variant with a column
  header and the tool grid as one plain content child, no `PanelGroup`, no tab
  bar.
- [x] 2.2 Retire the `Toolbox` `QDockWidget` and `toolsDock_`; host the tools
  column in the central splitter as a sibling column.
- [x] 2.3 Add the toggle-mode seam (rail mode vs tool-width mode): the header
  toggle reflows the tool grid one↔two columns and the tools column reports no
  iconic mode.
- [x] 2.4 Route the header drag through the column drag; an outer-band release
  commits a column, and a no-target release floats an in-window overlay.
- [x] 2.5 Session migration: drop the legacy tools dock state from the opaque
  layout and record the tools column, seeding it from the stored tool-column
  count; a store with no tools column loads the default left column (design
  label v6→v7; take the next free store version).
- [x] 2.6 Add self-tests: the tools column is tabless and atomic, floats
  in-window, commits an outer-band release as a column, and round-trips through
  the session.

## 3. Floats as drop targets

- [x] 3.1 Resolve a dragged panel or group against an in-window float in the
  drop resolver and show the shared insertion indicator.
- [x] 3.2 Commit a panel drop as a tab of the float's group and a group drop as
  a merge.
- [x] 3.3 Add a self-test: a dragged panel and a dragged group both drop into an
  existing in-window float.

## 4. Group tabify and blue outline

- [x] 4.1 Merge a group dropped on another group into the target group as tabs.
- [x] 4.2 Add the outline indicator widget and draw a blue region outline around
  the resolved target group, cleared on drag leave, cancel, and commit.
- [x] 4.3 Add a self-test: group-on-group tabify merges the panels and the blue
  outline follows the resolved target and clears on cancel.

## 5. Full-width header background

- [x] 5.1 Extend the panel-group header background behind the right-hand
  per-widget context-menu corner button; remove any corner gap or separate
  corner colour.
- [x] 5.2 Add a self-test: the header background spans the group width behind the
  corner button.

## 6. Whole-drag dim

- [x] 6.1 Dim a dragged panel, group, or column from drag start to drag end, not
  only while over a valid target.
- [x] 6.2 Clear the dim and leave no ghost overlay on cancel.
- [x] 6.3 Add a self-test: whole-drag dim holds over an invalid target, and a
  cancelled drag leaves no ghost.

## 7. Floating-icon parity

- [x] 7.1 Make a floating compact/icon row click open the same popup flyout as
  the docked strip with the clicked panel current.
- [x] 7.2 Make the floating group grip drag the whole group through the docked
  strip's drag grammar.
- [x] 7.3 Add a self-test: a floating icon click opens the flyout and the
  floating grip drags the group.

## 8. Verification

- [x] 8.1 `cmake --build build --parallel` and
  `./build/pictura --headless --self-test`.
- [x] 8.2 `bash scripts/verify-full.sh`.
- [x] 8.3 `openspec validate tools-panel-as-column --strict` and
  `openspec validate --all --strict`.
