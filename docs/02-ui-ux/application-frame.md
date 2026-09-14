# Application Frame

- **Spec ID:** `UI-001`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the CS6 **Application bar** was removed and the functions it held moved into the menu bar, the panel docks, and a screen-mode button at the bottom of the toolbar; the draggable title bar was reduced by over 30%; the interface gained four brightness levels and on-image displays.
- **Depends on:** `ARCH-003` qt6-ui-design, `UI-002` menus, `UI-003` workspace-and-docks, `UI-004` toolbox-and-options-bar, `ARCH-006` gpu-rendering-pipeline

> All module and widget names below are **design proposals**. No code exists in
> this repository. Facts confirmed by the fetched CS6 Help reference are stated
> plainly; anything reconstructed from secondary sources or inference is marked
> *(inferred)* or `(to verify)`.

## CS6 behavior

### The application frame

Photoshop CS6 groups its workspace elements into an **application frame**: a
single integrated window containing the menu bar, options bar, document
windows, Tools panel, and docks. Moving or resizing the frame moves/resizes its
contents together so panels do not overlap, and panels do not disappear when the
user switches applications or clicks outside the application. On Windows the
frame is always on; on Mac the traditional free-form interface is offered by
some Creative Suite products, but Photoshop's Application bar behaviour on the
Mac is only described generically in the CS6 Help *(inferred for Photoshop
specifically)*.

CS6 specifically **removed the Application bar** that CS5 had across the top.
The CS6 Help "What's New" states that "functions formerly in the application bar
have moved elsewhere" and that the draggable title bar was reduced by over 30%.
The workspace switcher moved into the panel dock area, screen-mode switching
moved to a button at the bottom of the toolbar, and application controls moved to
the menu bar (Windows). `(to verify)` the exact CS5 → CS6 control-by-control
relocation.

### Menu bar (Windows) vs. application menu (Mac)

- The CS6 Help's workspace overview says the **Application bar** contained
  "menus (Windows only)". On **Windows**, the application menus (File … Help)
  live in a menu bar across the top of the frame.
- On **Mac OS**, Photoshop follows the platform convention: a **Photoshop
  application menu** (Photoshop > About, Preferences, Services, Hide, Quit) sits
  in the system menu bar, and product preferences are reached via
  `Photoshop > Preferences` instead of `Edit > Preferences`. The Help writes
  `Choose Edit > Preferences (Windows) or Photoshop > Preferences (Mac OS)`
  throughout, confirming the split.
- The user-facing spec convention for Kooka Pictura: expose menu definitions
  once, with a platform mapping layer (see `UI-002`). On Linux there is no
  application menu, so the Windows-style menu bar is the model, with
  `Edit > Preferences` (or a first-class `Preferences` entry in an app menu if
  the desktop uses a global menu bar).

### Document windows and tabs

- When more than one file is open, document windows are **tabbed** by default.
  Drag a tab to reorder; drag a tab out of the group to untab/float it.
- `Window > Arrange > Float in Window` floats one document; `Window > Arrange >
  Float All In Windows` floats all. `Window > Arrange` also has `Tile`,
  `Consolidate All to Tabs`, `Match Zoom`, `Match Location`, `Match All`, and
  `New Window For [file]`.
- Documents can be grouped and docked; CS6 also supports **stacked/tiled
  groups** of documents via drop zones along a window's top/bottom/sides, with a
  Layout button on the (CS5) Application bar.
- New in CS6 on Windows: **New/Open document commands by context-clicking
  document tabs** (previously Mac-only).

### Title bar

- Every document window has a title bar showing the document name and, for the
  active layer, the active layer name ("The name of the active layer appears in
  the title bar of the document").
- CS6 reduced the draggable title bar height by over 30% to reclaim screen
  space. Exact pixel metric is `(to verify)`; capture from a CS6 screenshot.

### Status bar

- The **status bar** sits at the bottom of every document window. It shows
  current magnification and file size of the active image, plus brief
  instructions for the active tool.
- Clicking the triangle in its bottom border opens a view-options menu:
  Version Cue, Document Sizes, Document Profile, Document Dimensions,
  Measurement Scale, Scratch Sizes, Efficiency, Timing, Current Tool, 32-bit
  Exposure (HDR only).
- Clicking the file-information area shows width, height, channels, resolution;
  Control-click (Windows) / Command-click (Mac) shows tile width and height.
- If a Digimarc watermark is detected, a copyright symbol is shown in the
  window title bar.

### Dark CS6 interface

- CS6 introduced a darker UI than CS5. The Interface preferences offer **four
  brightness levels** ("Select from four different brightness levels"). Exact
  swatch values are `(to verify)`.
- Quick brightness shortcuts: the CS6 "What's New" says **Shift+F1** decreases
  and **Shift+F2** increases brightness (Mac laptops also press `Fn`). The
  shared workspace chapter separately says **Shift+1 / Shift+2**; the two are
  inconsistent in the same Help file. Treat Shift+F1/F2 as the CS6 Photoshop
  binding *(inferred? see Open questions)*.
- Other Interface-preference knobs relevant to the frame: `UI Font Size`
  (panel/tool-tip text), `Show Tool Tips`, `Show Menu Colors`,
  `Auto-Show Hidden Panels`, `Auto-Collapse Icon Panels`, `Restore Default
  Workspaces`.
- On-image displays (HUDs for selection dimensions, transform angles) and their
  placement via `Show Transformation Values` are part of the CS6 look.

### Screen modes and full-screen

CS6 has three screen modes, cycled with `F` (forward) and `Shift+F` (backward):

| Mode | Chrome | Background |
|---|---|---|
| Standard Screen Mode | menu bar + title bar + scroll bars | normal |
| Full Screen Mode With Menu Bar | menu bar only; no title bar, no scroll bars | 50% gray |
| Full Screen Mode | no menu bar, title bar, or scroll bars | black |

Also accessible via `View > Screen Mode > …` and a **Screen Mode button**.
In CS6 the button lives at the bottom of the toolbar; in CS5 it was in the
Application bar. Canvas colour can be cycled with `Space+F` (forward) and
`Space+Shift+F` (backward) or via right-click on the canvas background.

### Floating vs. docked windows

- Panels may be **docked** (in a dock, generally vertical) or **floating**
  (free-floating, optionally stacked). A document window may be tabbed, floated,
  or placed into a group. Full details live in `UI-003`.
- On Windows the whole app is one top-level frame; floated panels are child
  top-level windows. Taskbar behavior on Linux is compositor-defined; CS6's
  Windows taskbar behavior for floated documents is not documented and is left
  to `## Open questions`.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Application frame | Main window | n/a | Single integrated top-level window |
| Menu bar | Menu bar | `Alt`/`F10` *(platform)* | Windows-style; Mac uses app menu |
| Application menu | Menu (Mac) | n/a | Photoshop > Preferences/About/Quit; Linux mapping TBD |
| Document tab strip | Tab bar | `Ctrl+Tab`, `Shift+Ctrl+Tab` | One tab per open document |
| Document window | MDI child / top-level | n/a | Tabbed, docked, or floated |
| Canvas | Central widget | see `TOOL-042` | GPU-composited |
| Title bar | Window chrome | n/a | Shows doc name + active layer name |
| Status bar | Window bottom bar | click triangle | Magnification, file size, view options |
| Scren Mode button | Toolbar bottom button | `F` / `Shift+F` | Cycles screen modes |
| Brightness control | Preference | `Shift+F1`/`Shift+F2` `(to verify)` | Four brightness levels |
| Place/Edit guides | Canvas overlay | `Ctrl+R`, drag rulers | `View > Rulers`, `View > New Guide` |
| Window > Arrange | Menu | n/a | Tile/Cascade/Float/Match/New Window |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Brightness level | enum | CS6 default (darkest?) | 4 levels | Exact values `(to verify)` |
| UI font size | enum | Medium `(inferred)` | Small / Medium / Large? `(to verify)` | Interface preferences |
| Title bar height | px (DIP) | reduced in CS6 | `(to verify)` | >30% shorter than CS5 |
| Document tab height | px (DIP) | `(to verify)` | — | Theme token |
| Panel width | px (DIP) | theme default | 200–480 `(inferred)` | Per panel, persisted |
| Screen mode | enum | Standard | Standard / Full Screen With Menu Bar / Full Screen | Cycles with `F` |
| Full-screen background | colour | 50% gray / black | per mode | `Space+F` cycles canvas colour |
| Floating window geometry | screen rect | — | clamped to available geometry | Persist per workspace |
| Status bar view | enum | Document Sizes? `(inferred)` | 10 options | Per-document/session |
| Auto-Show Hidden Panels | bool | Off `(inferred)` | on/off | Interface preferences |
| Auto-Collapse Icon Panels | bool | Off `(inferred)` | on/off | Interface preferences |

## Algorithms & pipeline

Frame composition and window management. All names here are design proposals.

- **Frame = one `QMainWindow`.** Menu bar, options bar (`QToolBar`), status bar
  (`QStatusBar`), central canvas stack, and four dock areas.
- **Document area = a tabbed MDI stack, not an OS-MDI.** Each document is a
  `CanvasWidget` hosted in a tab (`QTabWidget`) or, when floated, a top-level
  window. This mirrors the CS6 tab/float model and avoids platform MDI quirks.
- **Screen-mode state machine.** `ScreenMode ∈ {Standard, FullWithMenuBar,
  Full}`. Transition hides/shows menu bar, title bar, status bar, scroll bars
  and sets the canvas backdrop; `F` is forward, `Shift+F` backward. Canvas colour
  is an independent cycle (`Space+F`).
- **Application frame on Linux.** There is no application menu; use a menu bar.
  Wayland compositors own window placement, so "float in window" becomes "make
  a new top-level document window"; absolute positioning is not assumed.
- **Status bar refresh.** Magnification, document size, scratch size, efficiency
  and timing are sampled from the renderer and allocator; updates are throttled
  to avoid per-frame churn *(inferred)*.
- **Theme.** One `Theme` object is the single source of truth for the four
  brightness levels, metrics and colours (see `ARCH-003`). No widget hard-codes a
  colour.

## Rust module mapping

Proposals; the shell is largely Qt/C++ with Rust supplying state.

- `pictura_ui::shell` — frame setup, menu/dock registration, screen-mode state,
  workspace save/restore.
- `pictura_ui::frame::ScreenMode` — `enum ScreenMode`, transition table, and
  `apply(mode)` intent emitted to Qt.
- `pictura_ui::frame::window_store` — per-document/per-window view state
  (`ViewportState` from `TOOL-042`), tab order, float state.
- `pictura_ui::theme` — brightness level → token table (colours, metrics).
- `pictura_ui::status` — computed status fields (`DocumentSize`, `ScratchSize`,
  `Efficiency`, `Timing`, `CurrentTool`, `HdrExposure`).

Crossing types: `ScreenMode`, `BrightnessLevel`, `WindowId`, small value
structs only. No image buffers cross this boundary.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `PicturaMainWindow` | `QMainWindow` | Frame, menu bar, options `QToolBar`, `QStatusBar`, dock areas, `saveState()`/`restoreState()` |
| `DocumentTabBar` | `QTabBar`/`QTabWidget` | Tab strip, reorder, drag-out-to-float, context menu (New/Open on Windows parity) |
| `CanvasWidget` | `QRhiWidget` (proposed) | GPU document surface, hosted in tab or float window |
| `StatusBar` | `QStatusBar` | Magnification/size/efficiency fields + view-options popup |
| `ScreenModeController` | `QObject` | Applies `ScreenMode`; binds `F`/`Shift+F`/`Space+F` |
| `BrightnessController` | `QObject` | Applies the four theme levels; binds brightness shortcuts |
| `Theme` | `QObject` (tokens) | Single source of truth for colours/metrics |

Widgets over QML for the shell, per `ARCH-003`. The frame is dense, keyboard-
centric and dock-heavy; QML's costs are not justified here.

## Data-model impact

- Frame state is **UI/session state, not document content**: window geometry,
  tab order, float state, screen mode, canvas colour, brightness, status-bar
  view. Persisted in the workspace/session store (`UI-003`,
  `11-cross-cutting/preference-storage.md`), never in PSD/XMP.
- Whether CS6 writes per-document view metadata (zoom/scroll/screen mode) into
  the PSD is *unverified* (`TOOL-042` open question).
- No undo records are produced by frame operations (resize, tab move, screen
  mode). `Ctrl+Z` must not revert them *(inferred)*.
- Status-bar values are derived from the document/renderer/allocator and are
  never serialized.

## Edge cases

- **Wayland.** No absolute window positioning; floating documents become
  compositor-managed top-levels and restore by relative layout only.
- **Multi-monitor.** Floated windows must be clamped to available screen
  geometry on restore; screen geometry is in DIP with scaling gaps.
- **High-DPI change at runtime.** Moving the frame between displays changes
  `devicePixelRatio`; the canvas buffer and icon assets must rebuild.
- **Full-screen on tiled compositors.** Full Screen Mode is compositor-
  controlled on Linux and may not be able to hide all chrome or may be refused.
- **GPU unavailable.** The shell and status bar must still start; canvas falls
  back to CPU compositing (`ARCH-006`).
- **Zero documents open.** Frame shows an empty canvas (or Home screen if
  adopted); menus that require a document are disabled.
- **Many documents.** Tab strip must scroll/overflow; float-all on a small
  screen must not produce unreachable windows.
- **Mac application menu semantics.** `Quit`, `About`, `Services`, `Hide` have
  no exact Linux equivalent; mapping needs a decision.
- **Taskbar behavior.** CS6's Windows grouping of floated documents is
  undocumented; Linux DE/taskbar behavior must be specified per environment.

## Parity acceptance criteria

- Given no document is open, then opening one adds a tab; opening a second adds
  a second tab; `Window > Arrange > Float All In Windows` re-parents every tab
  into its own top-level window.
- Given Standard mode, pressing `F` yields Full Screen With Menu Bar (menu bar
  only, 50% gray), pressing `F` again yields Full Screen (black, no chrome), and
  `Shift+F` reverses the sequence.
- Given Full Screen Mode, `Space+F` cycles the canvas background colour and
  `Space+Shift+F` reverses it.
- Given a document with a Digimarc-detected watermark, the copyright symbol
  appears in the title bar `(contingency; requires watermark plug-in)`.
- Given the status-bar triangle is clicked, the documented view-option list
  appears and selecting `Document Sizes` shows the two size figures.
- Given the Interface preferences, selecting each of the four brightness levels
  changes the whole frame chrome and canvas surround consistently, and the
  choice survives restart.
- Given `Shift+F1`/`Shift+F2` (or the verified CS6 binding), interface
  brightness steps down/up and is reflected immediately.
- Given a workspace is saved with floated panels, quit and relaunch restores
  tab order, float state, dock layout and screen mode within one pixel at the
  same scale factor.
- Given a two-document tab group, `Ctrl+Tab` cycles documents and
  `Shift+Ctrl+Tab` cycles backward.
- Given the frame is resized, all docked panels reflow without overlap and the
  canvas takes the remaining area.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference (downloaded and text-extracted).
  Establishes: application frame definition and behavior; Application bar
  contains "menus (Windows only)"; tabbed document windows and
  float-in/floats-all; `Window > Arrange` commands; title-bar shows active layer
  name; status bar location/contents and its view-option list; screen modes and
  `F`/`Shift+F`/`Space+F` cycling; the Screen Mode button; CS6 removal of the
  app bar and >30% title-bar reduction; four brightness levels and
  `Shift+F1`/`Shift+F2`; Interface preference items (UI Font Size, Show Tool
  Tips, Show Menu Colors, Auto-Show Hidden Panels, Auto-Collapse Icon Panels,
  Restore Default Workspaces); Windows-only New/Open on document-tab
  context menu.
- `https://itwiki.wpunj.edu/images/e/ee/Photoshop_CS6_Extended_-_Dacier.pdf` —
  secondary student tutorial. Corroborates the window's named regions (Menu Bar,
  Options Bar, Tools Palette, Document Window, collapsed palette bar, palette
  dock, layer options bar). Secondary source.
- `https://www.photoshopessentials.com/basics/photoshop-cs6-workspaces` —
  secondary CS6 tutorial. Corroborates the panel columns, workspace switcher
  location, and workspace reset behavior. Secondary source.
- SearXNG meta-search queries used to locate secondary sources (workspace
  presets; CS6 menu/window references). No facts taken from snippets alone.

Not parsed: `helpx.adobe.com` (HTTP 403 from this environment).

## Open questions

- **CS6 brightness shortcut conflict.** The CS6 "What's New" says
  `Shift+F1`/`Shift+F2`; the shared workspace chapter says `Shift+1`/`Shift+2`.
  Resolve with a CS6 keybinding capture or Adobe release notes.
- **Exact brightness palette values and metrics.** The four level colours, panel
  background, border and text colours, title-bar and tab heights are not
  specified. Resolve with a CS6 screenshot colour sample and pixel metrics.
- **Mac application-menu behavior for Photoshop CS6.** Whether Photoshop CS6 had
  a free-form (non-frame) mode on the Mac, and how its application menu was
  populated, is only generically sourced. Resolve with the Mac-specific CS6 Help
  or a capture.
- **Taskbar behavior for floated documents on Windows.** Not documented. Resolve
  with a CS6-on-Windows capture; then define the Linux DE mapping.
- **Screen mode vs. window manager on Linux.** Full-screen chrome hiding under
  X11 vs Wayland tiling compositors needs a decision. Resolve in `UI-003` /
  platform integration.
- **`UI Font Size` option set.** The Help mentions the setting but not its
  values. Resolve from the Interface preferences screenshot.
- **Status-bar default view option.** Only the option list is sourced; the
  default field is inferred. Resolve with a first-run CS6 capture.
