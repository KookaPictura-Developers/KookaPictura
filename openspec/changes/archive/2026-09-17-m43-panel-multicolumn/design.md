## Context

M41 (archived) replaced the right-hand `QDockWidget` area with a custom
`PanelColumn` (`crates/pictura-app/cpp/panels/panel_column.{h,cpp}`): a vertical
`QSplitter` of `PanelGroup : QWidget` top-tab groups inside a `QScrollArea`, a
`panelColumnToggle` normal/iconic switch, an iconic icon strip, `Qt::Popup`
flyouts, a seven-item tab context menu, and tab drag/regroup/insert with a blue
drop indicator. M42 (archived) refined it: bounded normal-mode minimum width, a
smallest-width compact transition, larger icons, compact-strip drag, a group-
styled inner-side flyout with a header close button, per-widget header menus, and
an in-window float overlay replacing the `Qt::Tool` `PanelFloat`. The frame's
central widget is already a horizontal `QSplitter` named `centerSplitter`
holding `tabs_` (the document `QTabWidget`) then `panelColumn_`
(`frame.cpp:77–84`), with stretch factor 1 on the document tabs.

The user's fifth refinement list is structural: the workspace must support
**multiple panel columns**, and a tab drag must be distinct from a group drag
end-to-end. The current `DropTarget` (`panel_column.h:163–172`) has only
`onTabBar`/`onStrip`/`group`/`tabIndex`/`boundary`/`stripIndex`; `resolveDrop`
(`panel_column.cpp:981–1039`) resolves only within the one column, and a tear-off
floats the source group regardless of whether the press was classified as a tab
drag (`panel_group.cpp:770–772` emits both `groupDragStarted` and
`tabDragStarted`). The session store is at schema v5 with `panelGroups`
(`session.h:20–30`).

Constraints: this change is docs/proposal only. No `.rs`, `.cpp`, `.h`,
`CMakeLists.txt`, or asset change lands here; the Rust bridge, document model,
compositor, and PSD I/O are untouched, and no dependency is added. `docs/` stays
untouched except the new brief. The frozen interfaces live in
`docs/dev/m43-panel-multicolumn.md`.

## Goals / Non-Goals

**Goals:**

- Separate tab drag from group drag: a tab drag moves only its panel (and a
  one-panel tear-off floats one panel); a header drag moves the group, for both
  docked and floating groups.
- Make the workspace an ordered multi-column host (left columns, document tabs,
  right columns) with dynamic column create on a drop and removal when emptied.
- Generalise `resolveDrop` to `new-column-left`/`new-column-right`,
  `into-group` (tab index), and `above-group`/`below-group`, for normal and
  iconic modes and for float and non-float drags, keeping the frozen blue
  indicator.
- Give the panel-group tab bar an `objectName` and scope the theme so the active
  tab takes the pane `base` colour and inactive tabs do not.
- Guarantee the corner `▾` button is never clipped at the column minimum width.
- Fix the compact flyout to use the actual button geometry and open strictly on
  the inner side; grow the compact icon again.
- Fix the Tools toolbar to its content width in one/two columns, floating and
  beside a column, and refuse width dragging.
- Reset fg/bg to black/white on `D`.
- Advance the session to schema v6 with the per-column layout and a v5 load path.
- Ship runnable checks: the `m43_*` self-test steps at exit codes **140–150**.

**Non-Goals:**

- Real content for the placeholder panels (Styles, Properties, Gradients,
  Patterns, Libraries) — they stay empty states.
- Pixel-exact CS6 metrics; column minimum widths, icon sizes, flyout offsets, and
  the new-column drop margins are chosen constants.
- Collaborative/independent per-column compact modes: `panelRailMode` stays one
  workspace-wide value in M43 (see Decision 8).
- Multi-monitor floating and OS-window float chrome; the float overlay stays
  in-window and clipped.
- Persisting a float's position or a torn-off float across restart.
- Any change to the tool catalogue, the implemented 10-tool set, the M40
  flyout/shortcut/dock contracts, the M41 tab context menu item list, or the M42
  per-widget menu contents.
- The Layers-panel program (filtering/search, management, styles/effects, smart
  objects) — it shifts again behind this pass (see §7 of the brief).

## Decisions

### 1. Multi-column host: ordered splitter panes, document tabs keep the stretch (frozen)

The `centerSplitter` becomes the host of an **ordered set** of panes: zero or
more left `PanelColumn`s, the document `QTabWidget`, then zero or more right
`PanelColumn`s. Every column is created equal (a `PanelColumn` whose groups are
dynamic); only its splitter index relative to `tabs_` marks its side. A drop
resolves a side, the frame inserts a new `PanelColumn` at the correct splitter
index, and the splitter stretch factors are re-applied so `tabs_` keeps factor 1
and every column factor 0. When a column ends up with no groups it is removed
from the splitter and deleted; when the last group leaves a column the same path
runs. This makes the M41 single-column layout the one-right-column case, so all
existing single-column behaviour is reused, not re-implemented.

*Side determination:* **layout order, not geometry** — a column is left if its
splitter index is below `tabs_`'s and right if above. Geometry is a rendering
consequence and is unreliable during a drag; index is the stored truth (Decision
7). `PanelColumn` gains a `side()` accessor derived from its splitter parent, and
`flyoutSide()` (M42: window-edge heuristic) becomes `side() == right ? left :
right` — the inner side for a left column is its right edge and vice versa.

*Alternative considered:* keep one column and model extra columns as nested
vertical splitters inside it. Rejected: it cannot place a column on the left of
the document tabs, and it conflates group stacking with column layout.

*Alternative considered:* a `QMainWindow` dock area per column. Rejected: the
M41 decision to leave `QDockWidget` is unchanged; the custom column is the host.

### 2. A tab drag moves a panel; a header drag moves a group (frozen)

`PanelGroup::eventFilter` already classifies a press on a tab as
`tabDragStarted(name, pos)` and a press on the empty header-right as
`groupDragStarted(pos)`. M43 threads that distinction through the whole drag:
`beginPanelDrag`/`beginGroupDrag` set the float **payload** (`panelName` or
empty-for-group), and `createFloat` builds a `PanelFloat` whose host group holds
either the single detached panel or the whole group. A single-panel float is
allowed to have exactly one panel (no tab strip re-grouping beyond re-docking);
a group float holds the group's panels. The same rule applies to a **floating**
group: the float's own tab bar routes a tab drag to a one-panel move and its
empty header to a group move.

*Alternative considered:* always float the group and prune it to one panel when
the drag started on a tab. Rejected: the user sees the whole group leave first,
and the re-dock bookkeeping is more complex than choosing the payload up front.

### 3. One `DropTarget`, generalised; one frozen indicator (frozen)

`DropTarget` gains a `kind` enum with the frozen members:

| kind | meaning |
|---|---|
| `reorder` | same-group tab reorder (`group`, `tabIndex`) |
| `into-group` | tab insert into a target group (`group`, `tabIndex`) |
| `above-group` / `below-group` | boundary insert relative to a group (`group`, `boundary`) |
| `on-strip` | compact-strip reorder (`stripIndex`) |
| `new-column-left` / `new-column-right` | allocate a column on the named side of the workspace |
| `outside` | tear-off / float |

`resolveDrop(globalPos, payload)` keeps the existing per-column resolution and
adds a column-level resolution: a pointer left of the workspace's leftmost column
(or left of the Tools toolbar) resolves `new-column-left`; a pointer right of the
rightmost column resolves `new-column-right`; a pointer on a group's tab bar
resolves `into-group` (compact included) and on a group's top/bottom half
resolves `above-group`/`below-group` (compact included). A stable `new-column`
margin (a chosen constant band beyond the outermost pane edge) drives the
side decision, so dropping just outside a column always means "new column on
that side", never a same-column boundary. The drag source records the payload
(panel vs group) so `commitDrop` routes to `applyPanelDrop`/`applyGroupDrop`.

The indicator is unchanged: the single `panelDropIndicator`/strip indicator
draws the same 3 px `#2a7fff` line; a `new-column` candidate is shown as a
full-height line at the workspace edge (a new paint mode of the existing
indicator, not a second widget system).

*Alternative considered:* a second drop resolver per column and a cross-column
dispatch. Rejected: two resolvers are two places to drift; extending one
`resolveDrop` with a column pass keeps a single grammar.

### 4. New columns on a side drop; removed when emptied (frozen)

Left/right of an existing column, of a group at the workspace edge, or of the
Tools toolbar SHALL allocate a **new `PanelColumn`** on that side and place the
dragged group in it — the user's explicit choice over "new group in the same
column". The frame exposes `createPanelColumn(side)`, `sideOf(column)`,
`columnCount()`, and `removeColumnIfEmpty(column)`. A drop onto an empty
workspace edge creates a fresh right/left column with the dragged group as its
only group; dropping a whole column's last group out of it removes the column.
New `PanelColumn`s are wired with the same `stateChanged`/menu signals as the
initial one, and the `Window > Panels` path targets the column that owns the
panel (or the primary column for a hidden panel).

*Alternative considered:* reuse the nearest column and add a group. Rejected:
the user explicitly chose a new column; a group in the same column cannot express
a left column.

### 5. Panel tab bar styling is scoped by `objectName` (frozen)

`PanelGroup` sets `objectName` `panelTabBar` on its `QTabBar`. `theme.cpp` adds
scoped selectors `QTabBar#panelTabBar::tab` (inactive: `${window}`, hover
`${hover}`) and `QTabBar#panelTabBar::tab:selected` (background `${base}`, the
`QTabWidget::pane` colour, with `${windowText}` text). `QTabWidget::pane` stays
the widget background. The document tab bar keeps the existing unscoped
`QTabBar::tab` rules, so this change cannot move the document tabs' colours.

*Alternative considered:* a dynamic property on the group and a
`[panelTab=true]` selector. Rejected: `objectName` is already the repo's
identification convention and the QSS selector is equivalent.

### 6. Reserve the corner width so the `▾` button is never clipped (frozen)

`PanelGroup`'s header is a `QTabWidget` with the `▾` `QToolButton` installed as
the `TopRightCorner` widget (M42). At the column minimum width the tab bar's
size hint can exceed the available width and the corner widget is pushed out or
clipped. The fix has three parts: (a) `QTabBar::setElideMode(Qt::ElideRight)` so
tab text shrinks instead of forcing width; (b) `column` minimum-width
calculation adds the corner button's `sizeHint().width()` (and the tab bar's
frame) to the widest group hint; (c) the corner button is given a fixed width and
the tab bar's `QTabBar::setExpanding(false)`. The `m43_corner` check asserts the
corner button's global rect is inside the group's tab-bar row at the column
minimum width.

*Alternative considered:* hide the `▾` at small widths. Rejected: the per-widget
menu is a required control; hiding it is a regression.

### 7. Session v6: `panelColumns` nests the existing `panelGroups` (frozen)

`SessionState` gains `QJsonArray panelColumns`, where each entry is
`{ "side": "left"|"right", "order": <int>, "groups": [ <the v5 panelGroups
entry shape> ] }`. The `panelGroups` compaction (group key = first-ever panel
`objectName`, fields `order`/`visible`/`minimized`/`collapsed`) is reused verbatim
inside each column's `groups`. `schemaVersion = 6`; on load, a store with no
`panelColumns` (or older than v6) SHALL synthesise a single right-hand column
from the legacy top-level `panelGroups`, so every v5 store opens unchanged.
`saveSession()` keeps the load-then-write path so unknown keys survive. The frame
writes the column array on every layout change (via each `PanelColumn::
stateChanged`).

*Alternative considered:* a flat `panelGroups` with a `column` index and a
separate `columns` array. Rejected: nesting is one object per column, mirrors
the layout, and keeps the v5 compactor untouched.

### 8. Compact mode is workspace-wide; compact icon grows to 34 (frozen)

`panelRailMode` stays one workspace-wide value, applied to every column, so the
workspace has one compact/normal posture and the existing toggle and session key
keep working. The M43 icon-size ask is a modest bump of the strip button constant
(30 → 34, pixmap 20 → 24) for legibility; it is a chosen constant, not a CS6
metric. `flyoutSide()` is derived from the column's `side()` and `placeFlyout`
uses the **actual button geometry** (`button->mapToGlobal(QPoint(0,0))` plus
`button->size()`), not `anchorRightTop ± kIconButtonSize`, so the popup meets the
button's inner edge exactly and is clamped to the screen without ever crossing to
the outer side (the clamp bounds the inner coordinate, it never flips the side).

*Alternative considered:* per-column compact mode. Rejected: it multiplies the
session state and the toggle UX for no observed need.

### 9. Tools toolbar: fixed width via `setFixedWidth` + `Fixed` size policy (frozen)

`Toolbox::updateContentMetrics()` currently calls `setMinimumWidth(content)`
(`toolbox.cpp:514–517`). M43 calls `setFixedWidth(content)` on the dock (which
sets minimum == maximum == the tight content width for the active column count)
and sets the body widget's horizontal `QSizePolicy` to `Fixed`, so the dock's
separator cannot resize it in either column count or while floating. Because
`QMainWindow`'s internal dock splitter can still attempt a resize on some
platforms, the dock's `eventFilter` (already installed) clamps any width change
back to the fixed value — the honest belt-and-braces fallback. `topLevelChanged`
keeps the M42 float-height behaviour (trailing stretch zeroed, body invalidated).
The M40 left/right-only, movable/floatable/closable, refuse-tabify contract is
untouched, and the dock may sit on either side or beside a column.

*Alternative considered:* `setMinimumWidth == setMaximumWidth` without
`setFixedWidth`. Equivalent in effect; `setFixedWidth` is the single-call form
and also disables the size grip; chosen for that reason.

### 10. `D` resets fg/bg to black/white (frozen)

The frame's shortcut list already maps `X` to the fg/bg swap (M42). M43 adds `D`
routed to the existing `ColorState` default-colours reset (black fg, white bg)
when no tool shortcut claims it (`D` is unassigned in the toolbox catalogue). The
reset already exists as the corner control; `D` is the keyboard path, matching
CS6.

*Alternative considered:* a new reset method. Rejected: the corner reset's method
is the behaviour; the key just calls it.

### 11. Test hooks and exit codes (frozen)

The `m43_*` steps run through `crates/pictura-app/cpp/main.cpp`, allocate codes
**140–150**, pump the event loop bounded (`QCoreApplication::processEvents()`)
before reading geometry/visibility, and `std::fflush(stderr)` for blocked
stderr; `--self-test` pins xcb as M42 does. Hooks: per-column
`columnCountForTest`, `columnSideForTest`, `dropKindForTest`,
`singlePanelFloatForTest`, `floatPayloadForTest`, `tabBarObjectNameForTest`,
`cornerButtonVisibleForTest`, `fixedWidthForTest`, `flyoutGeometryForTest`,
plus the frame's `defaultColoursForTest`. Each check leaves one runnable
assertion and pins geometry.

## Risks / Trade-offs

- **Multi-column is a wide structural change.** → A new column is the same
  `PanelColumn` widget, and the single-column case is the one-right-column case,
  so the M41/M42 behaviour is reused. `resolveDrop` is extended, not replaced.
- **Column create/remove has edge cases (remove while dragging, empty column,
  last column, left-insert index churn).** → Removal is gated on empty and runs
  after a committed drop; indices are recomputed from the splitter on every
  resolution; the brief records the untested combinations as honest limits.
- **A left column inverts several M42 assumptions** (flyout side, drop
  new-column side, default groups). → `side()` is a single accessor all of those
  read; the default layout stays one right column, so the current workspace is
  unchanged until the user creates a left column.
- **The session v6 shape could drift from v5.** → `panelGroups` is nested
  verbatim and the legacy top-level key is read as the v5 fallback; unknown keys
  survive; a shape change is a later v7.
- **`setFixedWidth` on a dock may still be nudged by the `QMainWindow` splitter
  on some platforms.** → The dock's event filter clamps the width back, and
  `m43_tools` asserts minimum == maximum == content and that a separator drag
  does not change the width; if the clamp proves flaky the fallback is a
  `QSizePolicy` lock and recording the platform limit.
- **The new-column margin constant is unsourced.** → A chosen constant in one
  place, marked `ponytail:`; a CS6 screenshot can retune it.
- **The active-tab `${base}` colour may look too close to the pane on some
  brightness levels.** → It is exactly the user's request; a later theme tweak
  changes one selector.

## Migration Plan

Additive and app-local, except the central-splitter host and the session v6
shape. Rollback: collapse the splitter to one right column, restore the
single-column `resolveDrop`, restore the group-only float, unscope the tab
selectors, restore `setMinimumWidth` and the `anchorRightTop` flyout placement,
and drop the v6 `panelColumns` key (the v5 `panelGroups` stays on disk). No
document, codec, bridge, compositor, PSD, or on-disk format change, so no data
migration is needed. Sequence: freeze the brief → Phase A chrome fixes (tab vs
group drag, single-panel float, `D`, compact icon, flyout geometry, corner
button, tab colours, Tools fixed width) → Phase B the multi-column host and the
generalised drop targets → Phase C session v6 and the `m43_*` self-tests →

## Open Questions

- **Should a column remember its own compact/normal mode?** Frozen as
  workspace-wide in M43; a per-column mode is a later change if asked.
- **Should a torn-off float remember its column side on re-dock?** M43 re-docks
  using the frozen drop grammar; a float position/side is not persisted.
- **How far outside a pane starts "new column" vs "same-column boundary"?** A
  chosen margin constant; the `above`/`below` boundary remains available inside
  the column, so the two intents are distinguishable.
- **Whether a left column's default groups mirror the right column's.** The
  default layout stays one right column; a user-created left column starts with
  the dragged group only, and the Window menu can add more.
