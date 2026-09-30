# Tasks

## 1. Application shell library

- [x] 1.1 In `CMakeLists.txt`, add a STATIC library `pictura_shell` with every source currently in the `pictura` executable except `main.cpp` and the `selftest*.cpp`/`selftest*.h` files; verify CMake configures and lists `pictura_shell`.
- [x] 1.2 Reduce the `pictura` executable to `main.cpp` plus the `selftest*` sources, link `pictura_shell`, and move the include dir plus `pictura_app`/`Qt6::Svg`/`Qt6::Network`/lcms2 link line onto `pictura_shell` PUBLICly; verify the build links.
- [x] 1.3 Build and run `./build/pictura --headless --self-test` with and without the `two_layers.psd` fixture; verify the token stream, `SUMMARY`, and exit codes are unchanged from before the split.

## 2. Qt Test and CTest harness

- [x] 2.1 Add `include(CTest)` and an `if(BUILD_TESTING)` branch to `CMakeLists.txt` that finds `Qt6::Test` and adds `crates/pictura-app/cpp/tests/`; verify configure succeeds with `BUILD_TESTING` on and emits no test targets with it off.
- [x] 2.2 Create `crates/pictura-app/cpp/tests/CMakeLists.txt` that builds each `tst_*.cpp` as its own executable linking `pictura_shell` and `Qt6::Test`, registering each with `add_test` under `ENVIRONMENT QT_QPA_PLATFORM=offscreen` and `TIMEOUT 120`; verify `ctest -N` lists the targets.
- [x] 2.3 Add a minimal `tst_smoke.cpp` exercising the `ScopedStateHome` temp-`XDG_STATE_HOME` fixture constructed before the window and run `ctest`; verify it passes offscreen and that separate suite processes share no state.

## 3. Migrate `tst_command_tree`

- [x] 3.1 Port `menus` (25) and `dispatch` (26) from `crates/pictura-app/cpp/selftest.cpp` into `tst_command_tree.cpp` using `QSignalSpy`/registry assertions; verify the Qt suite passes offscreen.
- [x] 3.2 Port `menu_count` (123), `menubar_clear` (138), `menus_panel` (110), `panelMenus_tools_icons` (139), and `widgetmenu_button` (135), keeping the widget-level interaction checks; verify the suite passes offscreen.
- [x] 3.3 Delete the ported `ST_BEGIN`/`ST_FAIL` blocks from `selftest.cpp`, adjust the allowlist entry, and verify `scripts/check-file-size.sh` passes and `--headless --self-test` still runs.
- [x] 3.4 Record the retired codes against `tst_command_tree` in the `design.md` ledger and verify no remaining self-test check uses codes 25, 26, 110, 123, 135, 138, or 139.

## 4. Migrate `tst_layers_panel`

- [x] 4.1 Port `lpr_rows` (211), `lpr_drag` (212), and `lpr_drop` (213) from `crates/pictura-app/cpp/selftest_layers_controls.cpp` into `tst_layers_panel.cpp`; verify the suite passes offscreen.
- [x] 4.2 Port the group expand/collapse checks `lpc_chrome` (210) and `lpc_nesting` (200); verify the suite passes offscreen.
- [x] 4.3 Delete the ported blocks from `selftest_layers_controls.cpp`, adjust the allowlist entry, and verify `scripts/check-file-size.sh` passes and `--headless --self-test` still runs.
- [x] 4.4 Record the retired codes (200, 210, 211, 212, 213) in the `design.md` ledger and verify none is reused.

## 5. Migrate `tst_edit_clipboard`

- [x] 5.1 Port the whole `edit_clipboard` check (529) from `crates/pictura-app/cpp/selftest_clipboard.cpp` into `tst_edit_clipboard.cpp`, preserving the raster/copy/cut/paste/purge assertions; verify the suite passes offscreen.
- [x] 5.2 Delete the ported function (and register the removal if `runClipboardChecks` becomes unused), adjust the allowlist entry, and verify `scripts/check-file-size.sh` passes and `--headless --self-test` still runs.
- [x] 5.3 Record the retired code 529 in the `design.md` ledger and verify it is not reused.

## 6. Unified report integration

- [x] 6.1 Add the Qt JUnit layer to `scripts/report_tests.py`, reusing `parse_junit`, and verify `python3 scripts/report_tests.py --self-check` passes.
- [x] 6.2 In `scripts/test-report.sh`, run `ctest --test-dir build -R '^tst_'` and pass the per-executable JUnit (`build/qt-test-results/*.xml`) to the reporter; verify the report lists a Qt Test subtotal and exits 0.
- [x] 6.3 Introduce one deliberate failing Qt Test case, verify it appears under `FAILURES` with a non-zero report exit, then revert it.

## 7. Migration guard documentation

- [x] 7.1 Update `docs/dev/testing-conventions.md` with the Qt Test + CTest layer, the migration guard (new GUI checks are Qt Test; the self-test only shrinks), and the new report commands, under a `TASK-ALLOWS-DOCS` commit.
- [x] 7.2 Update the testing section of `AGENTS.md` to name the Qt Test harness, the `tst_*` layout, and the guard; verify the commit carries `TASK-ALLOWS-DOCS`.

## 8. Integration verification

- [x] 8.1 Run `openspec validate --all --strict` and confirm it is clean.
- [x] 8.2 Reconfigure and build with CMake, run `ctest --output-on-failure`, and verify all three suites plus the smoke suite pass offscreen.
- [x] 8.3 Run `bash scripts/test-report.sh` and verify the unified report includes Rust, doctests, the self-test, and the Qt Test layer and exits 0.
- [x] 8.4 Run `./build/pictura --headless --self-test` and verify the remaining self-test still emits its tokens and pass summary.
