## Why

The right-hand panel area is still 15 `QDockWidget`s tabified into the M24
grouping, plus a far-right `PanelRail` toolbar that Photoshop CS6 does not have.
The user's report is specific: the Tools panel carries a `Tools` title, its
width is a hard-coded 66/104 rather than fitting the swatch, the panel column
has no width toggle, the rail is unwanted, per-category tabs render at the
bottom instead of the top, every panel is not treated as a title-bearing group,
a panel cannot be dropped between groups, and a group tab has no CS6 context
menu. This is the **M41 panel-column** milestone the M40 change explicitly
deferred (M40's non-goals named the dock/rail refactor, compact/iconic docks,
panel flyouts, `Auto-Collapse Iconic Panels`, and the Preferences dialog), and
it lands the two decisions the user confirmed: the CS6 Essentials default groups
and full CS6-style panel dragging with tear-off/floating groups.

## What Changes

- **Custom `PanelColumn` replaces the right-hand `QDockWidget` area.** Fifteen
  panel classes become plain `QWidget` content widgets (dropping the
  `QDockWidget` base and the `setWidget`/dock API); the column is a vertical
  stack of `PanelGroup`s whose heights the user drags. There are no hard panel
  minimums: the column scrolls rather than forcing the main window wider or
  taller, and the existing public panel APIs (`setView`, `refresh`, test hooks,
  `objectName`) are preserved where possible.
- **`PanelGroup` with top tabs and no separate label.** Each group is a
  `QTabWidget` with `setTabPosition(QTabWidget::North)` explicitly set; the tab
  text is the panel title, there is no group-title label widget, and a
  one-panel group still shows its single tab. This fixes the bottom-tab and
  group-label complaints in one structure. Verified against the current specs:
  `docs/02-ui-ux/preferences.md` (`UI-010`) fixes `Use Shift Key For Tool
  Switch` default on; `docs/02-ui-ux/workspace-and-docks.md` (`UI-003`) governs
  docks; `docs/02-ui-ux/panels/layers-panel.md` (`PAN-001`) and
  `docs/05-layers/layer-management-ui.md` (`LAY-002`) stay the Layers-panel
  contract. `openspec/specs/application-shell`, `tool-framework`,
  `workspace-persistence`, and `panel-rail` are the canonical requirements this
  change reconciles.
- **Default CS6 Essentials groups (confirmed):** `Color | Swatches | Styles`;
  `Adjustments`; `Layers | Channels | Paths`; `Navigator | Histogram | Info`;
  and iconic `History`, `Actions`. `Styles` takes the former
  Gradients/Patterns tab slot; the Properties content folds into `Adjustments`.
  The Gradients, Patterns, Properties, and Libraries panel classes remain
  registered (stable `objectName`s) and are reachable from `Window > Panels`,
  but are not in a default visible group.
- **Column width toggle + compact/iconic mode.** One thin double-chevron
  control at the top of the column (`objectName` `panelColumnToggle`) switches
  the whole column between normal and compact mode; compact collapses it to a
  narrow vertical strip of panel icons with group dividers and labels shown
  when the strip is widened. Clicking an icon opens a frameless `Qt::Popup`
  flyout containing that panel's content, dismissed on click-away without
  stealing focus permanently. The far-right `PanelRail` is **deleted**
  (`panels/panel_rail.{h,cpp}` removed; the `Window > Panels` toggles stay and
  remain the single command path).
- **Tab context menu.** Right-clicking a group's tab bar shows exactly `Close`,
  `Close Panel Group`, `Minimize`, `Collapse to Icons`, `Auto-Collapse Iconic
  Panels` (checkable), `Auto-Show Hidden Panels` (checkable), and `Interface
  Options…`. The two checkable items persist; `Interface Options…` opens the
  new Preferences dialog on its Interface pane. **Minimize** rolls a group up to
  its tab bar; **Collapse to Icons** reduces the group to its iconic
  representation.
- **Drag and drop with a thick blue insertion line.** Dragging a tab reorders
  within a group, regroups between groups, inserts a new group on the column
  background, or tears a group off into a floating `Qt::Tool`-style window that
  can be dragged back and re-docked. During any drag a thick blue line marks the
  candidate drop position (a boundary between groups or a target tab index).
- **Tools panel fixes.** Remove the `Tools` `QLabel` from the custom title bar
  (keep the `toolsColumnToggle` double-arrow and its `objectName`), and make the
  one/two-column widths fit the content instead of the hard-coded 66/104 so the
  `ForegroundBackgroundWidget` fits within the current column width and never
  dictates it. The M40 standalone dock, flyout triangle, hold/right-click menu,
  shortcut keys, and `Shift`+letter cycling are unchanged.
- **Session schema v5.** Adds `panelRailMode` (`normal`|`iconic`), `railWidth`,
  `autoCollapseIconic`, `autoShowHidden`, plus per-group collapsed/minimized/
  order/visibility state. The v4 keys (`toolsColumns`,
  `useShiftKeyForToolSwitch`) and all earlier keys keep their load-then-write
  semantics so unknown keys survive.
- **Preferences dialog (new, `General` + `Interface` panes).** The Interface
  pane gives `Use Shift Key For Tool Switch` (M40, currently UI-less) and
  `Auto-Collapse Iconic Panels` a home, plus the column width-toggle default.
  Reached from the tab menu's `Interface Options…` and, if the command tree's
  `Edit > Preferences > General` leaf is implemented, from
  `Edit → Preferences…`.
- **BREAKING (internal UI plumbing).** The 15 panel classes no longer derive
  from `QDockWidget`; callers that used the dock API (`setWidget`, `show`/
  `hide` as docks) move to the column registry. Public panel APIs (`setView`,
  `refresh`, test hooks, `objectName`) stay unchanged where possible.

## Capabilities

### New Capabilities

- `panel-column`: the right-hand `PanelColumn` host, `PanelGroup` top-tab
  groups, the default CS6 Essentials groups, the column width toggle and
  compact/iconic icon-strip mode with `Qt::Popup` flyouts, the seven-item tab
  context menu, minimize vs collapse-to-icons, drag reorder/regroup/insert with
  the blue drop indicator, tear-off float and re-dock, the panel registry, panel
  set/visibility toggles, and the panel-column session state (v5).

### Modified Capabilities

- `panel-rail`: the icon rail is **REMOVED** (the M24 "Right panel icon rail"
  requirement is deprecated and the "Rail and Window menu share panel toggles"
  requirement becomes a Window-menu-only contract); the "Panel set" requirement
  is restated for content widgets hosted by the column with the CS6 Essentials
  default set.
- `application-shell`: the "Default dock grouping and canvas colour"
  requirement changes from three tabbed docks to the `PanelColumn`'s CS6
  Essentials groups; the canvas colour and `objectName` stability clauses stay.
- `tool-framework`: the "Tools panel column layout" requirement drops the
  separate `Tools` title label and replaces the fixed 66/104 widths with
  content-fit widths that the foreground/background swatch cannot widen.
- `workspace-persistence`: "Panels registered as named docks" becomes
  "registered as named content widgets hosted by the column" (the Tools panel
  and options bar remain docks), and the session store advances to schema v5
  with the panel-column fields while old stores load defaults.

No capability is removed; adding `panel-column` takes the canonical capability
count from **59 to 60** after archive. The `panel-rail` capability persists
(minus its rail requirement) so its panel-set contract is not orphaned.

## Impact

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}` (new) — `PanelColumn`,
  `PanelGroup`, the iconic strip, the `Qt::Popup` flyout, the seven-item tab
  menu, and the drag/drop with the blue insertion line.
- `crates/pictura-app/cpp/panels/panel_registry.{h,cpp}` (new, or folded into
  the column) — the panel → title, icon asset id, group id, visibility,
  iconic/minimized state map.
- `crates/pictura-app/cpp/panels/panel_rail.{h,cpp}` — **deleted**.
- `crates/pictura-app/cpp/panels/*_panel.{h,cpp}` — the 15 panel classes drop
  the `QDockWidget` base and the dock API; public `setView`/`refresh`/test
  hooks/`objectName`s stay.
- `crates/pictura-app/cpp/frame.{h,cpp}` — replace the 15 `registerPanel(...)`
  / `tabifyDockWidget(...)` calls and the `PanelRail` toolbar with the
  `PanelColumn`; keep the `Window > Panels` toggle path; host the new
  Preferences dialog and `General`/`Interface` panes.
- `crates/pictura-app/cpp/toolbox.{h,cpp}` — drop the `Tools` label, make the
  one/two-column widths fit the content, and keep `toolsColumnToggle`.
- `crates/pictura-app/cpp/session.{h,cpp}` — schema v5 fields and per-group
  state; load defaults for a v4/older store.
- `crates/pictura-app/cpp/main.cpp` — the `m41_*` self-test steps, exit codes
  **120–130**.
- `CMakeLists.txt` — swap `panel_rail` for `panel_column` (+ registry) sources.
- `docs/dev/m41-panel-column.md` — the brief; `docs/dev/STATE.md` — status.
- No Rust, bridge, document, compositor, PSD, or dependency change. No asset
  change is required beyond any icon art the compact strip needs, which reuses
  the existing `window.panels.<name>` icon ids.
