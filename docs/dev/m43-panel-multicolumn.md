# M43 — panel multi-column (tab-vs-group drag, new columns, Tools fixed width)

- **Status:** proposed (`openspec/changes/m43-panel-multicolumn`); not
  implemented.
- **Type:** a user-requested refinement pass on the M41 `PanelColumn` chrome,
  extended to a structural **multi-column** workspace. It is the **fifth panel
  interruption** of the Layers-panel program; the program is renumbered behind it
  (see §7). STATE is handled separately.
- **Contract:** `openspec/specs/panel-column/`, `application-shell/`,
  `tool-framework/`, and `workspace-persistence/` are the canonical requirements
  this change reconciles. `docs/02-ui-ux/workspace-and-docks.md` (`UI-003`)
  governs docks. The tab-colour and active-tab contract is the same lineage as
  the M23 `application-shell` "CS6-style chrome styling" requirement.
- **Consumers:** `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`,
  `panels/panel_group.{h,cpp}`, `cpp/toolbox.{h,cpp}`, `cpp/frame.{h,cpp}`,
  `cpp/theme.cpp`, `cpp/session.{h,cpp}`, and `cpp/main.cpp`.
- **Non-goals:** placeholder-panel content, pixel-exact CS6 metrics,
  multi-monitor floating, OS-window float chrome, per-column compact modes,
  persisted float positions, and any change to the tool catalogue, the M40
  flyout/shortcut/dock contracts, the M41 tab context menu item list, or the M42
  per-widget menu contents. No Rust, bridge, document, compositor, PSD, or
  dependency change.

## 1. The user's requests (all in scope)

**Widget group**

1. Dragging a tab title must drag **only that panel**; dragging the **empty
   header area to the right of the tabs** must drag the **whole group**. The
   press classification already exists (`PanelGroup::eventFilter` →
   `tabDragStarted`/`groupDragStarted`); the gap is that tear-off currently
   floats the whole group even for a single-tab drag. A single-panel drag must
   float a **one-panel float**, a group drag the group. Same for a **floating**
   group.
2. **Active tab background must match the panel/widget background** (`${base}`,
   the `QTabWidget::pane` colour); **inactive tabs must not** match it (use
   `${window}`/`${hover}`). Scope this to the panel-group tab bars (give the tab
   bar an `objectName`, e.g. `panelTabBar`) so the document tab bar is unaffected.
3. At the column's **minimum width the widget context `▾` button gets pushed
   outside/clipped** — fix so the header always shows the full tab bar **and**
   the corner button (elide tab text, reserve the corner width, and/or account
   for it in the minimum-width calculation).

**Floating widget group — drop targets**

4. **Left/right of an existing column, group or the Tools toolbar → create a NEW
   COLUMN** (the user explicitly chose "always create a new column" over a new
   group in the same column).
5. **Inside an existing widget group** (tab insert) — **including compact/iconic
   mode**.
6. **Above** an existing group (new boundary) and **below** an existing group —
   **including compact mode**.

**Tools toolbar (`Toolbox`)**

7. **Disable width dragging** (the dock separator must not resize it).
8. **Always use the minimum space** in **both** 1- and 2-column modes,
   **including while floating** (min == max == content width; the M42 float-height
   behaviour stays).
9. It can be **dragged and placed on either side of the workspace or beside the
   panel column** (left/right), without breaking the fixed-width rule or the M40
   standalone-dock contract.

**Compact widget panel**

10. Make the icon buttons **a bit bigger again** (M42 set 30 px).
11. Some flyouts still open **in front of the icon button** — they must
    **strictly** open on the **inner side**: left of the column when the column
    is on the right, right of the column when it is on the left. Fix the
    placement to use the **actual button geometry** (the current `placeFlyout`
    derives the rect from `anchorRightTop ± kIconButtonSize`, which can overlap)
    and clamp without ever crossing to the outer side.

**Foreground/background**

12. **`D` key resets fg/bg to the default colours** (black/white) — complementing
    the `X` swap added in M42.

## 2. Current state (inventory)

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`: `PanelColumn` with a
  `QSplitter` of `PanelGroup`s in a `QScrollArea` (`:121–133`), the
  `panelColumnToggle`, the iconic strip (`kIconButtonSize = 30`,
  `kIconPixmapSize = 20`, `:36–37`), the M42 group-styled flyout
  (`:600–736`), the per-widget header menu, the drag/drop with the blue
  indicator (`indicator_` `panelDropIndicator`, `stripIndicator_`), and the
  in-window `PanelFloat` (`:75`, parented to `window()` not `centerSplitter`).
  `DropTarget` (`panel_column.h:163–172`) has only
  `onTabBar`/`onStrip`/`group`/`tabIndex`/`boundary`/`stripIndex`/`outside`.
  `resolveDrop` (`:981–1039`) resolves one column; `:984` handles the iconic
  branch; there is no column-side or new-column branch.
  `flyoutSide()` (`:675–688`) is a window-edge heuristic;
  `placeFlyout(anchorRightTop, size)` (`:690–712`) reconstructs the icon rect as
  `anchorRightTop.x() - kIconButtonSize` — the item-11 overlap source.
  `setRailMode` sets the column minimum width (`:354–368`).
- `crates/pictura-app/cpp/panels/panel_group.{h,cpp}`: `PanelGroup` with
  `QTabWidget::North`, the `▾` corner widget
  (`tabs_->setCornerWidget(headerButton_, Qt::TopRightCorner)`, `:328`), the
  strip/collapse helpers, and the tab-drag classification in `eventFilter`
  (`:742–792`; `emit groupDragStarted` `:770`, `emit tabDragStarted` `:772`).
  The tab bar has no `objectName` and no elide mode; the corner width is not
  reserved in the column minimum.
- `crates/pictura-app/cpp/toolbox.{h,cpp}`: `contentWidth()`/`updateContentMetrics()`
  (`:503–517`; `setMinimumWidth(content)` at `:517`), the M42 float-height path
  on `topLevelChanged` (`:456`), the event filter (`:476`), and the one/two-column
  reflow (`:585`).
- `crates/pictura-app/cpp/frame.cpp`: the central `QSplitter` `centerSplitter`
  holds `tabs_` then `panelColumn_` with stretch 1/0 (`:77–84`); it is the host
  that becomes the ordered column set. `panelColumn_` is a single member
  (`frame.h:173`); there is no column list, `createPanelColumn`, or side helper.
  `restoreStoredLayout` and `saveSession` write the v5 fields
  (`:678–716`).
- `crates/pictura-app/cpp/theme.cpp`: the unscoped `QTabBar::tab` /
  `QTabBar::tab:hover` / `QTabBar::tab:selected` rules (`:166–169`) apply to both
  tab bars; `QTabWidget::pane { background: ${base}; }` (`:171`). There is no
  `panelTabBar` selector.
- `crates/pictura-app/cpp/session.h`: schema v5 with `panelRailMode`, `railWidth`,
  `autoCollapseIconic`, `autoShowHidden`, and `panelGroups` (`:20–30`); no
  `panelColumns`.
- `crates/pictura-app/cpp/main.cpp`: the M42 checks end at code **139**; M43
  allocates **140–150**.
- **No** single-panel float, multi-column host, new-column drop, `panelTabBar`
  styling, corner-width reservation, fixed-width Tools dock, actual-geometry
  flyout, larger compact icon, or `D` reset exists today.

## 3. Frozen design

### 3.1 Multi-column host

The `centerSplitter` becomes the host of an **ordered set**: zero or more left
`PanelColumn`s, the document `QTabWidget`, then zero or more right
`PanelColumn`s. A column's side is its **layout order** relative to `tabs_`
(index below = left, above = right), not its geometry. The frame gains
`createPanelColumn(side)`, `sideOf(column)`, `columnCount()`, and
`removeColumnIfEmpty(column)`; the document tabs keep the splitter stretch and
every column keeps factor 0. A new column is the same `PanelColumn` widget,
wired like the initial one; a column with no groups is removed. The M41
single-column layout is the one-right-column case.

### 3.2 Tab drag vs group drag (single-panel float)

The `tabDragStarted`/`groupDragStarted` classification drives the float
**payload**: a tab drag floats a one-panel float and a header drag floats the
group. `commitDrop` routes by payload to `applyPanelDrop`/`applyGroupDrop`. A
floating group's own tab bar applies the same distinction.

### 3.3 Unified drop resolution

`DropTarget.kind` is frozen to `reorder`, `into-group`, `above-group`,
`below-group`, `on-strip`, `new-column-left`, `new-column-right`, `outside`.
`resolveDrop` keeps the per-column resolution and adds a column pass: beyond the
outermost pane edge (a chosen **new-column margin** constant) resolves
`new-column-left`/`new-column-right`; on a tab bar resolves `into-group`
(compact included); on a group's top/bottom half resolves `above-group`/
`below-group` (compact included). One `panelDropIndicator`/strip indicator draws
the frozen 3 px `#2a7fff` line; a `new-column` candidate is a full-height mark at
the workspace edge.

### 3.4 New columns on a side drop, removed when emptied

Left/right of a column, of a group at the workspace edge, or of the Tools
toolbar allocates a **new `PanelColumn`** on that side and places the group
(the user's explicit choice). A drop into an empty workspace edge creates a
fresh column; removing a column's last group removes the column; the splitter
stretch factors are re-applied; `Window > Panels` targets the owning column.

### 3.5 Panel tab bar colour

`PanelGroup` sets `objectName` `panelTabBar` on its `QTabBar`; `theme.cpp` adds
`QTabBar#panelTabBar::tab` (inactive `${window}`, hover `${hover}`) and
`QTabBar#panelTabBar::tab:selected` (active `${base}`, the pane colour). The
document tab bar's unscoped rules are untouched; `QTabWidget::pane` stays the
widget background.

### 3.6 Corner-button reservation

`QTabBar::setElideMode(Qt::ElideRight)` and `setExpanding(false)`, a fixed
corner-button width, and the corner width added to the column's minimum-width
calculation, so the `▾` button is fully inside the header row at the minimum
width and tab text elides instead.

### 3.7 Compact icon and strict inner-side flyout

The strip button/pixmap constants grow again to 34/24. `flyoutSide()` derives
from the column's `side()`; `placeFlyout` uses the **actual button geometry**
(`button->mapToGlobal` + `button->size()`) and clamps only the inner-side
coordinate, so the flyout meets the button's inner edge and never crosses to the
outer side.

### 3.8 Tools fixed width and no width dragging

`updateContentMetrics()` calls `setFixedWidth(content)` (minimum == maximum ==
the tight content width for the active column count) and sets the body's
horizontal size policy to `Fixed`; the existing event filter clamps any
separator attempt back to the fixed width. Recomputed on column-count change and
while floating; the M42 float-height behaviour and the M40 left/right-only dock
contract stay.

### 3.9 `D` resets fg/bg

The frame routes `D` (unassigned in the tool catalogue) to the existing
`ColorState` default-colours reset (black fg, white bg); `X` stays the swap.

### 3.10 Session v6

`SessionState` gains `QJsonArray panelColumns`: per column
`{ "side": "left"|"right", "order": <int>, "groups": [ <v5 panelGroups entry> ] }`.
`schemaVersion = 6`; a store without `panelColumns` synthesises one right-hand
column from the legacy top-level `panelGroups`, so every v5 store opens
unchanged. Load-then-write and unknown-key preservation are kept.

## 4. Frozen self-test exit codes (140–150)

| Code | Check |
|---|---|
| 140 | `m43_tabdrag` — a tab drag floats one panel; a header drag floats the group; a floating group routes both |
| 141 | `m43_tabcolors` — the active panel tab is the pane `${base}` colour; an inactive tab is `${window}` and differs; the document tabs are unchanged |
| 142 | `m43_corner` — at the column minimum width the corner `▾` button is fully inside the header row and tab text elides |
| 143 | `m43_newcolumn` — dropping left/right of a column, of a group at the edge, or beside the Tools toolbar creates a column on that side; an emptied column is removed |
| 144 | `m43_intogroup` — a drop inside a group inserts a tab, in normal and compact modes |
| 145 | `m43_boundary` — a drop above/below a group inserts a boundary group, in normal and compact modes |
| 146 | `m43_singlefloat` — a one-panel float carries only its panel; re-docking restores the group with the other panels |
| 147 | `m43_tools` — min == max == content width in one/two columns and while floating; a separator drag does not change the width |
| 148 | `m43_icon` — the strip icon button/pixmap are larger than the M42 sizes |
| 149 | `m43_flyout` — the flyout's inner edge meets the clicked button's actual edge and is never placed on the outer side |
| 150 | `m43_dreset` — `D` resets the foreground to black and the background to white |

Suggested test hooks: `columnCountForTest`, `columnSideForTest`,
`dropKindForTest`, `singlePanelFloatForTest`, `floatPayloadForTest`,
`tabBarObjectNameForTest`, `cornerButtonVisibleForTest`, `fixedWidthForTest`,
`flyoutGeometryForTest`, and the frame's `defaultColoursForTest`. Pump the event
loop bounded before reading geometry/visibility (the M40 Wayland-popup lesson;
the driver pins xcb for `--self-test`) and `std::fflush(stderr)`.

## 5. Session v6

`panelColumns` nests the existing `panelGroups` per column. `schemaVersion` 6; a
v5 store loads a single right-hand column from the legacy top-level
`panelGroups`; unknown keys survive. A float's position is still not persisted.

## 6. Honest limits

- Multi-column create/remove has untested edge cases: removing a column while a
  drag is in flight, removing the last column, and left-insert index churn when
  several columns share a side are best-effort and not all self-tested.
- `panelRailMode` is workspace-wide; a per-column compact/normal posture is a
  non-goal, so toggling compact affects every column.
- The new-column margin and the column minimum/icon constants are chosen, not
  sourced CS6 metrics; a screenshot can retune them.
- `setFixedWidth` on a dock may still be nudged by `QMainWindow`'s internal
  splitter on some platforms; the event-filter clamp is the fallback, and the
  self-test asserts the invariant rather than every platform.
- The in-window float overlay stays clipped to the main window: it cannot move
  to another monitor and carries no OS window chrome.
- The `panelColumns` ordering is best-effort when the same side has several
  columns; `order` is written from the live splitter order.
- The active-tab `${base}` colour is the user's explicit request; if it reads too
  close to the pane on a brightness level, that is a theme tweak, not a
  behavioural one.

## 7. Layers-panel program shift (record only)

The M40/M41/M42 interruptions pushed the Layers-panel program back by three;
this fifth pass pushes it once more. The renumbered program is:

- **M44** — layer filtering/search
- **M45** — remaining layer management operations
- **M46** — layer styles / effects
- **M47** — smart objects / vector masks / artboards / layer comps

STATE's program line is updated separately; this brief is the record.

## 8. Non-goals

Placeholder-panel content, workspace presets, pixel-exact CS6 metrics,
multi-monitor floating, OS-window float chrome, per-column compact modes,
persisted float positions, implementing any disabled per-panel menu entry, and
any change to the tool catalogue, the M40 flyout/shortcut/dock contracts, the
M41 tab context menu item list, or the M42 per-widget menu contents. No Rust,
bridge, document, compositor, PSD, or dependency change.

## 9. Verification

- `openspec validate m43-panel-multicolumn --strict` and
  `openspec validate --all --strict`.
- `git status --porcelain` shows docs + OpenSpec changes only for this proposal.
- Implementation (later) is verified by `cmake --build build`, both app
  self-tests, `cargo fmt`/`clippy`/`test` (expected unchanged), and the `m43_*`
  steps at codes 140–150.
