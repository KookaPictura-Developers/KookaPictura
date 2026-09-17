## ADDED Requirements

### Requirement: Explicit headless mode
The system SHALL provide a `--headless` command-line flag that selects the Qt
offscreen QPA plugin before constructing `QApplication`, so the executable runs
without an X or Wayland display. The flag SHALL take precedence over the
`--self-test` xcb override and SHALL require no external display server. When
`--headless` is given with neither a document argument nor `--self-test`, the
system SHALL run the bridge self-test. `--headless` SHALL NOT force the offscreen
plugin onto `--interop-probe`, which requires a platform Vulkan instance.

#### Scenario: Headless selects the offscreen platform
- **WHEN** the executable is started with `--headless` and `QT_QPA_PLATFORM` is unset
- **THEN** the offscreen QPA plugin is selected before `QApplication` is constructed and the process does not require `DISPLAY` or `WAYLAND_DISPLAY`

#### Scenario: Headless self-test asserts the offscreen platform
- **WHEN** `--headless --self-test` runs
- **THEN** the self-test verifies that the application platform is `offscreen`, prints the self-test summary, and exits 0

#### Scenario: A bare headless run implies the self-test
- **WHEN** `--headless` is given with no document argument and without `--self-test`
- **THEN** the system runs the bridge self-test instead of blocking in the event loop

#### Scenario: An explicitly set platform is respected
- **WHEN** `--headless` is given and `QT_QPA_PLATFORM` is already set in the environment
- **THEN** the environment value is used unchanged

#### Scenario: Wrong platform fails the headless self-test
- **WHEN** the headless self-test runs while the application platform is not `offscreen`
- **THEN** the self-test prints a failure and exits non-zero

## MODIFIED Requirements

### Requirement: Headless window shell
The system SHALL open a top-level Qt Widgets `QMainWindow` frame containing a
menu bar, a tabbed document area that hosts one canvas per open document, a
status bar, and dock areas, and SHALL run under a headless display server (Xvfb,
the offscreen QPA plugin, or the explicit `--headless` flag) without requiring a
GPU.

#### Scenario: Run under Xvfb
- **WHEN** the executable is started under `xvfb-run` or with `QT_QPA_PLATFORM=offscreen`
- **THEN** it opens its window and exits without error

#### Scenario: Run with the headless flag
- **WHEN** the executable is started with `--headless`
- **THEN** it opens its window on the offscreen platform and exits without error

#### Scenario: Frame chrome is present at startup
- **WHEN** the window is shown
- **THEN** the menu bar, tabbed document area, status bar, and dock area exist

#### Scenario: No document open
- **WHEN** the frame starts with no document loaded
- **THEN** document-requiring commands are disabled and the application does not crash
