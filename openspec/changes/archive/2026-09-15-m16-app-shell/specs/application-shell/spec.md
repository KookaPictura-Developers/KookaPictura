## MODIFIED Requirements

### Requirement: Headless window shell
The system SHALL open a top-level Qt Widgets `QMainWindow` frame containing a
menu bar, a central image canvas, a status bar, and dock areas, and SHALL run
under a headless display server (Xvfb or the offscreen QPA plugin) without
requiring a GPU.

#### Scenario: Run under Xvfb
- **WHEN** the executable is started under `xvfb-run` or with `QT_QPA_PLATFORM=offscreen`
- **THEN** it opens its window and exits without error

#### Scenario: Frame chrome is present at startup
- **WHEN** the window is shown
- **THEN** the menu bar, central canvas, status bar, and dock area exist

#### Scenario: No document open
- **WHEN** the frame starts with no document loaded
- **THEN** document-requiring commands are disabled and the application does not crash

## ADDED Requirements

### Requirement: Interface theme with brightness levels
The system SHALL apply a dark application theme through a single `Theme` unit as
the source of truth, and SHALL offer four interface brightness levels applied at
runtime. Brightness SHALL step down and up with `Shift+F1` and `Shift+F2`, and
the selected level SHALL persist across restart.

#### Scenario: Theme applies at startup
- **WHEN** the frame is shown
- **THEN** the frame chrome uses the dark theme rather than the platform default

#### Scenario: Brightness shortcut steps the level
- **WHEN** `Shift+F2` is pressed
- **THEN** the interface brightness increases by one level and is reflected immediately

#### Scenario: Brightness persists
- **WHEN** the brightness level is changed and the application restarts
- **THEN** the selected level is restored

### Requirement: Screen modes and canvas colour
The system SHALL support three screen modes -- Standard, Full Screen With Menu
Bar, and Full Screen -- cycled forward with `F` and backward with `Shift+F`, and
SHALL cycle the canvas background colour with `Space+F`.

#### Scenario: Cycle screen modes
- **WHEN** `F` is pressed from Standard mode
- **THEN** the frame enters Full Screen With Menu Bar, and pressing `F` again enters Full Screen

#### Scenario: Reverse the cycle
- **WHEN** `Shift+F` is pressed
- **THEN** the screen mode steps backward through the cycle

#### Scenario: Cycle canvas colour
- **WHEN** `Space+F` is pressed
- **THEN** the canvas background colour advances to the next value

### Requirement: Status bar readouts
The system SHALL show the active document's magnification and file size in a
status bar, along with a tool-hint field, and SHALL expose a view-options popup
from the status bar for selecting which readout is displayed.

#### Scenario: Magnification and size shown
- **WHEN** a document is displayed
- **THEN** the status bar shows the current magnification and document size

#### Scenario: View options popup
- **WHEN** the status-bar options control is activated
- **THEN** a popup lists the selectable readouts and the chosen readout is displayed
