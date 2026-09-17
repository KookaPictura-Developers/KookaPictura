## Context

M20–M24 built the right-hand panels as `QDockWidget`s: `frame.cpp:749–871`
creates fifteen panels (`LayersPanel`, `HistoryPanel`, `NavigatorPanel`,
`ColorPanel`, `SwatchesPanel`, `InfoPanel`, `HistogramPanel`, and the
`PlaceholderPanel`s Gradients, Patterns, Properties, Adjustments, Libraries,
Channels, Paths, Actions), registers them in `Qt::RightDockWidgetArea`,
tabifies them into Color+Swatches+Gradients+Patterns,
Properties+Adjustments+Libraries, and Layers+Channels+Paths, and hides the five
"rail" panels (History, Actions, Info, Navigator, Histogram). A
`pictura::PanelRail` (`panels/panel_rail.{h,cpp}`) is added as a far-right
`Qt::RightToolBarArea` toolbar whose five buttons share a toggle path with the
`Window > Panels` commands. `frame.cpp` uses
`setDockOptions(AnimatedDocks | AllowTabbedDocks)`.

M40 (archived) rebuilt the Tools panel and fixed the CS6 contract there: a
custom `QDockWidget` title bar with a `Tools` label and a
`toolsColumnToggle` double arrow (`toolbox.cpp:264–279`), hard widths
`setMinimumWidth(66)` / `104` (`toolbox.cpp:376, 409`), and a
`ForegroundBackgroundWidget` fixed at `40×40` (`toolbox.cpp:160–171`). M40
explicitly deferred the panel dock/rail refactor, compact/iconic docks, panel
popup flyouts, `Auto-Collapse Iconic Panels`, and the Preferences dialog to M41
(`docs/dev/m40-tools-panel.md` §5). The session store is at schema v4
(`session.h:10–21`) with the M39 load-before-write save path.

Constraints: this change is docs/proposal only. No `.rs`, `.cpp`, `.h`,
`CMakeLists.txt`, or asset change lands here; the Rust bridge, document model,
compositor, and PSD I/O are untouched, and no dependency is added. `docs/`
stays untouched except the new brief (which needs a `TASK-ALLOWS-DOCS` marker
at commit time). The frozen interfaces live in `docs/dev/m41-panel-column.md`.

## Goals / Non-Goals

**Goals:**

- Replace the right-hand `QDockWidget` area with a custom `PanelColumn` of
  `PanelGroup`s whose tabs are on top and whose tab text is the only group
  chrome.
- Fix the Tools panel title and width complaints without regressing the M40
  flyout/dock/column contracts.
- Freeze the column width toggle, compact/iconic icon strip, `Qt::Popup`
  flyouts, the seven-item tab context menu, and minimize vs collapse-to-icons.
- Freeze the drag/drop grammar (reorder, regroup, new-group insert, tear-off
  float, re-dock) and the thick blue drop indicator.
- Delete the M24 `PanelRail` and keep the `Window > Panels` toggles as the
  single command path.
- Extend the session store to schema v5 and add the Preferences dialog with
  `General` + `Interface` panes.
- Ship runnable checks: the `m41_*` C++ self-test steps with frozen exit codes
  120–130.

**Non-Goals:**

- Real content for the placeholder panels (Gradients, Patterns, Properties,
  Libraries) — they stay placeholder content widgets and are reachable from
  `Window > Panels` but default-hidden. `Styles` is a new placeholder, not a
  working Styles panel.
- Workspace presets / `Window → Workspace` (the column layout is one global
  session value, matching M40's single `toolsColumns`).
- Pixel-exact CS6 metrics for the column header, tab bar, iconic strip, or
  flyout chrome; the exact CS6 icon art for the compact strip.
- Multi-monitor tear-off placement; floating-window platform chrome (no
  custom frame/always-on-top behaviour), which is left to the window manager.
- The remaining Layers-panel program work (filtering/search, management ops,
  styles, smart objects) — it stays behind M42+.
- Any change to the Tools catalogue, the 10-tool implemented set, or the M40
  flyout/shortcut behaviour.
- `Tab`/`Shift+Tab` hide-all semantics for the new column (see Open
  Questions): the `Window > Panels` toggles and `Auto-Show Hidden Panels`
  cover visibility in M41.

## Decisions

### 1. `PanelColumn` is plain widgets; the 15 panels drop `QDockWidget` (frozen)

The column is a `QWidget` with a vertical `QSplitter` of `PanelGroup`s
(handles give the user-draggable group heights) wrapped in a `QScrollArea` so a
short window scrolls instead of demanding height. Each of the fifteen panel
classes becomes a plain `QWidget`; its `setWidget`/dock API is deleted and its
`objectName`, `setView`, `refresh`, and test hooks are preserved. A small frame
registry maps each panel to `{title, icon asset id, group id, visible,
iconic, minimized, order}`.

*Alternative considered:* keep `QDockWidget`s and style/hide their tab bars.
Rejected: the bottom-tab orientation, the separate title bar, the per-dock
minimum sizes, and the tear-off semantics are exactly what the user reported
broken; fighting `QDockWidget`'s behaviour is more code than a
`QTabWidget`/`QSplitter` host, and `setTabPosition(North)` already fixes the
tab complaint for free.

*Alternative considered:* `QTabWidget`-per-group inside a `QToolBox`. Rejected:
`QToolBox` forces a header per page and a single expanded page, which
contradicts the splitter-dragged multi-group column.

### 2. `PanelGroup` tabs are on top and carry the title; no group label (frozen)

`PanelGroup : QTabWidget` sets `setTabPosition(QTabWidget::North)` explicitly
in the constructor so no stylesheet or platform default can move it. The tab
text is the panel title; there is no `QLabel` group header, and a group with one
panel still shows its single tab (matching CS6's title bar). The group's tab bar
is the right-click target for the context menu and is the drag handle for
tear-off/regroup.

*Decided where CS6 is silent:* whether a one-panel group hides its tab in CS6 is
unsourced; this design shows the tab (the tab IS the title).

### 3. Default groups are the confirmed CS6 Essentials set (frozen)

`Color | Swatches | Styles`; `Adjustments`; `Layers | Channels | Paths`;
`Navigator | Histogram | Info`; iconic `History`, `Actions`. `Styles` is a new
placeholder content widget that occupies the former Gradients/Patterns tab slot.
The Gradients, Patterns, Properties, and Libraries panels remain registered
content widgets with their existing `objectName`s, are reachable from
`Window > Panels`, and are not in a default visible group (the Properties
content conceptually folds into Adjustments; the panel itself is retained so
the M24 `panel-rail` "Panel set" contract is not orphaned).

*Alternative considered:* delete the Gradients/Patterns/Properties panels.
Rejected: the panels' `objectName`s are load-bearing for the M24 Window-menu
toggles and the session layout; keeping them registered is a smaller change and
lets a later milestone give them real content.

### 4. Column width toggle + compact/iconic mode (frozen)

One thin `QToolButton` (double chevron, `objectName` `panelColumnToggle`, same
visual language as M40's `toolsColumnToggle`) sits at the top of the column and
switches `panelRailMode` between `normal` and `iconic`. In `iconic` mode the
column collapses to a narrow vertical strip with one icon button per panel and
group dividers between groups; when the strip is widened past a threshold the
panel labels appear beside the icons (or in the flyout header). Clicking an icon
opens a frameless `Qt::Popup` containing that panel's content; `Qt::Popup`
closes on click-away by construction and the popup is non-modal so it does not
permanently steal focus. `railWidth` persists the strip width.

*Alternative considered:* a `QDockWidget` with `DockWidgetVerticalTitleBar`. It
does not give per-panel iconic flyouts or group dividers.

### 5. Tab context menu is exactly seven items (frozen)

Right-clicking a group tab bar shows, in order, `Close`, `Close Panel Group`,
`Minimize`, `Collapse to Icons`, `Auto-Collapse Iconic Panels` (checkable),
`Auto-Show Hidden Panels` (checkable), `Interface Options…`. `Minimize` rolls
the group to its tab bar only (content hidden, distinct from iconic collapse);
`Collapse to Icons` puts the group into the iconic representation. The two
checkable items persist in the session and gate auto-collapse/auto-show.
`Interface Options…` opens the Preferences dialog on the Interface pane.
Commands that are not implementable (e.g. `Close Panel Group` on a one-panel
group) are disabled, never shown as inert stubs.

*Alternative considered:* reuse the M40 `ToolSlotButton` menu style. The item
set is fixed by the user, so no alternative set is considered.

### 6. Drag and drop grammar and the blue insertion line (frozen)

A `QDrag` starts from a tab (or the group tab bar for a whole group). The drop
rules:

| Drag source | Target | Result |
|---|---|---|
| a tab | the same group's tab bar | reorder within the group |
| a tab | another group's tab bar | regroup (move the panel into the target group at the target index) |
| a tab | the column background | insert a new one-panel group at the drop boundary |
| a group tab bar | the column background / between groups | move the group to the boundary |
| any drag | past the column edge | tear off into a floating `Qt::Tool` window |

While dragging, a `QWidget` overlay on the target column/group paints a thick
blue line (a few pixels) at the candidate drop position: between groups for a
group/column drop, or at the target tab index for a tab/regroup drop. The line
is drawn by the drop target, not the drag source, so it appears on whichever
column/group is under the pointer. A floating group is a `Qt::Tool` window
containing a `PanelColumn` fragment; dropping it over a column/group re-docks
it and closes the float.

*Alternative considered:* Qt's `QTabBar::setMovable(true)` for reorder.
Rejected: it covers only reorder within one group; the between-group, new-group,
tear-off, and re-dock behaviours all need a custom `QDrag` handler, and adding
both would be more code than one handler.

*Alternative considered:* `QDockWidget` floating windows for tear-off.
Rejected: the panels are no longer docks (Decision 1), and a `Qt::Tool`
container is the direct equivalent.

### 7. Tools panel: no `Tools` label, content-fit widths (frozen)

`Toolbox`'s custom title bar drops the `QLabel` and keeps the
`toolsColumnToggle` button (same `objectName`). The one/two-column minimum
widths are computed from the actual slot button size plus layout margins and the
`ForegroundBackgroundWidget` size, rather than the hard-coded 66/104; the
widget is laid out so it fits **within** the current column width and never
widens the dock. The M40 standalone-dock constraints, flyout triangle,
hold/right-click menu, shortcut keys, and `Shift`+letter cycling are unchanged.

*Alternative considered:* keep 66/104 and shrink the swatch. Rejected: the
swatch size is the feature; the column should fit it, not the reverse.

### 8. Session schema v5 (frozen)

`SessionState` gains `QString panelRailMode = "normal"`,
`int railWidth = 0` (0 = derive), `bool autoCollapseIconic = false`,
`bool autoShowHidden = false`, and a per-group state collection
(collapsed/minimized/order/visible per group id) serialized as a JSON object,
with `schemaVersion = 5`. `loadSession()` reads each with a default;
`saveSession()` writes them. The M40/M39 load-before-write path is unchanged, so
a v4 store loads the new defaults and all earlier keys survive. The frame
passes the loaded values into the column and persists changes.

*Alternative considered:* store per-workspace state. Rejected: M41 has no
workspaces; one global column state matches M40's single `toolsColumns`.

### 9. Preferences dialog (frozen)

A new `PreferencesDialog` with two panes, `General` and `Interface`. The
Interface pane carries `Use Shift Key For Tool Switch` (the M40 preference,
currently UI-less), `Auto-Collapse Iconic Panels` (default off), and the column
width-toggle default. Applying writes the session and emits changes that the
frame routes to the toolbox and column. `Interface Options…` opens the dialog on
the Interface pane; if the `Edit > Preferences > General` leaf is promoted to an
implemented command, it opens the General pane (`Ctrl+K`). The other CS6 pane
leaves stay disabled and unimplemented.

*Alternative considered:* put both preferences in the Tools panel's own menu.
Rejected: `UI-010` places `Use Shift Key For Tool Switch` in the General pane,
and the user asked for an `Interface Options…` entry.

### 10. Test hooks and exit codes (frozen)

The `m41_*` steps run through `crates/pictura-app/cpp/main.cpp`, allocate
codes **120–130**, and use `QCoreApplication::processEvents()` after layout
changes plus `std::fflush(stderr)` for blocked stderr, as the existing driver
does. Test hooks on the column/group: `tabsOnTopForTest`,
`groupLabelForTest` (always absent), `columnToggleForTest`,
`setRailModeForTest`, `popupForTest`, `tabMenuTextsForTest`, `minimizeForTest`,
`dragReorderForTest`, `dropIndicatorForTest`, `tearOffForTest`,
`redockForTest`, and the session round-trip through `saveSession`/`loadSession`.

## Risks / Trade-offs

- **The panel rewrite is wide (15 classes).** → The public panel APIs,
  objectNames, and test hooks are preserved, so the frame and every existing
  self-test keep their call sites; the mechanical change is the base class and
  the registry registration.
- **`PanelColumn` is bespoke drag code.** → Qt covers the tab host
  (`QTabWidget`), the heights (`QSplitter`), the scroll (`QScrollArea`), and
  click-away dismissal (`Qt::Popup`); only the drop zones and the blue line are
  hand-rolled, and the drop rules are frozen above.
- **Tear-off is the riskiest path.** → The floating window is a `Qt::Tool`
  container with a `PanelColumn` fragment and a re-dock drop target; if it
  proves flaky the fallback is menu-driven `Collapse to Icons`/visibility, and
  the brief records the limitation.
- **No hard panel minimums can let a tiny window clip content.** → The column
  scrolls (Decision 1); the splitter still enforces a small per-handle cursor
  affordance, and the panel content is scrollable/clipped rather than pushing
  the window.
- **Session v5 is additive but the per-group state shape could grow.** → v5
  reads each field with a default and preserves unknown keys, matching the M39
  load-before-write path; a shape change is a later v6.
- **`Styles`/`Properties`/`Gradients`/`Patterns` are placeholders.** → Stated
  as a non-goal; they are content widgets with empty states, not features.
- **`Interface Options…` and `Edit → Preferences` are two entry points.** →
  Both call one `PreferencesDialog` with a pane selector; no duplicated state.
- **Exact CS6 tab/iconic metrics are unsourced.** → The structure is frozen,
  the pixels are not; the brief records this as an open question.

## Migration Plan

Additive and app-local, except the column rewrite and the panel base-class
change. Rollback: restore the `QDockWidget` panel area, the `PanelRail`
toolbar, the 66/104 widths and the `Tools` label, and drop the v5 session
fields. No document, codec, bridge, compositor, or on-disk PSD format change,
so no data migration is needed. Sequence: freeze the brief → the column host +
registry + panel conversion → fixed groups + tabs-North + rail removal + Tools
fixes + width toggle → compact/iconic + popups + tab menu + minimize → drag
reorder/regroup/insert + blue line + tear-off/re-dock → session v5 +
Preferences dialog → the `m41_*` self-tests → STATE.

## Open Questions

- **`Tab`/`Shift+Tab` on the new column.** The canonical
  `workspace-persistence` hide-all requirement names panel docks; whether `Tab`
  should hide the whole `PanelColumn` (and restore it) alongside the Tools dock
  is left unchanged in M41 to avoid widening the change. A follow-up can extend
  it.
- **`Styles` content and the default-group membership of Libraries.** The user
  confirmed the group names but not whether Libraries should join a default
  group; M41 leaves it registered and Window-toggleable only.
- **One-panel group tab visibility.** Whether CS6 hides a single tab is
  unsourced; this design always shows it.
- **Iconic strip widen-to-label threshold and flyout sizing.** Chosen
  constants; a CS6 screenshot would settle them.
- **Multi-monitor tear-off.** Not tested; the floating window is clamped to the
  screen under the pointer.
- **`Edit → Preferences…` command promotion.** The `Edit > Preferences >
  General` leaf exists but is disabled; M41 may leave it disabled and rely on
  the tab menu, or promote it. The brief records either outcome.
