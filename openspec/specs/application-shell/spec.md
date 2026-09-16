# application-shell Specification

## Purpose
TBD - created by archiving change m0-walking-skeleton. Update Purpose after archive.
## Requirements
### Requirement: CXX-Qt bridge exposes a Rust QObject to C++
The system SHALL define a Rust `QObject` through a `#[cxx_qt::bridge]` module and
SHALL expose it to the C++ shell using `cxx-qt` 0.10 and `cxx-qt-build`, rather
than hand-written `extern "C"` glue.

#### Scenario: C++ constructs the Rust bridge object
- **WHEN** the C++ shell constructs the Rust-defined `PictureView` object
- **THEN** the generated `QObject` is available to C++ and its methods are callable

#### Scenario: Rust owns the bridge state
- **WHEN** the bridged object is destroyed by C++
- **THEN** its Rust-side state is released without a double free

### Requirement: CMake builds the bridge and the C++ shell
The system SHALL build the application from a top-level `CMakeLists.txt` using
C++17, `find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets)`, and
`cxx_qt_import_crate` (Corrosion) to import the `pictura_app` static library and
link it into the `pictura` executable.

#### Scenario: Configure and build with CMake
- **WHEN** a developer runs `cmake -S . -B build && cmake --build build` with Qt 6 present
- **THEN** CMake imports `pictura_app` and links the `pictura` executable

### Requirement: Qt 6 is discovered through qmake6
The system SHALL locate a Qt 6 `qmake` by preferring `qmake6` over `qmake`,
verifying that `qmake -query QT_VERSION` reports a major version of 6, and SHALL
fail configuration with an error that names the missing package when no Qt 6
`qmake` is found.

#### Scenario: Qt 6 qmake is found
- **WHEN** `qmake6` reports a version starting with `6`
- **THEN** CMake uses it as the Qt discovery executable

#### Scenario: No Qt 6 qmake is available
- **WHEN** neither `qmake6` nor a Qt 6 `qmake` exists on `PATH`
- **THEN** CMake configuration fails with an error naming the Qt 6 development package

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

### Requirement: Bridge self-test mode
The system SHALL provide a `--self-test` mode that exercises the Rust↔Qt bridge,
writes a summary to stderr, and exits non-zero when the bridged image is missing.

#### Scenario: Self-test succeeds
- **WHEN** the executable is started with `--self-test` and the bridge returns an image
- **THEN** it prints a self-test summary and exits 0

#### Scenario: Self-test fails on a null image
- **WHEN** the bridge returns no image
- **THEN** the self-test prints a failure and exits with a non-zero status

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

### Requirement: CS6-style chrome styling

The system SHALL style the application chrome (menu bar, options bar, Tools
panel, panel docks and their tabs and title bars, status bar, tool buttons, and
scrollbars) with a CS6-style dark stylesheet driven by the same brightness ramp
as the palette, and SHALL rebuild the stylesheet when the brightness level
changes. The bar chrome SHALL be a low-contrast dark surface distinct from the
document canvas, and panel tabs SHALL be flat with a distinctly highlighted
active tab.

#### Scenario: Chrome is styled at startup

- **WHEN** the frame is shown
- **THEN** the dock tabs, dock title bars, tool buttons, menu bar, and status bar use the dark stylesheet rather than unstyled Fusion defaults

#### Scenario: Brightness restyles the chrome

- **WHEN** the brightness level changes with `Shift+F1` or `Shift+F2`
- **THEN** the stylesheet is regenerated for the new level and the chrome colours change

### Requirement: Default dock grouping and canvas colour

The system SHALL present the default workspace with the panels grouped into
three CS6 tabbed docks: **Color, Swatches, Gradients, and Patterns**;
**Properties, Adjustments, and Libraries**; and **Layers, Channels, and Paths**.
The document canvas SHALL use the CS6 dark canvas colour, and the document tab
strip SHALL be styled to match the chrome. Panel `objectName`s SHALL remain
stable so `Tab`/`Shift+Tab` and the persisted session layout keep working.

#### Scenario: Panels share tabbed docks

- **WHEN** the frame starts with a fresh session
- **THEN** Color, Swatches, Gradients, and Patterns share one tabbed dock; Properties, Adjustments, and Libraries share one; and Layers, Channels, and Paths share one

#### Scenario: Existing panel behaviour is unchanged

- **WHEN** `Tab` is pressed
- **THEN** all panel docks are hidden and restored as before, including the grouped ones

#### Scenario: Canvas matches the chrome

- **WHEN** the frame is shown
- **THEN** the document view background is the CS6 dark canvas colour

### Requirement: Canvas transparency checkerboard

The document canvas SHALL draw a two-tone checkerboard behind the document image,
clipped to the document rect, so that any pixel with alpha less than 255 reveals
it. A fully transparent document SHALL show the checkerboard, and the
checkerboard SHALL NOT be drawn outside the document rect.

#### Scenario: A transparent document reveals both checker tones

- **WHEN** a fully transparent document is displayed on the canvas
- **THEN** pixels inside the document rect show the two checkerboard tones and
  pixels outside the document rect show the canvas colour

#### Scenario: Opaque content covers the checkerboard

- **WHEN** a document region is fully opaque
- **THEN** that region shows the document pixels and the checkerboard is not
  visible through it

### Requirement: Canvas clips content to the document bounds

The canvas SHALL clip the composited document and the live move preview to the
document rect, and content outside the canvas SHALL NOT be drawn.

#### Scenario: A layer preview dragged outside the document is cropped

- **WHEN** the move preview draws a layer whose position puts part of it outside
  the document rect
- **THEN** the part inside the document rect shows the layer and the part outside
  shows the canvas colour

### Requirement: Checkerboard is screen-space and document-anchored

The checkerboard cell size SHALL be constant in screen space, independent of
zoom, and the checkerboard SHALL be anchored to the document origin so that
panning does not move the pattern relative to the document.

#### Scenario: Zoom does not change the cell size

- **WHEN** the canvas zoom changes
- **THEN** the checkerboard cell size in screen pixels stays the same

#### Scenario: Panning does not shift the pattern

- **WHEN** the canvas is panned
- **THEN** the checkerboard stays aligned to the document origin rather than to
  the viewport

