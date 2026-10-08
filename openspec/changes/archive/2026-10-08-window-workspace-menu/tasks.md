# Tasks

## 1. Workspace store

- [x] 1.1 Add `crates/pictura-app/cpp/workspace_store.h`/`.cpp`: a `WorkspaceStore` reading/writing `$XDG_STATE_HOME/kooka-pictura/workspaces/` (an `index.json` with schema version, active workspace, and order; one `<id>.json` per workspace holding `{name, kind, layout, factory?}`) via `QSaveFile`, preserving unknown keys. Verify `WorkspaceStore::directory()` equals `QFileInfo(pictura::sessionFilePath()).absolutePath() + "/workspaces"`.
- [x] 1.2 Register `workspace_store.cpp` in `CMakeLists.txt` (no globbing) and verify `cmake --build build` links.
- [x] 1.3 Add a store unit test in `tst_workspaces.cpp` (round-trip, unknown-key survival, corrupt **index** falls back to built-ins, corrupt single workspace drops only it) and register `tst_workspaces` in `PICTURA_QT_TESTS` in `crates/pictura-app/cpp/tests/CMakeLists.txt`. Verify `ctest --test-dir build -N -R '^tst_workspaces$'` lists it and the suite passes.

## 2. Re-group-capable apply engine

- [x] 2.1 Add `PicturaMainWindow::regroupAndApply(const QJsonArray& columns)` per design D4 in a new `frame_workspaces.cpp` (declared in `frame.h`): harvest panels from docked groups and floats (`exclude` the tools column), tear down floats, clear dynamic columns, rebuild groups, place the main column and an anchored secondary column, restore per-group state/rail/width, re-show the primary if `removeColumnIfEmpty` hid it, and keep unnamed panels in a hidden overflow group. Verify compile.
- [x] 2.2 Register `frame_workspaces.cpp` in `CMakeLists.txt` and verify the build.
- [x] 2.3 Add a Qt Test applying a layout that re-groups a panel (panel ends in a different group), that re-docks a floating panel, and that leaves the Tools column unchanged; verify all three.
- [x] 2.4 Add a Qt Test that applies a preset omitting a default panel and then invokes `Window > Panels > <panel>`, verifying the panel is reachable and the check reflects visibility.

## 3. Preset factories and authentic Essentials default

- [x] 3.1 Add `workspacePresets()` returning the factory `panelColumns` for Essentials, Painting, Photography, and Typography per design D6, and verify each builds a valid column array.
- [x] 3.2 Build the fresh-session default from the Essentials factory **after** `applyPanelSession`'s `clearDynamicColumns()` (or spare a built-in secondary column), so a fresh session really shows two columns. Verify a Qt Test asserts the fresh session has the main groups `Color|Swatches`, `Adjustments|Styles`, `Layers|Channels|Paths` and an iconic secondary column `History`, `Properties`.
- [x] 3.3 Update the checks that assert the old default: the type-panel Qt suites (`tst_type_panels`, `tst_character_paragraph_panels`, `tst_paragraph_styles_panel`), and the self-test checks `groups_grouped` (`selftest.cpp:1707`), the `panelColumnCountForTest() == 3` assertions (`selftest_ui_persistence.cpp:93`, `selftest_session.cpp:130`, `selftest.cpp:4470`), and the drag/tear-off checks that pull Navigator/Histogram/Info from the primary column. Do not grow `selftest.cpp` (allowlist ceiling); retire or adjust in place. Verify `ctest --test-dir build -R '^tst_' --output-on-failure`, `./build/pictura --headless --self-test`, and `bash scripts/check-selftest-budget.sh` all pass.

## 4. Menu wiring and the active indicator

- [x] 4.1 Add the frozen workspace command ids in `commands.h`; register the four presets, New/Delete/Reset under those ids (presets as `CommandSpec{checkable = true}`, not `leaf()`), and remove the 3D/Advanced 3D/Motion/`New Features` placeholder leaves entirely. Verify a Qt Test asserts the four presets and New/Delete/Reset are enabled and checkable where expected, and the placeholders do not exist.
- [x] 4.2 Add `wireWorkspaceCommands()` in `frame_workspaces.cpp` (called once from `frame_menus.cpp`) wiring handlers, `setCheckedProvider` (active name), and the `Reset [Workspace]` label provider. Verify `frame_menus.cpp` stays under its file-size cap via `bash scripts/check-file-size.sh`, and that the active workspace is checked and the checkmark moves on switch (Qt Test).
- [x] 4.3 Populate user workspaces in the `Window > Workspace` submenu on `aboutToShow` without duplicating actions across repeated shows. Verify a Qt Test shows the menu twice and counts one entry per workspace.

## 5. New / Delete / Reset

- [x] 5.1 Implement `New Workspace…` (name prompt; reject empty/whitespace, duplicate user name, built-in name, and over-length; snapshot current layout; make active) and verify a Qt Test creates a workspace, sees it after a store reload, and is refused `Essentials` and an existing user name.
- [x] 5.2 Implement `Delete Workspace…` (always enabled; chooser lists every workspace except the active; the active cannot be deleted; presets are deletable) and verify Qt Tests cover deleting a non-active user workspace, deleting a preset, and the chooser excluding the active.
- [x] 5.3 Implement `Reset [Workspace]` restoring a built-in's CS6 factory or a user workspace's creation snapshot, and verify a Qt Test resets a modified workspace, then switches away and back, and sees the factory; and that the label reads `Reset <name>`.

## 6. Auto-remember and durability

- [x] 6.1 Implement switch-time snapshot + startup reconciliation (design D3), including snapshotting the active workspace's layout into the store on save/quit so it is durable independent of `state.json`. Verify a Qt Test changes a layout, switches away and back, and sees the changed layout; then resets and sees the factory.
- [x] 6.2 Verify the active workspace's arrangement is restored when `state.json` is discarded but the store is intact (drive `WorkspaceStore` directly in the test).

## 7. Integration verification

- [x] 7.1 Run `openspec validate --all --strict` and verify all four deltas validate.
- [x] 7.2 Run `bash scripts/guard.sh`, `bash scripts/check-file-size.sh`, and `bash scripts/check-selftest-budget.sh` and verify all pass.
- [x] 7.3 Run `bash scripts/test-report.sh` and verify the unified report shows `tst_workspaces` passing and no regressions.

## 8. Menu cleanup and Options-bar switcher

- [x] 8.1 Remove the placeholder preset leaves (3D, Advanced 3D, Motion, New Features) from `command_tree.cpp` and add separators after the presets and after `Reset`. Verify a Qt/spec check that they are absent and the menu has the three sections.
- [x] 8.2 Add the Options-bar workspace switcher (`OptionsBar` button showing the active workspace, opening the `Window > Workspace` menu) and attach it from the controller. Verify it shows the active workspace and stays visible across tools.
- [x] 8.3 Make `Delete Workspace…` always enabled and chooser-based, listing every workspace except the active; allow deleting presets; seed presets only when the store is absent so a deleted preset stays deleted. Verify a deleted preset does not reappear after a store reload.
- [x] 8.4 Update `tst_workspaces` and `tst_command_tree` for the above, then run `cmake --build build`, `ctest -R '^tst_'`, `./build/pictura --headless --self-test`, and `openspec validate --all --strict`.
