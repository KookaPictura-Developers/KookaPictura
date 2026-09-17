## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m41-panel-column.md`: the user-reported defects, the
  current-state inventory with file refs, the frozen design (panel column host,
  top-tab groups, default groups, width toggle, iconic strip + popups, seven-item
  tab menu, minimize vs collapse, drag grammar + blue line, tear-off/re-dock,
  Tools fixes, session v5, Preferences dialog), the test hooks and exit codes,
  the honest limits, and the non-goals
- [x] 1.2 Write `proposal.md`, `design.md`, this `tasks.md`, the
  `specs/panel-column/spec.md` (ADDED), the `specs/panel-rail/spec.md`
  (MODIFIED + REMOVED), and the `application-shell`, `tool-framework`,
  `workspace-persistence` MODIFIED deltas
- [x] 1.3 Freeze in `design.md`: `QTabWidget::North` and the no-label rule; the
  default CS6 Essentials groups and the Styles/Properties disposition; the
  splitter/scroll no-minimum rule; the `panelColumnToggle` control; the
  compact/iconic strip and `Qt::Popup` flyouts; the exact seven-menu items; the
  minimize vs collapse distinction; the drop-rule table and the thick blue
  indicator; the content-fit Tools widths; session schema v5; the Preferences
  panes; the test hooks and the 120–130 exit-code allocation
- [x] 1.4 Update `docs/dev/STATE.md`: M41 proposed, the M40/M41 pair, and the
  Layers-panel program line
- [x] 1.5 Validate: `openspec validate m41-panel-column --strict` and
  `openspec validate --all --strict`

## 2. Phase A — column infrastructure, panel conversion, fixed groups, rail removal, Tools fixes, width toggle

- [x] 2.1 Add the panel registry (panel → title, icon asset id, group id,
  visibility, iconic state, minimized state, order) and the `PanelColumn` host
  (vertical `QSplitter` of `PanelGroup`s inside a `QScrollArea`); no hard panel
  minimums, window resizes freely
- [x] 2.2 Add `PanelGroup : QTabWidget` with `setTabPosition(QTabWidget::North)`
  in the constructor; the tab text is the panel title, no separate group-title
  label, and a single-panel group still shows its tab
- [x] 2.3 Convert the fifteen panel classes (`LayersPanel`, `HistoryPanel`,
  `NavigatorPanel`, `ColorPanel`, `SwatchesPanel`, `InfoPanel`, `HistogramPanel`,
  and the Gradients, Patterns, Properties, Adjustments, Libraries, Channels,
  Paths, Actions placeholders) from `QDockWidget` to plain `QWidget`; preserve
  `objectName`, `setView`, `refresh`, and the existing test hooks
- [x] 2.4 Add the `Styles` placeholder content widget and register it in the
  former Gradients/Patterns slot; fold the Properties content into `Adjustments`;
  keep Gradients, Patterns, Properties, and Libraries registered and
  `Window > Panels`-toggleable but default-hidden
- [x] 2.5 In `frame.cpp`, replace the fifteen `registerPanel(...)` /
  `tabifyDockWidget(...)` calls with the `PanelColumn` and the CS6 Essentials
  groups; keep the Tools panel as a left/right dock; keep the `Window > Panels`
  toggle path as the single command path
- [x] 2.6 Delete `panels/panel_rail.{h,cpp}` and its `addToolBar` registration;
  remove the `PanelRail` connects; update `CMakeLists.txt` (swap `panel_rail`
  for `panel_column` + registry); retire the `m24_rail` self-test line in favour
  of `m41_rail`
- [x] 2.7 Add the `panelColumnToggle` thin double-chevron control at the top of
  the column, switching `normal`/`iconic`, with the same visual language as
  `toolsColumnToggle`
- [x] 2.8 Tools fixes in `toolbox.{h,cpp}`: remove the `Tools` `QLabel` from the
  custom title bar (keep `toolsColumnToggle`), and derive the one/two-column
  widths from the slot buttons and `ForegroundBackgroundWidget` so the swatch
  fits within the current column width and never widens it
- [x] 2.9 C++ `m41_tabs` (exit **120**): `setTabPosition` is `North` for every
  group, the groups have no title label beyond the tabs, each tab text is the
  panel title, and a single-panel group shows its tab
- [x] 2.10 C++ `m41_minwidth` (exit **121**): the column toggle exists with
  `objectName` `panelColumnToggle`, the main window can shrink below the groups'
  size hints, the column scrolls, and no panel reports a hard minimum that
  forces a larger window
- [x] 2.11 C++ `m41_groups` (exit **120** continuation or its own code if the
  code budget allows — otherwise folded into `m41_tabs`): the default groups are
  exactly Color/Swatches/Styles, Adjustments, Layers/Channels/Paths,
  Navigator/Histogram/Info, and iconic History and Actions; the folded panels
  are reachable from the Window menu
- [x] 2.12 C++ `m41_rail` (exit **129**): no `PanelRail` and no far-right rail
  toolbar exists, and the five former rail toggles still work from the Window
  menu
- [x] 2.13 C++ `m41_tools` (exit **130**): the title bar has no `Tools` label,
  the widths are content-derived, the foreground/background control fits within
  the one- and two-column widths, and the M40 flyout/dock/column behaviour is
  unchanged

## 3. Phase B — compact/iconic mode, popup flyouts, tab menu, minimize

- [x] 3.1 Implement iconic mode: collapse the column to a narrow vertical strip
  with one icon button per panel and group dividers; show labels when the strip
  is widened past the threshold; persist and restore `railWidth` (persistence is
  Phase D; the strip/labels/dividers land here)
- [x] 3.2 Implement the `Qt::Popup` frameless flyout containing the panel
  content, opened from a strip icon, dismissed on click-away, without permanent
  focus theft
- [x] 3.3 Implement the seven-item tab context menu exactly as specified:
  `Close`, `Close Panel Group`, `Minimize`, `Collapse to Icons`,
  `Auto-Collapse Iconic Panels` (checkable), `Auto-Show Hidden Panels`
  (checkable), `Interface Options…`; disable an inapplicable action rather than
  showing an inert stub (`Interface Options…` emits `interfaceOptionsRequested`;
  the dialog is Phase D)
- [x] 3.4 Gate `Auto-Collapse Iconic Panels` and `Auto-Show Hidden Panels` on
  the persisted session flags and wire their effects
- [x] 3.5 Implement `Minimize` (group rolled up to its tab bar, content hidden)
  and `Collapse to Icons` (iconic representation) as distinct actions with
  per-group persisted state (persistence is Phase D)
- [x] 3.6 C++ `m41_iconic` (exit **122**): iconic mode collapses the column,
  dividers are present, an icon opens a `Qt::Popup` flyout, and a click-away
  closes it
- [x] 3.7 C++ `m41_menu` (exit **123**): the tab menu contains exactly the seven
  named items in order, the two checkable items toggle and persist, and
  `Interface Options…` opens the Preferences dialog on the Interface pane (the
  toggle and the signal land here; persistence and the dialog are Phase D)
- [x] 3.8 C++ `m41_minimize` (exit **124**): `Minimize` hides the content and
  leaves only the tab bar, `Collapse to Icons` is distinct, and the minimized
  state round-trips through the session (round-trip is Phase D)

## 4. Phase C — drag and drop, the blue insertion line, tear-off and re-dock

- [x] 4.1 Implement tab dragging with the frozen drop rules: reorder within a
  group, regroup into another group at the target index, insert a new group on
  the column background/boundary, and group drags between columns
- [x] 4.2 Implement the thick blue insertion line drawn by the drop target: a
  boundary between groups for a column/group drop, the target tab index for a
  tab/regroup drop; clear it when the drag leaves or is cancelled
- [x] 4.3 Implement tear-off into a floating `Qt::Tool` window containing a
  `PanelColumn` fragment, and re-dock by dropping the float on a column or group
- [x] 4.4 Add the test hooks: `dragReorderForTest`, `dropIndexForTest`,
  `dropIndicatorForTest`, `tearOffForTest`, `redockForTest`
- [x] 4.5 C++ `m41_drag` (exit **126**): a tab reorders within its group, a tab
  regroups into another group at the drop index, a drop between groups inserts a
  new group, and the blue indicator is present at the candidate position and
  clears on cancel
- [x] 4.6 C++ `m41_tearoff` (exit **127**): a group tears off into a floating
  window and re-docks into the column on drop

## 5. Phase D — session v5, Preferences dialog, self-tests

- [x] 5.1 Extend `session.{h,cpp}` to schema v5 with `panelRailMode`,
  `railWidth`, `autoCollapseIconic`, `autoShowHidden`, and the per-group
  collapsed/minimized/order/visibility state; read each with a default; keep the
  M39/M40 load-before-write path so unknown keys and the v4 keys survive
- [x] 5.2 Persist the column state on every change (mode, width, group state,
  the two checkable flags) and restore it at startup
- [x] 5.3 Add the `PreferencesDialog` with `General` and `Interface` panes; the
  Interface pane carries `Use Shift Key For Tool Switch`, `Auto-Collapse Iconic
  Panels`, and the column width-toggle default; applying writes the session and
  applies to the toolbox/column without a restart
- [x] 5.4 Wire `Interface Options…` to open the dialog on the Interface pane;
  promote `Edit > Preferences > General` to an implemented command opening the
  General pane if the command tree allows it (otherwise record the deferral)
- [x] 5.5 C++ `m41_prefs` (exit **125**): the Interface pane shows the three
  controls, a changed `Use Shift Key For Tool Switch` round-trips and is honoured
  by the toolbox, and only General/Interface are enabled
- [x] 5.6 C++ `m41_session` (exit **128**): the v5 fields round-trip, a schema-4
  store loads the defaults, and the v4 `toolsColumns` /
  `useShiftKeyForToolSwitch` keys and an unknown key are preserved
- [x] 5.7 Run `xvfb-run -a ./build/pictura --self-test` and the
  `two_layers.psd` variant, both exit 0; confirm every earlier check (m20–m40)
  is unchanged except the retired `m24_rail` line

## 6. Verification and close-out

- [x] 6.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo test --workspace` (expected unchanged — no Rust change)
- [x] 6.2 `cmake -S . -B build && cmake --build build`
- [x] 6.3 `openspec validate m41-panel-column --strict`;
  `openspec validate --all --strict`
- [x] 6.4 Record the M41 result in `docs/dev/STATE.md` (capability count → 60
  after archive)
- [ ] 6.5 Archive the change (`openspec archive m41-panel-column`) and commit —
  deferred; this task does not commit
