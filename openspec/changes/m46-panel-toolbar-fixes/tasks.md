## 1. Drop indicator correctness

- [x] 1.1 `PanelGroup::tabInsertionX` maps insertions over visible tabs only, falling back to the nearest visible tab or bar edge (panel_group.cpp). `tabInsertionIndexAt` already skips hidden tabs and stays in full-tab space.
- [x] 1.2 `PanelColumn::resolveDrop` accepts any valid non-outside cross-column delegated target and carries `owner = other` (panel_column.cpp).
- [x] 1.3 `PanelColumn::showIndicatorFor` clamps the boundary/group-body line inside the scroll viewport (panel_column.cpp).
- [x] 1.4 `PanelColumn::resolveDrop` new-column owner is the column adjacent to the target workspace edge, not the drag source (panel_column.cpp).
- [x] 1.5 Cross-column body/boundary commits route through `target.owner` in `applyPanelDrop`/`applyGroupDrop`, so the drop lands in the column that drew the line (found by the new m46 cross-body test).

## 2. Floating Tools gesture and drop grammar

- [x] 2.1 `Toolbox::eventFilter` records the title-bar press and accepts move/release on the dock while floating, so the gesture survives Qt's mouse grab (toolbox.cpp).
- [x] 2.2 `PicturaMainWindow::resolveToolboxDrop` falls back to `columnAtGlobal` + pointer-half side and shows the edge indicator (frame.cpp).

## 3. Panel lifecycle and sizing

- [x] 3.1 `centerSplitter_` and the column group splitter are non-collapsible (frame.cpp, panel_column.cpp).
- [ ] 3.2 DEFERRED: content-derived scroll minimum for right-side no-clip regresses M43 column geometry; needs a group-level horizontal-scroll fix (see design D7).
- [x] 3.3 `PanelGroup::applyMinimize` saves/clamps/restores minimum height as well as maximum (panel_group.cpp / .h).
- [x] 3.4 Empty-column removal applies to the primary column (hidden, not deleted), the empty test is ancestor-independent, and live floats re-test after destruction via `destroyFloat`/`cleanupEmptyGroup` (frame.cpp, panel_column.cpp).
- [x] 3.5 `PanelColumn::showPanel` re-shows a hidden host column when a panel is shown again.

## 4. Compact popup and grip verification

- [x] 4.1 Confirmed the compact-strip popup hosts the whole `PanelGroup` with the clicked panel current (m45_popup_group passes; no change needed).
- [x] 4.2 Confirmed a dragged group's line is drawn above the grip (m45_compact_group_line passes; no change needed).

## 5. Self-tests

- [x] 5.1 Hidden-tab group regression for the right-side/rightmost line (`m46_indicator_hidden_tab`, 180).
- [x] 5.2 Cross-column group-body and boundary indicator and placement (`m46_indicator_cross_body`, 181).
- [x] 5.3 Bottom-boundary indicator asserts geometry inside the viewport (`m46_indicator_bottom_inside`, 182) plus the `scrollViewportHeightForTest` hook.
- [x] 5.4 Real `QMouseEvent` title-bar drag emits after the dock grabs the mouse (`m46_tools_gesture`, 183).
- [x] 5.5 Splitter non-collapsibility and non-vanishing floor (`m46_splitter_nocollapse`, 184).
- [x] 5.6 Minimize a group whose content minimum exceeds the tab bar (`m46_minimize_tall_group`, 185).
- [x] 5.7 Primary-column empty-removal and recreate-on-show (`m46_primary_empty`, 186).

## 6. Verification

- [x] 6.1 `openspec validate --all --strict` (61 passed).
- [x] 6.2 `bash scripts/verify-fast.sh`: fmt ok, clippy 0 errors, 589 tests passed. Guard trips only on the pre-existing untracked `docs/dev/mcp-agentic-control-plan.md`, which is not part of this change.
- [x] 6.3 `cargo test --workspace` (via verify-fast) and `cargo test --workspace --doc`.
- [x] 6.4 CMake build and `./build/pictura --headless --self-test` (exit 0, all `m46_*` green).
- [ ] 6.5 Manual real-QPA pass over the reported items (deferred; not runnable headless in this environment).
