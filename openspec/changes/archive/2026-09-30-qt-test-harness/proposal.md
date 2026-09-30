# Proposal

## Why

The C++ GUI layer has one hand-rolled self-test: `runSelfTest()` in
`crates/pictura-app/cpp/selftest.cpp`, a single sequential function with 511
`ST_BEGIN` sites (475 executed in a bare run) in one process. It cannot isolate
state between checks, stops
at the first failure (`ST_FAIL` returns), and has no `QSignalSpy`,
`QTest::mouseClick`/`keyClick`, or data-driven (`_data()`) tests.
`docs/11-cross-cutting/testing-strategy.md` (`XC-010`, Draft) proposed Qt Test +
CTest but never shipped. This change implements that harness and migrates the
first checks onto it.

## What Changes

- **Split the app into a shell library and a thin executable.** All app C++
  sources except `main.cpp` and the `selftest*.cpp`/`selftest*.h` harness files
  move into a new STATIC library `pictura_shell`; the `pictura` executable keeps
  `main.cpp` plus the self-test harness and links `pictura_shell`. Pure move, no
  behavior change. `pictura_shell` links `pictura_app` (the Rust staticlib),
  `Qt6::Svg`, `Qt6::Network`, and lcms2 PUBLICly and exposes
  `crates/pictura-app/cpp` as a PUBLIC include dir, so cxx-qt generated headers
  reach test executables.
- **Add a Qt Test + CTest harness.** `include(CTest)` gated on `BUILD_TESTING`; a
  `crates/pictura-app/cpp/tests/` directory with its own `CMakeLists.txt` builds
  each `tst_*.cpp` as its own executable, links `pictura_shell` + `Qt6::Test`,
  and registers it with `add_test` under `ENVIRONMENT QT_QPA_PLATFORM=offscreen`
  and `TIMEOUT 120`.
- **Seed three suites by moving checks off the self-test** (a migration, not a
  permanent parallel layer): `tst_command_tree` (command registry and menu tree,
  including widget-level menu checks), `tst_layers_panel` (row controls/chrome,
  group expand/collapse, and drop bands), and `tst_edit_clipboard` (the whole
  `edit_clipboard` check). Moved checks are DELETED from the self-test. A fourth
  executable, `tst_smoke`, is the harness smoke test: it exercises the state-home
  fixture and proves CTest runs a suite offscreen.
- **Retire exit codes, never reuse them.** A retired self-test exit code is never
  reassigned; every retired check name and its integer code is recorded in
  `design.md`.
- **Fold Qt Test results into the unified report.** CTest registers each
  `tst_*` executable and emits per-executable JUnit via `-o <path>,junitxml`;
  `scripts/report_tests.py` gains a Qt layer that reuses its existing JUnit
  parser; `scripts/test-report.sh` runs `ctest -R '^tst_'` and folds
  `build/qt-test-results/*.xml` into the report. CTest is a local convenience
  wrapper for the Qt tests only and does NOT wrap `cargo nextest`.
- **State a migration guard.** New GUI checks MUST be Qt Test; the self-test only
  shrinks. A new self-test check may be added only to preserve the `--headless`
  smoke contract and the `--self-test` exit-code identity.

## Capabilities

### New Capabilities

- `verification/qt-test-harness`: the `pictura_shell` static library, the Qt
  Test + CTest harness, the per-executable test layout, per-suite state
  isolation, and the self-test migration/retirement contract.

### Modified Capabilities

- `verification/test-reporting`: the unified report SHALL also cover the Qt Test
  executables, and `scripts/test-report.sh` SHALL run and fold their per-exe
  JUnit output.

## Impact

- New: `crates/pictura-app/cpp/tests/CMakeLists.txt`,
  `crates/pictura-app/cpp/tests/qt_test_support.h` (the `ScopedStateHome` fixture
  and `makeMainWindow`), `crates/pictura-app/cpp/tests/tst_smoke.cpp`,
  `crates/pictura-app/cpp/tests/tst_command_tree.cpp`,
  `crates/pictura-app/cpp/tests/tst_layers_panel.cpp`,
  `crates/pictura-app/cpp/tests/tst_edit_clipboard.cpp`,
  `scripts/check-selftest-budget.sh`, and `scripts/selftest-budget.txt`.
- Edit: `CMakeLists.txt` (new `pictura_shell` STATIC library, thin `pictura`
  executable, `include(CTest)`, `add_subdirectory`),
  `scripts/report_tests.py` (Qt JUnit layer), `scripts/test-report.sh` (run CTest
  and fold the Qt JUnit), `docs/dev/testing-conventions.md`, `AGENTS.md`,
  `crates/pictura-app/cpp/selftest.cpp`,
  `crates/pictura-app/cpp/selftest_layers_controls.cpp`, and the corresponding
  `scripts/file-size-allowlist.txt` ceilings.
- Remove: `crates/pictura-app/cpp/selftest_clipboard.cpp` and
  `selftest_clipboard.h` (the `edit_clipboard` check moves to Qt Test).
- Dependency note (AGENTS.md rule 4): `Qt6::Test` ships with the
  already-required system Qt; it is build-time and test-only, so no new runtime
  dependency is introduced.
- Touching `docs/` requires a `TASK-ALLOWS-DOCS` marker on the eventual commit.
- Out of scope: porting photorust's individual `tst_*` cases (they target
  photorust widgets), rewriting the whole self-test, and adopting the rest of
  `XC-010` (proptest, fuzz, Criterion, golden manifests, `xtask`). The monolithic
  self-test stays as the `--headless` smoke test.
