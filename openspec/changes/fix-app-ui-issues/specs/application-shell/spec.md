## MODIFIED Requirements

### Requirement: Headless window shell
The system SHALL open a top-level Qt Widgets `QMainWindow` frame containing a
menu bar, a tabbed document area that hosts one canvas per open document, a
status bar, and dock areas, and SHALL run under a headless display server (Xvfb,
the offscreen QPA plugin, or the explicit `--headless` flag) without requiring a
GPU. A normal launch with no document argument SHALL open no document and leave
the document-requiring commands disabled; the scratch document used by the
self-test checks SHALL be created only for a `--self-test` or `--headless` run
that needs it. An explicitly opened or created document SHALL behave as before.

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

#### Scenario: A normal launch opens no document [las_no_scratch]
- **WHEN** the application is launched with no document argument and without
  `--self-test` or `--headless`
- **THEN** no tab and no document are created, and the document-requiring
  commands are disabled

#### Scenario: The self-test keeps its scratch document [las_selftest_scratch]
- **WHEN** the application is launched with `--self-test` or `--headless` and no
  document argument
- **THEN** the scratch document the bridge checks depend on is created and the
  self-test runs normally

### Requirement: Bridge self-test mode

The system SHALL provide a `--self-test` mode that exercises the Rust↔Qt bridge,
writes a summary to stderr, and exits non-zero when the bridged image is missing.
The self-test SHALL expose each later milestone as an independently addressable
check with a stable exit code, so a failure names the check. The headless
self-test SHALL assert that the application platform is `offscreen` and SHALL
reserve exit code **152** for a platform mismatch; later checks SHALL allocate
codes from **153** upward. The M44 checks SHALL occupy codes **153–166** and the
M45 checks SHALL occupy codes **167–179**. New checks SHALL take the next free
codes from **299** upward and SHALL live in a `selftest_*.cpp` translation unit
rather than growing `selftest.cpp`.

#### Scenario: Self-test succeeds

- **WHEN** the executable is started with `--self-test` and the bridge returns an image
- **THEN** it prints a self-test summary and exits 0

#### Scenario: Self-test fails on a null image

- **WHEN** the bridge returns no image
- **THEN** the self-test prints a failure and exits with a non-zero status

#### Scenario: The headless platform is asserted

- **WHEN** `--headless --self-test` runs while the application platform is not `offscreen`
- **THEN** the self-test prints a failure and exits **152**

#### Scenario: Milestone checks are independently addressable

- **WHEN** an M45 check fails
- **THEN** the self-test exits with that check's code in the range **167–179** and prints which check failed

#### Scenario: New checks use append-only codes [las_codes]

- **WHEN** a new UI check is added for this change
- **THEN** it takes a code at or above **299**, lives in a new `selftest_*.cpp`
  file, and the existing exit codes are unchanged
