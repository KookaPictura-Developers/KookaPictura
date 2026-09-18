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

### Requirement: Bridge self-test mode

The system SHALL provide a `--self-test` mode that exercises the Rust↔Qt bridge,
writes a summary to stderr, and exits non-zero when the bridged image is missing.
The self-test SHALL expose each later milestone as an independently addressable
check with a stable exit code, so a failure names the check. The headless
self-test SHALL assert that the application platform is `offscreen` and SHALL
reserve exit code **152** for a platform mismatch; later checks SHALL allocate
codes from **153** upward. The M44 checks SHALL occupy codes **153–166** and the
M45 checks SHALL occupy codes **167–179**.

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
active tab. The Tools toolbar and the normal and compact widget panels SHALL
draw a darker grey border from the shared theme, and a widget SHALL use the same
colour scheme and style whether docked, shown in a compact flyout, or floating.
The document tab strip SHALL draw a dark grey border on its right side and SHALL
NOT draw an extra top border, because the options bar above it already draws a
dark grey bottom border. Every new rule SHALL be scoped to the panel family and
the document tab strip so the rest of the chrome is unaffected.

#### Scenario: Chrome is styled at startup

- **WHEN** the frame is shown
- **THEN** the dock tabs, dock title bars, tool buttons, menu bar, and status bar use the dark stylesheet rather than unstyled Fusion defaults

#### Scenario: Brightness restyles the chrome

- **WHEN** the brightness level changes with `Shift+F1` or `Shift+F2`
- **THEN** the stylesheet is regenerated for the new level and the chrome colours change

#### Scenario: Panels draw the darker grey border [m44_panelborder]

- **WHEN** the Tools toolbar and a normal- or compact-mode widget panel are shown
- **THEN** each draws a darker grey border from the shared theme

#### Scenario: The document tab strip has only a right border [m44_filebar]

- **WHEN** the document tab strip is inspected
- **THEN** it draws a dark grey border on its right side and no extra top border

#### Scenario: A widget looks the same in all three presentations [m44_popupstyle]

- **WHEN** the same widget is docked, shown in a compact flyout, and floating
- **THEN** all three present the same background, borders, and tab-bar styling

### Requirement: Default dock grouping and canvas colour

The system SHALL present the default workspace with the right-hand panels
hosted by the `PanelColumn` in the CS6 Essentials groups: **Color, Swatches, and
Styles**; **Adjustments**; **Layers, Channels, and Paths**; **Navigator,
Histogram, and Info**; and the iconic **History** and **Actions**. The central
area SHALL be a horizontal splitter hosting an ordered set of `PanelColumn`s
around the document tab area: zero or more columns to the left of the document
tabs and zero or more to the right, with the document tabs keeping the stretch.
A `PanelColumn` SHALL be creatable dynamically by a drop and removed when
emptied. The document canvas SHALL use the CS6 dark canvas colour, and the
document tab strip SHALL be styled to match the chrome. Panel `objectName`s SHALL
remain stable so the persisted session layout keeps working. The Tools panel
SHALL remain a left/right dock separate from the columns.

#### Scenario: Panels form the CS6 Essentials groups

- **WHEN** the frame starts with a fresh session
- **THEN** Color, Swatches, and Styles share one group; Adjustments is its own
  group; Layers, Channels, and Paths share one; Navigator, Histogram, and Info
  share one; and History and Actions are iconic

#### Scenario: Existing panel behaviour is unchanged

- **WHEN** a panel is shown or hidden from the `Window > Panels` menu
- **THEN** its visibility toggles as before, including the grouped panels

#### Scenario: Canvas matches the chrome

- **WHEN** the frame is shown
- **THEN** the document view background is the CS6 dark canvas colour

#### Scenario: The document tabs keep the stretch [m43_newcolumn]

- **WHEN** a new panel column is created on either side
- **THEN** the document tab area keeps the stretch and the new column takes only
  its own width

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

### Requirement: Toolbox catalogue and flyout groups

The system SHALL define a frozen toolbox catalogue of the CS6 tools in
single-column flyout slots, and SHALL show every catalogue tool, implemented or
not, in its slot's flyout. Each catalogue entry SHALL carry a stable asset id,
a display label, a shortcut (or none), its flyout group and slot, whether it is
implemented, a `Qt::CursorShape` fallback, and a cursor hotspot. The toolbox
SHALL present one button per slot, SHALL mark a slot with more than one member
with a corner triangle, and SHALL reveal the slot's members on hold (with
`Alt`-click or the slot shortcut cycling the enabled members). The slot's
visible tool SHALL be its last-used member. A tool that is not implemented SHALL
be shown disabled with the tooltip `<label> — not implemented yet`. The 10
implemented tools SHALL keep their existing shortcut, checked state, cursor, and
canvas behaviour.

#### Scenario: The catalogue is complete

- **WHEN** the toolbox catalogue is enumerated
- **THEN** it contains 71 tools in 23 slots, with exactly 10 marked implemented

#### Scenario: A slot with hidden tools reveals them

- **WHEN** the user holds the mouse on a slot whose group has more than one member
- **THEN** a flyout lists every member of that group

#### Scenario: An unimplemented tool is visible but disabled

- **WHEN** the flyout of a group containing a tool with no engine is opened
- **THEN** that tool's entry is disabled and its tooltip is `<label> — not implemented yet`

#### Scenario: An implemented tool is selected from a flyout

- **WHEN** the user picks an implemented tool from a flyout
- **THEN** that tool becomes active and the slot shows its icon as the last-used member

#### Scenario: Cycling a slot's enabled members

- **WHEN** the user `Alt`-clicks a slot, or presses `Shift` plus the slot's shortcut
- **THEN** the active tool advances to the next enabled member of that group, wrapping deterministically

#### Scenario: Implemented tools behave as before

- **WHEN** an implemented tool is activated by shortcut or by clicking its button
- **THEN** it is marked active, its cursor is applied, and its options bar is shown, exactly as before this change

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be allowed in the main-window dock areas only on the left
and right sides of the workspace — not on the top or bottom — and SHALL support
being moved, floated, and closed, but SHALL NOT be grouped with other panels in
a tab group. A drop of the Tools panel onto a tab bar SHALL NOT tabify it; when
a drop still results in tabification, the frame SHALL re-dock the panel to its
previous area as a fallback. When floated, the panel SHALL size to the minimum
height its content needs rather than expanding to fill the window, and that
height SHALL NOT be drag-resizable. The panel's custom title bar SHALL remain
draggable so the panel can be moved and floated. The panel's content size SHALL
be fixed along the dock's major axis: a fixed content width for a left or right
dock. Dragging the dock separator SHALL NOT resize it. The panel SHALL be
placeable on any side of any widget panel or column, wherever the columns are
docked — to the left of a right column, between two columns, or at the outer
edge — through the same column drop grammar and the same single insertion
indicator the widget columns use, without breaking the fixed-size rule or the
no-tabification contract.

#### Scenario: The panel docks only on the left or right [m45_tools_sides]

- **WHEN** the Tools panel's allowed areas are queried
- **THEN** only the left and right main-window dock areas are permitted, and the
  top and bottom areas are refused

#### Scenario: The panel can float [m40_dock]

- **WHEN** the Tools panel is dragged out of its dock area
- **THEN** it floats as an independent window and can be docked back to a side

#### Scenario: The floated dock hugs its content height [m42_tools]

- **WHEN** the Tools panel is floated
- **THEN** its height is the minimum its content needs and it does not expand to
  fill the window

#### Scenario: The floated height cannot be dragged [m44_toolsfloat]

- **WHEN** a resize is attempted on the floating Tools panel
- **THEN** it keeps its fixed content height and does not change

#### Scenario: Tabification is refused [m40_dock]

- **WHEN** the Tools panel is dropped onto another panel's tab bar
- **THEN** it does not become a tab in that group

#### Scenario: A tabified drop falls back to a side dock [m40_dock]

- **WHEN** a drop nonetheless leaves the Tools panel tabified with another panel
- **THEN** the frame re-docks it to its previous dock area

#### Scenario: The width cannot be dragged [m43_tools]

- **WHEN** the dock separator beside the Tools panel is dragged
- **THEN** the Tools panel's width does not change and stays at its content width

#### Scenario: The panel docks beside a widget column [m45_tools_beside_column]

- **WHEN** the floating Tools panel is dragged to a side of a widget column,
  including the left of a right-hand column or between two columns
- **THEN** the single blue indicator marks that boundary and the panel is placed
  there without being tabified and without losing its fixed content width

### Requirement: Menu bar is not overlaid

The application shell SHALL keep the menu-bar row clear of every other widget: no
toolbar, dock title bar, floating overlay, or stray child widget SHALL be drawn
over the menu bar. A persisted layout that no longer matches the current chrome
SHALL be discarded rather than restoring a widget over the menu bar.

#### Scenario: Nothing overlays the menu bar [m42_menubar]

- **WHEN** the frame is shown in its default and restored arrangements
- **THEN** no child widget's global geometry intersects the menu-bar row and only
  the menu bar is drawn there

#### Scenario: A stale layout is discarded [m42_menubar]

- **WHEN** a persisted layout that references chrome no longer present is
  restored
- **THEN** it is discarded in favour of the default arrangement and no widget is
  drawn over the menu bar

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

### Requirement: Central splitter keeps every pane

The application shell SHALL keep every pane of its central splitter — the
ordered set of left widget columns, the document tab area, and right widget
columns — at or above its minimum size, and SHALL NOT allow a handle drag to
collapse a pane to zero. The splitter SHALL clamp a handle drag at the pane's
minimum size, and a widget column SHALL remain visible at its minimum floor
instead of disappearing. The per-column group splitter SHALL enforce the same
non-collapsible invariant for its panel groups.

#### Scenario: Dragging the central splitter handle cannot hide a pane [m46_center_splitter]

- **WHEN** the central splitter handle is dragged past a widget column's minimum
  width
- **THEN** the column is clamped at its minimum and stays visible rather than
  collapsing to zero

#### Scenario: The group splitter cannot collapse a group to zero [m46_no_collapse]

- **WHEN** a column's group splitter handle is dragged past a panel group's
  minimum height
- **THEN** the group is clamped at its minimum and its tab bar stays visible

### Requirement: Central splitter hosts the Tools pane

The application shell SHALL allow the Tools panel to be re-hosted from its dock
into the central splitter as a fixed-width pane, on either side of a widget
column or between two widget columns, through the same column drop grammar and
indicator the widget columns use. While it is a splitter pane the Tools panel
SHALL keep its fixed content width, fill the splitter height, remain re-draggable
by its title bar, and SHALL NOT be tabified. A docked or pane-hosted Tools panel
SHALL NOT block a floating widget overlay from being dragged across it.

#### Scenario: The Tools panel is hosted between columns [m47_tools_pane]

- **WHEN** the floating Tools panel is dropped between two widget columns
- **THEN** it becomes a fixed-width splitter pane at that boundary and is not
  tabified

#### Scenario: A widget overlay crosses the Tools pane [m47_float_over_tools]

- **WHEN** a widget overlay is dragged over the Tools panel
- **THEN** it continues to follow the cursor instead of stopping at the central
  area edge

### Requirement: Compact group chrome shading

The compact/iconic group container SHALL use the panel surface shade rather than
the darker base shade, and its drag-handle dots SHALL be dark gray, distinct
from the near-white window text, so the dots read as a handle.

#### Scenario: Compact group background and dots [m47_compact_shade]

- **WHEN** the compact strip is built
- **THEN** each group container uses the panel surface shade and its drag dots
  render in dark gray

