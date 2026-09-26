# M41 — CS6 panel column (groups, iconic strip, drag/drop, Preferences)

- **Status:** proposed (`openspec/changes/m41-panel-column`); not implemented.
- **Type:** a user-requested milestone that pairs with M40 (M41 was the
  Preferences dialog in M40's plan; the user asked for the panel column in the
  same breath). It interrupts the Layers-panel program, which shifts by two
  behind the M40/M41 pair (see `docs/dev/STATE.md`).
- **Contract:** `docs/02-ui-ux/preferences.md` (`UI-010`, `Use Shift Key For
  Tool Switch` default on, General pane), `docs/02-ui-ux/workspace-and-docks.md`
  (`UI-003`, dock rules), `docs/02-ui-ux/panels/layers-panel.md` (`PAN-001`), and
  `docs/05-layers/layer-management-ui.md` (`LAY-002`). Canonical requirements:
  `openspec/specs/ui/panel-column/` (new), `ui/panel-rail`, `ui/application-shell`,
  `tools/tool-framework`, `document/workspace-persistence`.
- **Consumers:** `crates/pictura-app/cpp/panels/panel_column.{h,cpp}` (new),
  `panels/panel_registry.{h,cpp}` (new or folded),
  `panels/*_panel.{h,cpp}` (the fifteen converted panels),
  `panels/panel_rail.{h,cpp}` (deleted), `frame.{h,cpp}`, `toolbox.{h,cpp}`,
  `session.{h,cpp}`, `main.cpp`, and `CMakeLists.txt`.
- **Non-goals:** real content for the placeholder panels (Gradients, Patterns,
  Properties, Libraries are empty states), workspace presets /
  `Window → Workspace`, pixel-exact CS6 metrics, multi-monitor tear-off,
  floating-window platform chrome, `Tab`/`Shift+Tab` hide-all for the column,
  and any change to the tool catalogue or the M40 flyout/shortcut behaviour.

## 1. The user's report (verbatim intent)

1. The Tools panel should not have a "Tools" title.
2. The Tools panel should always use the smallest width possible; the
   foreground/background swatch must not be wider than the current width state.
3. The panel column is missing a width toggle.
4. The far-right rail is not in Photoshop CS6 and is unwanted.
5. Per-category tabs render at the bottom; the tabs must be on top.
6. Every panel in the column is a group: there is no separate group label — the
   tab holds the title.
7. A panel cannot be inserted between groups; dropping must show a thick blue
   line at the candidate placement.
8. Right-clicking a group tab must show `Close`, `Close Panel Group`,
   `Minimize`, `Collapse to Icons`, `Auto-Collapse Iconic Panels` (checkable),
   `Auto-Show Hidden Panels` (checkable), `Interface Options…`.

Two confirmed decisions: the CS6 Essentials default groups (`Color | Swatches |
Styles`; `Adjustments`; `Layers | Channels | Paths`; `Navigator | Histogram |
Info`; iconic `History`, `Actions`; `Styles` replaces the Gradients/Patterns
placeholder tabs and Properties folds into Adjustments), and full CS6-style
panel dragging with tear-off and re-dock.

## 2. Current state (inventory)

- `crates/pictura-app/cpp/frame.cpp:749–871`: fifteen panels built as
  `QDockWidget`s, registered in `Qt::RightDockWidgetArea` (`:799–813`) and
  tabified into Color+Swatches+Gradients+Patterns, Properties+Adjustments+
  Libraries, Layers+Channels+Paths (`:815–826`); History/Actions/Info/Navigator/
  Histogram hidden (`:828–833`); a `PanelRail` toolbar on the far right
  (`:835–870`).
- `crates/pictura-app/cpp/panels/panel_rail.{h,cpp}`: `PanelRail : QToolBar`
  (objectName `panelRail`), `addPanel`/`setPanelChecked`, one `commandTriggered`
  signal shared with the Window menu.
- `crates/pictura-app/cpp/panels/*_panel.{h,cpp}`: all fifteen panels derive
  from `QDockWidget`; `PlaceholderPanel(title, message, parent)` is shared by the
  eight placeholders.
- `crates/pictura-app/cpp/toolbox.{h,cpp}`: the M40 custom title bar already
  exists (`:264–279`) with a `Tools` `QLabel` and `toolsColumnToggle`; hard
  widths `setMinimumWidth(66)` (`:376`) and `104` (`:409`);
  `ForegroundBackgroundWidget` is fixed at `40×40` (`:160–171`).
- `crates/pictura-app/cpp/session.{h,cpp}`: schema v4 (`session.h:10–21`) with
  `toolsColumns` and `useShiftKeyForToolSwitch`; load-before-write save.
- `crates/pictura-app/cpp/main.cpp`: the M40 checks run through exit code 119;
  M41 allocates **120–130**.
- `crates/pictura-app/cpp/command_tree.cpp:173–186`: the `Edit > Preferences`
  leaves exist but are disabled (`General` carries `Ctrl+K`).
- **No** `PanelColumn`, `PanelGroup`, panel registry, column width toggle,
  iconic strip, panel popup flyout, tab context menu, panel drag/drop indicator,
  tear-off, or Preferences dialog exists today.

## 3. Frozen design

### 3.1 Panel column host

`PanelColumn` is a `QWidget` with a vertical `QSplitter` of `PanelGroup`s
inside a `QScrollArea`. Group heights are user-draggable; there are no hard
panel minimums, and a short window scrolls instead of forcing the main window
wider or taller.

### 3.2 Panel groups and top tabs

`PanelGroup : QTabWidget` with `setTabPosition(QTabWidget::North)` set
explicitly in the constructor. The tab text is the panel title; there is no
separate group-title label, and a single-panel group still shows its tab.

### 3.3 Default groups (CS6 Essentials)

`Color | Swatches | Styles`; `Adjustments`; `Layers | Channels | Paths`;
`Navigator | Histogram | Info`; iconic `History`, `Actions`. `Styles` is a new
placeholder in the former Gradients/Patterns slot; Properties content folds into
`Adjustments`. Gradients, Patterns, Properties, and Libraries stay registered
with their `objectName`s, reachable from `Window > Panels`, default-hidden.

### 3.4 Columns width toggle

A thin double-chevron `QToolButton` (`objectName` `panelColumnToggle`) at the
top of the column switches `panelRailMode` between `normal` and `iconic`. The
far-right `PanelRail` is deleted; the `Window > Panels` toggles remain the single
command path.

### 3.5 Compact / iconic mode

The column collapses to a narrow vertical strip of panel icons with group
dividers; panel labels appear when the strip is widened past a threshold.
Clicking an icon opens a frameless `Qt::Popup` flyout containing that panel's
content; it closes on click-away and does not steal focus permanently.
`railWidth` persists.

### 3.6 Tab context menu (exactly seven items)

`Close`, `Close Panel Group`, `Minimize`, `Collapse to Icons`,
`Auto-Collapse Iconic Panels` (checkable, default off), `Auto-Show Hidden
Panels` (checkable, default off), `Interface Options…`. Inapplicable actions
are disabled, never inert stubs. `Minimize` rolls the group to its tab bar;
`Collapse to Icons` reduces it to the iconic representation.

### 3.7 Drag and drop (frozen rules)

| Drag source | Target | Result |
|---|---|---|
| a tab | the same group's tab bar | reorder within the group |
| a tab | another group's tab bar | regroup at the target index |
| a tab | the column background / between groups | insert a new group at the boundary |
| a group tab bar | the column background | move the group to the boundary |
| any drag | past the column edge | tear off into a floating `Qt::Tool` window |

A floating group re-docks when dropped on a column or group. The **thick blue
insertion line** is drawn by the drop target: a boundary between groups for a
column/group drop, the target tab index for a tab/regroup drop, cleared on leave
or cancel.

### 3.8 Tools panel fixes

Remove the `Tools` `QLabel` from the custom title bar; keep
`toolsColumnToggle` and its `objectName`. Derive the one/two-column widths from
the slot buttons and `ForegroundBackgroundWidget` so the swatch fits within the
current column width and never widens it. M40's standalone dock, flyout
triangle, hold/right-click menu, shortcut keys, and `Shift`+letter cycling are
unchanged.

### 3.9 Session schema v5

`SessionState` gains `QString panelRailMode = "normal"`, `int railWidth = 0`,
`bool autoCollapseIconic = false`, `bool autoShowHidden = false`, and a
per-group state object (collapsed/minimized/order/visibility), with
`schemaVersion = 5`. A v4/older store loads the defaults; the load-before-write
path preserves unknown keys and the v4 keys.

### 3.10 Preferences dialog

A new `PreferencesDialog` with `General` and `Interface` panes. The Interface
pane carries `Use Shift Key For Tool Switch` (the M40 preference, currently
UI-less), `Auto-Collapse Iconic Panels`, and the column width-toggle default.
`Interface Options…` opens it on the Interface pane; promoting
`Edit > Preferences > General` (`Ctrl+K`) is optional and recorded. Other panes
stay disabled.

### 3.11 Test hooks

`PanelColumn`/`PanelGroup`: `tabsOnTopForTest`, `groupLabelForTest`,
`columnToggleForTest`, `setRailModeForTest`, `popupForTest`,
`tabMenuTextsForTest`, `minimizeForTest`, `dragReorderForTest`,
`dropIndicatorForTest`, `tearOffForTest`, `redockForTest`. Session v5 through
`saveSession`/`loadSession`. `Toolbox`: a title-bar label probe and a
width/content probe. Use `QCoreApplication::processEvents()` after layout
changes and `std::fflush(stderr)`.

## 4. Frozen self-test exit codes

| Code | Check |
|---|---|
| 120 | `m41_tabs` — tabs on top; no group label; a single-panel group shows its tab |
| 121 | `m41_minwidth` — column width toggle present; no hard minimum; window shrinks and the column scrolls |
| 122 | `m41_iconic` — iconic strip with dividers; icon opens a `Qt::Popup` flyout; click-away closes |
| 123 | `m41_menu` — the seven-item tab menu, in order; the checkable items persist; `Interface Options…` opens the Interface pane |
| 124 | `m41_minimize` — minimize hides content; collapse-to-icons is distinct; state round-trips |
| 125 | `m41_prefs` — Interface pane controls; preference round-trip; only General/Interface enabled |
| 126 | `m41_drag` — tab reorder, regroup, new-group insert, blue drop indicator draw/clear |
| 127 | `m41_tearoff` — group tears off to a floating window and re-docks |
| 128 | `m41_session` — v5 round-trip; v4 store loads defaults; v4 and unknown keys preserved |
| 129 | `m41_rail` — no `PanelRail`; Window-menu toggles still work |
| 130 | `m41_tools` — no `Tools` label; content-fit widths; swatch fits; M40 behaviour intact |

Also `m41_groups` (default groups) may share code 120 or take an unallocated
slot only if the table is amended here first; the table above is the contract.

## 5. Honest limits

- The placeholder panels (Gradients, Patterns, Properties, Libraries, Styles)
  remain empty states; M41 is chrome and behaviour, not panel content.
- Exact CS6 column-header, tab-bar, iconic-strip, and flyout metrics are
  unsourced; the structure is frozen, the pixels are not.
- Tear-off is clamped to the screen under the pointer; multi-monitor placement
  and custom floating-window chrome are out of scope.
- `Tab`/`Shift+Tab` hide-all for the column is not changed in M41.
- Whether CS6 hides a single-panel group's tab is unsourced; M41 always shows
  it (the tab is the title).
- The `Edit → Preferences…` entry point depends on promoting the disabled
  `Edit > Preferences > General` command; the tab menu's `Interface Options…`
  always works.

## 6. Non-goals

Placeholder content, workspace presets / `Window → Workspace`, pixel-exact CS6
metrics, multi-monitor tear-off, floating-window platform chrome, column
`Tab`/`Shift+Tab`, and any change to the tool catalogue or the M40 flyout,
shortcut, or dock contracts. No Rust, bridge, document, compositor, PSD, or
dependency change.

## 7. Verification

- `openspec validate m41-panel-column --strict` and
  `openspec validate --all --strict`.
- `git status --porcelain` shows docs + OpenSpec changes only for this proposal.
- Implementation (later) is verified by `cmake --build build`, both app
  self-tests, `cargo fmt`/`clippy`/`test` (expected unchanged), and the
  `m41_*` steps at codes 120–130.
