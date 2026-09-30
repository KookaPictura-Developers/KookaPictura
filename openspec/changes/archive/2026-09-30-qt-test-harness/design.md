# Design

## Context

See `proposal.md` — Why. The current state that shapes the approach:

- The C++ shell is one `add_executable(pictura …)` in the root `CMakeLists.txt`
  listing `main.cpp`, every app source, and every `selftest*.cpp`. There is no
  library target, so a test executable cannot link the app without rebuilding it
  and inheriting the self-test harness.
- `runSelfTest()` (`crates/pictura-app/cpp/selftest.cpp`, one sequential
  function) plus the `selftest_*.cpp` helpers hold 511 `ST_BEGIN` sites (475
  executed in a bare run) in one process. `ST_FAIL` returns the check's code, so
  the run stops at the first failure.
- `scripts/report_tests.py` already parses JUnit (`parse_junit`, reading
  `<testsuite>`/`<testcase>`/`<failure|skipped>`), the doctest summary, and the
  self-test token streams. `scripts/test-report.sh` folds them into one report.
- `scripts/check-file-size.sh` detects tests by path: Rust under `tests/` or
  `tests.rs`, C++ `*_test.{cpp,h}`. A `tst_*.cpp` under `cpp/tests/` is covered
  by the `*/tests/*` classification, so it gets the 1400-line test budget with no
  rule change.
- Constraints: `docs/` edits need `TASK-ALLOWS-DOCS`; `Qt6::Test` ships with the
  already-required system Qt (`qt6-base-dev`); milestones must not appear in
  identifiers or test names.

## Goals / Non-Goals

**Goals:** a linkable application shell library; a Qt Test + CTest harness for
GUI tests; three seeded suites migrated off the self-test; per-executable JUnit
folded into the unified report; retired self-test codes frozen and recorded.

**Non-Goals:** porting photorust's `tst_*` cases (they target photorust widgets);
rewriting the whole self-test; adopting the rest of `XC-010` (proptest, fuzz,
Criterion, golden manifests, `xtask`); making CTest the Rust runner; parallelizing
either harness.

## Decisions

### D1 — `pictura_shell` is a STATIC library
The shell is a STATIC library, not OBJECT or SHARED. The linker pulls only the
objects a consumer references, so the self-test translation units do not bloat
the test executables built under `build/crates/pictura-app/cpp/tests/`, and each
test executable links the same code the app uses. SHARED would add a runtime
library to ship and load; OBJECT would link every object into every executable.

### D2 — Shell contents vs executable contents
`pictura_shell` holds every app source except `main.cpp` and the
`selftest*.cpp`/`selftest*.h` files. The `pictura` executable holds `main.cpp`
plus the self-test harness and links `pictura_shell`. This is a pure move: no
source is renamed and no behavior changes. It keeps the self-test (an
application concern wired into `main.cpp`) out of the test executables.

### D3 — PUBLIC link and include visibility
`pictura_shell` links `pictura_app` (the Rust staticlib), `Qt6::Svg`,
`Qt6::Network`, and lcms2 PUBLICly and exposes `crates/pictura-app/cpp` as a
PUBLIC include directory. A staticlib does not propagate its native link
dependencies (the current `CMakeLists.txt` already adds `-llcms2` by hand), and
the cxx-qt generated headers live under the crate tree, so test executables need
both without restating them. PRIVATE visibility would force every test target to
repeat the link list.

### D4 — Tests live at `crates/pictura-app/cpp/tests/`
The Qt suites sit in `crates/pictura-app/cpp/tests/` with their own
`CMakeLists.txt`. The path is already classified as tests by
`scripts/check-file-size.sh` (`*/tests/*` → 1400-line budget), and the suites sit
next to the headers they exercise. Each `tst_*.cpp` is its own executable so a
crash or hang in one suite cannot take the others down.

### D5 — State isolation via a `ScopedStateHome` RAII fixture
Each suite declares a `pictura::test::ScopedStateHome` member BEFORE its
`PicturaMainWindow`, so the fixture outlives the window's session save. The
constructor points `XDG_STATE_HOME` at a fresh `QTemporaryDir`; the destructor
restores the pre-existing value (save/restore, not a blanket unset). It is a
member, not `initTestCase()` setup: the fixture must be alive while the window is
constructed and destroyed, and `initTestCase()` cannot guarantee that ordering.
The frame reads `XDG_STATE_HOME`; `QApplication` does not, so `QTEST_MAIN` works
without a custom main or a global setup hook. Suites do not share state because
each `tst_*` is its own executable in its own process, and each gets its own
temporary state home. This mirrors what `main.cpp` already does for the headless
self-test.

### D6 — CTest owns the Qt tests only
`include(CTest)` and `add_test` cover the `tst_*` executables. CTest does NOT
wrap `cargo nextest`: nextest plus doctests are already owned by
`scripts/test-report.sh`, and duplicating them into CTest would create a second
runner to keep in sync. CTest is a local convenience for running and filtering
GUI suites.

### D7 — Gated on `BUILD_TESTING`
`include(CTest)` defines `BUILD_TESTING` (default ON); the tests subdirectory is
added only when it is set. `Qt6::Test` is found only in that branch. Test-only
and build-time, so no new runtime dependency (AGENTS.md rule 4).

### D8 — Seed scope is "everything" for the three suites
The seed includes widget-level menu checks (not only the data-level registry), so
`tst_command_tree` proves `QTest::mouseClick`/`QSignalSpy` style widget
interaction, not just registry calls. Seed scope is the named checks only;
porting the other 500-odd checks is future work.

### Migration guard
New GUI checks are Qt Test; the self-test only shrinks. A new self-test check may
be added only to preserve the `--headless` smoke contract and the `--self-test`
exit-code identity. This is stated in `docs/dev/testing-conventions.md` and
`AGENTS.md`. Without the guard the two harnesses would grow in parallel, which
this change explicitly avoids.

### Dependency note (AGENTS.md rule 4)
`Qt6::Test` is part of the system Qt 6 the build already requires
(`qt6-base-dev`, found by `find_package(Qt6 …)`), is linked only by test
executables, and is not a runtime dependency of `pictura`. No new third-party
dependency is introduced.

### Code-retirement ledger
Every code below is retired by this change: the check moves to its Qt Test suite
and its `ST_BEGIN`/`ST_FAIL` block is deleted from the self-test. Codes are
append-only and are never reused. The next new self-test check takes the next
unused code after the current maximum.

| Retired check | Suite | Exit code |
|---|---|---|
| `menus` | `tst_command_tree` | 25 |
| `dispatch` | `tst_command_tree` | 26 |
| `menus_panel` | `tst_command_tree` | 110 |
| `menu_count` | `tst_command_tree` | 123 |
| `widgetmenu_button` | `tst_command_tree` | 135 |
| `menubar_clear` | `tst_command_tree` | 138 |
| `panelMenus_tools_icons` | `tst_command_tree` | 139 |
| `lpc_nesting` | `tst_layers_panel` | 200 |
| `lpc_chrome` | `tst_layers_panel` | 210 |
| `lpr_rows` | `tst_layers_panel` | 211 |
| `lpr_drag` | `tst_layers_panel` | 212 |
| `lpr_drop` | `tst_layers_panel` | 213 |
| `edit_clipboard` | `tst_edit_clipboard` | 529 |

`lpc_chrome` carries the group expand/collapse assertion
(`chevronClickExpandsForTest`); `lpc_nesting` carries the nesting lock/reparent
assertions. If a named check proves incoherent to port, it stays in the
self-test and its code is not retired until it does move.

## Risks / Trade-offs

- [Moving ~180 sources into a library touches the build in one large edit] → the
  move is mechanical (source list + `target_link_libraries`); the self-test token
  stream and exit codes are the regression check that behavior did not change.
- [Widget-level Qt Test cases can be flaky under offscreen] → each test runs
  `QT_QPA_PLATFORM=offscreen` with a 120s timeout, suites use `QSignalSpy` and
  `QTRY_*` rather than fixed sleeps, and a flaky suite is fixed or descoped
  rather than retried.
- [A retired self-test code could collide if the ledger drifts] → the ledger is
  recorded here and re-checked when a check is removed; `ST_FAIL` codes are
  append-only, so a removed code is simply never emitted again.
- [Public include/link propagation could leak test-only flags into the app] →
  only existing app link requirements (lcms2, Qt modules) become PUBLIC, and
  `Qt6::Test` stays PRIVATE to the test executables.
- [Two test systems could coexist forever] → the migration guard makes the
  self-test shrink-only and new GUI checks Qt Test only.

## Migration Plan

1. Add `pictura_shell` and thin `pictura` in `CMakeLists.txt`; run the headless
   self-test and confirm the token stream and exit codes are unchanged.
2. Add `include(CTest)`, `crates/pictura-app/cpp/tests/CMakeLists.txt`, and one
   trivial smoke `tst_*`; confirm `ctest` runs it offscreen.
3. Port the three suites, deleting each check from its `selftest*.cpp` and
   updating the file-size allowlist entries.
4. Add the Qt JUnit layer to `report_tests.py` and the run step to
   `test-report.sh`; confirm the unified report includes it.
5. Update `docs/dev/testing-conventions.md` and `AGENTS.md` with the migration
   guard, under a `TASK-ALLOWS-DOCS` commit.

Rollback is per step: the shell split is behavior-neutral, and each ported suite
can return to the self-test with its original code (still recorded above) if a
suite cannot be stabilised.
