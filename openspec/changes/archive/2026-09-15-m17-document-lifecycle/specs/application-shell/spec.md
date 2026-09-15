## MODIFIED Requirements

### Requirement: Headless window shell
The system SHALL open a top-level Qt Widgets `QMainWindow` frame containing a
menu bar, a tabbed document area that hosts one canvas per open document, a
status bar, and dock areas, and SHALL run under a headless display server (Xvfb
or the offscreen QPA plugin) without requiring a GPU.

#### Scenario: Run under Xvfb
- **WHEN** the executable is started under `xvfb-run` or with `QT_QPA_PLATFORM=offscreen`
- **THEN** it opens its window and exits without error

#### Scenario: Frame chrome is present at startup
- **WHEN** the window is shown
- **THEN** the menu bar, tabbed document area, status bar, and dock area exist

#### Scenario: No document open
- **WHEN** the frame starts with no document loaded
- **THEN** document-requiring commands are disabled and the application does not crash
