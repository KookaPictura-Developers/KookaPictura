# M40 — Tools panel (CS6 flyouts, columns, dock, Shift cycling)

- **Status:** proposed (`openspec/changes/m40-tools-panel`); not implemented.
- **Type:** a user-requested Tools-panel milestone that pairs with the M41
  Preferences dialog. It was requested between the Layers-panel program's
  panel-anatomy milestone (M39, done) and its filtering/search stage; the Layers
  program therefore shifts by two behind this pair (see §6).
- **Contract:** `docs/02-ui-ux/toolbox-and-options-bar.md` (`UI-004`) is the CS6
  toolbox spec; `docs/02-ui-ux/workspace-and-docks.md` (`UI-003`) is the dock
  contract; `docs/02-ui-ux/preferences.md` (`UI-010`, General pane) fixes
  `Use Shift Key For Tool Switch` default **on**. This file freezes the
  interfaces those specs leave open. Canonical requirements:
  `openspec/specs/tools/tool-framework/spec.md`,
  `openspec/specs/ui/application-shell/spec.md`.
- **Consumers:** `crates/pictura-app/cpp/toolbox.{h,cpp}`,
  `crates/pictura-app/cpp/tools.{h,cpp}`,
  `crates/pictura-app/cpp/frame.{h,cpp}`,
  `crates/pictura-app/cpp/session.{h,cpp}`,
  `crates/pictura-app/cpp/main.cpp`, and the two new asset files.
- **Non-goals:** the panel dock/rail refactor, compact/iconic docks, popup
  flyouts for panels, `Auto-Collapse Iconic Panels`, the Preferences dialog,
  temporary (spring-loaded) tool switching, tool-preset pickers, and any change
  to the 71-tool catalogue or the 10-tool implemented set.

## 1. CS6 behavior (cited)

- **The top double arrow toggles one and two columns.** CS6 starts in a
  **single column**; the **double-arrow button in the bar at the very top** of
  the Tools panel shows two columns side by side, and a second click returns to
  one (teachucomp, agitraining, the official CS6 Project 1 workspace PDF;
  `UI-004` lines 20–22, 155–157, 177).
- **The Tools panel is special.** "Unlike most panels … you cannot dock the
  Tools panel at the bottom … or group it with other panels in panel tab groups"
  (teachucomp); it floats and docks to either side (`UI-003` treats the panel
  rules generally and names the Tools panel as the special case; the `Tab`/
  `Shift+Tab` rules keep it with the options bar, `UI-003` lines 136–137).
- **The lower-right triangle marks hidden tools.** A small **triangle at a
  tool's lower-right corner** means hidden tools are available; **click-and-hold**
  or **right-click** opens the flyout, and `Alt`-click cycles the hidden tools —
  except Add Anchor Point, Delete Anchor Point, Convert Point (`UI-004` lines
  22–25; agitraining, photoshoptrainingchannel).
- **`Shift`+letter cycles hidden tools.** "By default `Shift` + repeatedly
  pressing the shortcut cycles the group. Disable `General > Use Shift Key For
  Tool Switch` to cycle without `Shift`." (`UI-004` lines 31–33; manualzz CS6
  Tools, howtogeek; `UI-010` line 78 sets the default **on**).
- **Temporary tool switching** (hold the letter to switch while held) is a real
  CS6 behavior (`UI-004` lines 29–30) but is **out of scope** for M40.

## 2. Current state (inventory)

- `crates/pictura-app/cpp/toolbox.{h,cpp}`:
  - `ToolSlotButton` (`toolbox.cpp:31–88`): a `QToolButton` with a
    `QTimer holdTimer_` at **350 ms** (`:37`), `onMenu`/`onCycle` callbacks,
    right-click opens the menu (`:52–57`), `Alt`-click cycles (`:59–66`), a
    left press starts the timer (`:67–70`), and a release after the timeout is
    consumed without a click (`:74–83`).
  - `Toolbox` (`toolbox.h:48–70`, `toolbox.cpp:193–299`): `QDockWidget`
    `objectName` `toolsPanel`; one `ToolSlotButton` per group 1..23 in a single
    `QVBoxLayout` column (`:205–268`); `MenuButtonPopup` plus a `QMenu` for
    multi-member groups (`:234–249`); `slotButtons()` accessor (`toolbox.h:55`);
    `setMinimumWidth(66)` (`:290`); a `ForegroundBackgroundWidget` and a
    `screenModeButton` after the column (`:271–286`).
  - `refreshSlot()` (`:323–344`) sets the icon/tooltip/checked state and a
    per-button `setShortcut()` for implemented slots **except Brush/Pencil**
    (`:334–338`). `groupCurrentTool()`/`selectMember()`/`cycleGroup()` are the
    slot helpers (`:301–368`); `cycleGroup` already skips unimplemented members.
- `crates/pictura-app/cpp/tools.{h,cpp}`: the frozen 71-tool catalogue
  (`ToolId` `tools.h:23–95`, `ToolInfo` `:98–109`, `kToolTable`
  `tools.cpp:14–157`), `toolInfo` (`:165`), `allToolIds` (`:180`),
  `implementedToolIds` (`:211`), `toolImplemented` (`:171`), `toolIdName`
  (`:173`). Each entry carries `group` 1..23 and `shortcut` (some `QChar()`
  none); every distinct letter maps to exactly one group in the current table.
- `crates/pictura-app/cpp/frame.{h,cpp}`: `setDockOptions(AnimatedDocks |
  AllowTabbedDocks)` (`frame.cpp:72`); `registerPanel()` (`:163–171`);
  `buildTools()` (`:870–950`) creates the `Toolbox` (`:924–926`), the
  `OptionsBar`, and the hard-coded `cyclePaintTool` lambda with the `B` and
  `Shift+B` shortcuts (`:884–893`; brush-size/hardness brackets `:895–922`).
  `toolsDock_` is a `QDockWidget*` (`frame.h:155`).
- `crates/pictura-app/cpp/session.{h,cpp}`: `SessionState` at
  `schemaVersion = 3` (`session.h:10–19`) with `layout`, `brightnessLevel`,
  `gpuCompute`, the three Layers fields, and `recent`; `loadSession()` reads each
  field with a default (`session.cpp:40–56`); `saveSession()` writes JSON through
  `QSaveFile` (`:59–86`). M39 made `saveSession()` load before writing.
- `crates/pictura-app/cpp/icons.{h,cpp}`: `icon(id)` resolves
  `:/icons/<id>.svg` and returns a null `QIcon` for a missing asset
  (`icons.cpp:14–21`). `assets/pictura.qrc` lists every icon file;
  `assets/icons/tool.move.svg` is the 24×24 stroke-`#c8c8c8` style reference.
- **No** custom title bar, column toggle, flyout keys, dock area/feature
  constraint, tabify defence, generic letter handler, or `panel.columns*` assets
  exist today.
- Self-test state at proposal time: `m38_tools` (exit 95–98, `main.cpp:2610–2659`)
  reads `slotButtons()` and checks 23 slots / implemented enablement; `m23_toolbox`
  (exit 59–61, `main.cpp:1904`); the M39 checks run through exit code 112. The
  highest return code is 112, so M40's new steps use 113+.

## 3. Frozen interfaces

### 3.1 Flyout indicator

`ToolSlotButton::paintEvent` draws the base button, then a small filled triangle
in the button's lower-right corner iff `members(group).size() >= 2`, counting
unimplemented members. No `MenuButtonPopup`; the icon stays centred.

### 3.2 Flyout opening

Hold timer **300 ms**. On timeout:
`menu->popup(button->mapToGlobal(QPoint(0, button->height())))`, clamped to
`screen()->availableGeometry()`. Right-click opens immediately. A left release
before the timeout stops the timer and runs the normal activation. A
single-member slot has no menu.

### 3.3 Flyout shortcut keys

Each `QAction`: `setShortcut(slotKey)`, `setShortcutVisibleInContextMenu(true)`,
`setShortcutContext(Qt::WidgetWithChildrenShortcut)`. Disabled items keep the
exact `<label> — not implemented yet` tooltip. `refreshSlot()` no longer calls
`setShortcut()`; the frame owns every letter.

### 3.4 Column layout

Custom `setTitleBarWidget` with a `Tools` label and a double-arrow button
showing the **target** layout (`panel.columnsTwo` in one column,
`panel.columnsOne` in two). Two columns reflow the 23 slots **row-major** in the
`QGridLayout` (`(i / 2, i % 2)`); one column is `(i, 0)`. The dock minimum is
66 px in one column and two button widths plus margins in two. The
foreground/background widget and the Screen Mode button stay below the grid.

### 3.5 Dock constraints

`setAllowedAreas(Left | Right)` and `setFeatures(Movable | Floatable | Closable)`.
An event filter rejects drops onto a tab bar; `dockLocationChanged` re-docks the
panel to its previous side (`setFloating(true)` then `addDockWidget`) when
`tabifiedDockWidgets()` is non-empty. The custom title bar is draggable. Qt has
no clean tabify-veto API; the fallback is the contract.

### 3.6 Shortcut cycling and preference

A letter resolves to exactly one group. Two shortcuts per distinct letter
(plain, `Shift`):
- preference **on**: plain → activate the slot's current member; `Shift`+letter
  → `cycleGroup()` (implemented only, wrapping);
- preference **off**: plain → `cycleGroup()`.
An all-unimplemented group does nothing. The unlettered Blur/Sharpen/Smudge slot
is unaffected. The `B`/`Shift+B` special case is deleted.

### 3.7 Session schema v4

`SessionState` gains `int toolsColumns = 1;` and
`bool useShiftKeyForToolSwitch = true;`, `schemaVersion = 4`. `loadSession()`
reads both with the defaults; `saveSession()` writes them; the M39
load-before-write path is unchanged. The frame passes the loaded values into
`buildTools()` and persists `tools->columns()`.

### 3.8 Test hooks

`Toolbox`: `columns()`/`setColumns(int)`, `openSlotFlyoutForTest(int)`,
`slotMenuForTest(int)`, `slotMenuActionsForTest(int)`, `cycleGroupForTest(int)`,
`titleBarToggleForTest()`, `hasFlyoutTriangleForTest(int)`. `frame`: a
test entry to the generic shortcut handler.

## 4. Task split

1. Brief + frozen interfaces (this file, the OpenSpec artifacts) — **done in the
   proposal**.
2. Assets: the two `panel.columns*` icons + qrc.
3. Flyout triangle + long-press/right-click + keys.
4. Title bar + one/two-column toggle.
5. Standalone dock + tabify refusal.
6. Generic `Shift`+letter cycling + preference + session v4.
7. Self-test (`m40_columns`, `m40_flyout`, `m40_keys`, `m40_shift`, `m40_dock`,
   `m40_session`; exit codes 113–119).
8. Verification + close-out.

## 5. Non-goals

The panel dock/rail refactor; compact/iconic docks; popup flyouts for panels;
`Auto-Collapse Iconic Panels`; the Preferences dialog and the preference's UI
(M41); temporary tool switching; tool-preset pickers; and any change to the
71-tool catalogue or the 10-tool implemented set. No Rust, bridge, document,
compositor, PSD, or dependency change.

## 6. Program renumbering (M40/M41)

M40 is this Tools-panel milestone and **M41 is the Preferences dialog** (General
+ Interface panes, giving `Use Shift Key For Tool Switch` and
`Auto-Collapse Iconic Panels` a UI). The Layers-panel program therefore shifts by
two behind the pair: **M42** filtering/search, **M43** remaining management ops,
**M44** styles/effects, **M45** smart objects / vector masks / artboards / layer
comps. The deferred canvas-performance tracks (history copy-on-write, resident
per-layer GPU sources, 256² tiles + LoD, zero-copy present) are read **by name**,
not by number. `docs/dev/layers-panel-program.md` still carries the pre-shift
numbers; `docs/dev/STATE.md` is the up-to-date anchor.

## 7. Verification

- `openspec validate m40-tools-panel --strict` and
  `openspec validate --all --strict`.
- `git status --porcelain` shows docs-only changes for this proposal.
- Implementation (later) is verified by `cmake --build build`, both app
  self-tests, `cargo fmt`/`clippy`/`test` (expected unchanged), and the `m40_*`
  steps.

## 8. Decisions where CS6 is silent

- Two-column reading order is **row-major** (left-to-right, top-to-bottom).
- The toggle button shows the **target** layout's icon, not the current one.
- The hold delay is **300 ms**.
- The custom title bar carries only the label and the double arrow (no panel
  menu, no float/close glyphs).
- The column choice is one global session value, not per workspace.
- Qt has no clean tabify-veto API; the `dockLocationChanged` re-dock is the
  fallback.
