# verification/qt-test-harness Specification

## Purpose
The Qt Test + CTest harness for the C++ GUI shell: an application static library
that test executables link, per-suite executables registered with CTest, per-suite
state isolation, the seeded suites migrated off the self-test, and the contract
that the monolithic self-test only shrinks as checks move.

## Requirements

### Requirement: Application shell library

The C++ application SHALL be split into a STATIC library `pictura_shell` holding
every application source except `main.cpp` and the `selftest*.cpp`/`selftest*.h`
harness files, and a `pictura` executable holding `main.cpp` plus the self-test
harness that links `pictura_shell`. The split SHALL be a pure move with no
behavior change: the `pictura` executable's observable behavior, including the
`--headless --self-test` token stream and exit codes, SHALL be unchanged.
`pictura_shell` SHALL link `pictura_app` (the Rust staticlib), `Qt6::Svg`,
`Qt6::Network`, and lcms2 with PUBLIC visibility, and SHALL expose
`crates/pictura-app/cpp` as a PUBLIC include directory, so a target that links
`pictura_shell` inherits the cxx-qt generated headers and the native link
dependencies without repeating them. A STATIC library is chosen over an OBJECT
library so the linker pulls only the objects a test executable references and
the self-test objects do not bloat the test executables.

#### Scenario: The executable keeps only main and the harness

- **WHEN** the shell library is built
- **THEN** `pictura_shell` contains no `main.cpp` and no `selftest*` source, and
  the `pictura` executable links it

#### Scenario: A test executable inherits the cxx-qt surface

- **WHEN** a test executable links `pictura_shell`
- **THEN** it can include the cxx-qt generated headers and link lcms2 without
  declaring either itself

#### Scenario: The self-test behavior is unchanged

- **WHEN** the relocated `pictura` binary runs `--headless --self-test`
- **THEN** it emits the same tokens and returns the same exit codes as before
  the split

### Requirement: Qt Test and CTest harness

The build SHALL call `include(CTest)` gated on `BUILD_TESTING` and SHALL add a
`crates/pictura-app/cpp/tests/` directory with its own `CMakeLists.txt`. That
file SHALL build each `tst_*.cpp` as its own executable, linking `pictura_shell`
and `Qt6::Test`, and SHALL register each executable with `add_test` under
`ENVIRONMENT QT_QPA_PLATFORM=offscreen` and `TIMEOUT 120`. Each `add_test` SHALL
pass `-o <CMAKE_BINARY_DIR>/qt-test-results/<name>.xml,junitxml` so CTest itself
emits a per-executable JUnit report. The harness SHALL be gated on
`BUILD_TESTING` with no effect when it is off. CTest SHALL own only the Qt Test
executables and SHALL NOT wrap or replace `cargo nextest`; the Rust suite and
doctests stay with `scripts/test-report.sh`.

#### Scenario: Configure builds the suites

- **WHEN** CMake configures with `BUILD_TESTING` on
- **THEN** the tests subdirectory is added and each `tst_*.cpp` is built as its
  own executable linked against `pictura_shell` and `Qt6::Test`

#### Scenario: CTest runs each suite offscreen

- **WHEN** `ctest` runs a registered Qt test executable
- **THEN** it runs with `QT_QPA_PLATFORM=offscreen` and a `TIMEOUT` of 120
  seconds

#### Scenario: CTest emits per-executable JUnit

- **WHEN** `ctest` runs a registered Qt test executable
- **THEN** the executable writes a JUnit report to
  `build/qt-test-results/<name>.xml`

#### Scenario: Tests are optional

- **WHEN** CMake configures with `BUILD_TESTING` off
- **THEN** no test executable or test registration is emitted

#### Scenario: CTest does not wrap nextest

- **WHEN** the harness is built
- **THEN** no CTest test invokes `cargo nextest` or the doctests

### Requirement: Per-suite state isolation

Each Qt Test suite SHALL isolate persisted state so no suite depends on or
mutates another's preferences or recovery state. Each suite SHALL declare a
`pictura::test::ScopedStateHome` RAII member constructed BEFORE its main window,
so the fixture outlives the window's session save. The fixture's constructor
SHALL point `XDG_STATE_HOME` at a fresh temporary directory, and its destructor
SHALL restore the previous value (including unsetting it only when it was not
set before), rather than blanket-unsetting. Isolation SHALL NOT rely on
`initTestCase()`: the fixture must be alive across window construction and
destruction, which `initTestCase()` cannot guarantee. The frame SHALL read
`XDG_STATE_HOME` and `QApplication` SHALL NOT, so `QTEST_MAIN` SHALL be used
without a custom main function or a global setup hook.

#### Scenario: Suites do not share state

- **WHEN** two suites run in the same `ctest` invocation
- **THEN** each runs as its own executable in its own process with its own
  temporary state home, so neither observes state written by the other

#### Scenario: Each run starts clean

- **WHEN** a suite writes preferences and the suite runs again
- **THEN** it starts from a clean temporary state directory

#### Scenario: A pre-existing state home is restored

- **WHEN** `XDG_STATE_HOME` is already set in the environment and the fixture is
  destroyed
- **THEN** the previous value is restored, not unset

### Requirement: Self-test migration and code retirement

The Qt Test suites SHALL be populated by MOVING checks off the monolithic
self-test, not by duplicating them: a migrated check SHALL be DELETED from its
`selftest*.cpp` source. A retired self-test exit code SHALL never be reused; the
next self-test check SHALL take the next unused code. `design.md` SHALL record
every retired check by name and integer exit code. New GUI checks SHALL be
written as Qt Test cases; a new self-test check MAY be added only to preserve the
`--headless` smoke contract and the `--self-test` exit-code identity.
`scripts/check-file-size.sh` SHALL continue to classify
`crates/pictura-app/cpp/tests/` as tests under the 1400-line budget with no rule
change. The shrink-only rule SHALL be enforced mechanically by
`scripts/check-selftest-budget.sh`, which counts `ST_BEGIN` sites across
`crates/pictura-app/cpp/**/*.cpp` and fails when the count exceeds the budget in
`scripts/selftest-budget.txt`; the budget is lower-only, and the script SHALL run
from `scripts/verify-fast.sh` and the guards CI workflow. This is in addition to
the `selftest.cpp` entry in `scripts/file-size-allowlist.txt`, which remains a
ceiling that may only shrink.

#### Scenario: A migrated check leaves the self-test

- **WHEN** a check is ported to a Qt Test suite
- **THEN** its `ST_BEGIN` block is removed from the self-test and the check runs
  only through CTest

#### Scenario: A retired code is never reused

- **WHEN** a self-test check is retired
- **THEN** its integer exit code is not assigned to any later check

#### Scenario: New checks use Qt Test

- **WHEN** a new GUI check is added
- **THEN** it is a Qt Test case, unless it preserves the `--headless` smoke
  contract or the `--self-test` exit-code identity

#### Scenario: Test files get the test budget

- **WHEN** `scripts/check-file-size.sh` runs on a file under
  `crates/pictura-app/cpp/tests/`
- **THEN** it applies the 1400-line test budget

#### Scenario: The self-test budget only shrinks

- **WHEN** `scripts/check-selftest-budget.sh` runs and a new `ST_BEGIN` site
  raises the count above `scripts/selftest-budget.txt`
- **THEN** the script fails

### Requirement: Seeded suites

The change SHALL seed four Qt Test executables: the three MIGRATED suites
`tst_command_tree`, `tst_layers_panel`, and `tst_edit_clipboard`, plus
`tst_smoke`, the harness smoke suite that exercises the state-home fixture and
proves CTest runs a suite offscreen. `tst_command_tree` SHALL cover the command
registry and menu tree, including the widget-level menu checks `menus`,
`dispatch`, `menu_count`, `menubar_clear`, `menus_panel`,
`panelMenus_tools_icons`, and `widgetmenu_button`. `tst_layers_panel` SHALL cover
Layers panel row controls/chrome (the opacity label, `%` suffix, semantic lock
icons, and tree drag enabled), group expand/collapse, and drop bands via
`lpc_nesting`, `lpc_chrome`, `lpr_rows`, `lpr_drag`, and `lpr_drop`; it does not
assert row selection. `tst_edit_clipboard` SHALL cover the whole
`edit_clipboard` check.

#### Scenario: The seeded suites run

- **WHEN** `ctest` runs the harness
- **THEN** it runs `tst_smoke`, `tst_command_tree`, `tst_layers_panel`, and
  `tst_edit_clipboard`

#### Scenario: The migrated checks leave the self-test

- **WHEN** the seed migration is complete
- **THEN** none of the named checks emits a self-test token
