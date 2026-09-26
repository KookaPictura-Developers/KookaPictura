# M45 — panel fixes (seventh pass)

- **Status:** proposed (`openspec/changes/m45-panel-fixes`); not implemented.
- **Type:** a user-requested correction pass on the M41–M44
  `PanelColumn`/`PanelGroup`/`PanelFlyout`/`PanelFloat` chrome. It is the
  **seventh panel interruption** of the Layers-panel program; the program shifts
  behind it (see §7). STATE is handled at close-out.
- **Contract:** `openspec/specs/ui/panel-column/`, `ui/application-shell/`, and
  `tools/tool-framework/` are the canonical requirements this change reconciles.
  `docs/02-ui-ux/workspace-and-docks.md` (`UI-003`) governs docks. The drop and
  indicator lineage is M41–M44; the Tools fixed-size lineage is M43/M44.
- **Consumers:** `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`,
  `panels/panel_group.{h,cpp}`, `cpp/toolbox.{h,cpp}`, `cpp/frame.{h,cpp}`,
  `cpp/main.cpp`. `theme.cpp` is not expected to change.
- **Non-goals:** placeholder-panel content, pixel-exact CS6 metrics, a second
  drag/drop or indicator system, multi-monitor/OS-window float chrome,
  per-column compact modes, persisted float positions, a new dependency, any
  PSD/session/on-disk format change, and any bridge-ABI change.

## 1. The user's requests (all in scope)

Weakening a test to pass, or changing a golden baseline, is not part of this
pass. Each item is a correction to existing M41–M44 machinery.

**T. Tools toolbar**

1. **T1 — column-toggle sizing.** Switching 1↔2 columns mis-sizes the dock: the
   one-column layout gets cut in half and the two-column layout suddenly takes
   about twice the height it needs. Both modes must size exactly to content —
   width = content width, height = content height — with no jump, and the fixed
   width/height locks must be recomputed on `setColumns` and on dock/float.
2. **T2 — no top/bottom docking.** The M44 `AllDockWidgetAreas` is reverted: the
   toolbar is restricted to left/right again, but relative to widget columns.
3. **T3 — dock beside a widget column anywhere.** Dragging the floating toolbar
   to the left side of a widget panel/column showed no highlight and was
   refused; only the workspace's left/right edge worked. It must be placeable on
   any side of a widget panel/column, wherever the columns are docked (left of a
   right column, between columns, etc.), with the highlight shown.

**W. Widget panel (docked groups)**

4. **W1 — indicator on the wrong side.** Dragging a widget to the right side of
   the panel showed the placement line on the left while placing it on the
   right. The highlight must be drawn where the drop will actually happen.
5. **W2 — missing indicator on cross-column drops.** Dragging a widget to
   another widget panel allowed the placement but showed no placement line.
6. **W3 — rightmost-tab drop shows the line at the leftmost edge.** Dropping as
   the rightmost tab must show the line at that tab position, not the far left.
7. **W4 — remove an emptied column.** When a widget panel has no widgets left,
   the column must be removed (the M43 `removeColumnIfEmpty` exists; a case is
   escaping it).
8. **W5 — minimize must actually collapse.** A minimized group still occupies
   the expanded height. It must take only the tab height and hide the content,
   and the group menu entry must read **"Expand Panel"** while minimized (and
   "Minimize" otherwise).
9. **W6 — bottom-boundary indicator.** Dragging a group/item to the bottom of
   the panel must show the placement line at the bottom.
10. **W7 — never clip the right side.** When the column width is too narrow the
    right side is clipped/hidden. The panel must never hide content; min
    widths/eliding/scrolling must prevent clipping.
11. **W8 — minimum width floor.** Resizing the column to its absolute minimum
    makes the panel disappear; it must hold a minimum width **equal for all
    widget columns** and never vanish.

**C. Compact/iconic panel — full parity**

12. **C1 — the popup must be a real widget group.** Clicking an icon opens a
    single-panel popup; it must open the **same `PanelGroup`** (all the group's
    tabs, with the clicked panel active) exactly as docked and floating, with no
    parity differences between the three presentations.
13. **C2 — group-drag indicator position.** Dragging a whole group in compact
    mode shows the placement line **inside** the group (below the drag dots); it
    must show **above the drag dots** (the insertion point above the group).

## 2. Current state (inventory)

- `crates/pictura-app/cpp/toolbox.cpp`: `setColumns` (`:535`) calls `reflow`,
  `updateContentMetrics`, `updateTitleIcon`; `contentWidth` (`:548`) derives the
  grid width; `updateContentMetrics` (`:559`) fixes width for left/right and
  height for top/bottom, both for floating, and invalidates the body layout;
  `topLevelChanged` (`:459`) pins `floatHeight_ = sizeHint().height()`;
  `setAllowedAreas(Qt::AllDockWidgetAreas)` (`:328`). The fixed-axis recompute
  does not refresh both dimensions from a freshly activated layout, so a mode
  switch can leave a stale lock.
- `crates/pictura-app/cpp/frame.cpp`: `columnEdgeAnchorAt` (`:875`),
  `newColumnSideAt` (`:842`), `columnAtGlobal` (`:831`), `createPanelColumn`
  (`:782`), `removeColumnIfEmpty` (`:804` — early-returns for the primary and
  non-dynamic column, and only runs when called), the toolbox registration as a
  left `QDockWidget` (`:1551`).
- `crates/pictura-app/cpp/panels/panel_column.cpp`: `resolveDrop` (`:1195`)
  delegates cross-column only for `IntoGroup` (`:1231–1240`) and returns the
  target without an owning-column identity; `resolveLocalDrop` (`:1245`) returns
  `AboveGroup`/`BelowGroup` by the group's vertical center and `IntoGroup` on the
  tab bar; `resolveIconicDrop` (`:1310`); `showIndicatorFor` (`:1453`) always
  maps coordinates through the **source** column's viewport and draws the
  compact line above the first icon button (`:1467–1483`); `commitDrop` (`:1618`)
  is the only caller of `removeColumnIfEmpty` (`:1669`); `updateMinimumWidth`
  (`:407`) clamps the widest group's hint between `kNormalMinWidthFloor` (180)
  and `kNormalMinWidthCap` (320); `openIconFlyout` (`:753`) detaches **one**
  panel into `PanelFlyout` with a bespoke `panelFlyoutHeader`
  (`ensureFlyout`, `:718`); `buildTabMenu` (`:908`) hard-codes `Minimize`
  (`:928`); `kTabMenuTexts` carries the literal `"Minimize"` (`:58`).
- `crates/pictura-app/cpp/panels/panel_group.cpp`: `applyMinimize` (`:658`)
  clamps only `tabs_->maximumHeight` (`:666`), leaving the group widget at its
  expanded height; `contentHiddenForTest` (`:847`) reads that clamp; the group
  constructor (`:308`) has no minimized size policy.
- `crates/pictura-app/cpp/main.cpp`: `--headless --self-test` reserves exit
  **152** for a platform mismatch (`:173`); the M44 checks occupy **153–166**;
  no `m45_*` check exists.

## 3. Frozen design

### 3.1 Indicator is rendered from the resolved `DropTarget` (W1/W2/W3/W6/C2)

The line and the commit must never disagree. The resolver already returns the
exact pending insertion (`onTabBar`/`tabIndex`, `boundary`, `stripIndex`,
`kind`); M45 makes the indicator consume **that** target, in the column that
owns it, instead of re-deriving geometry from the drag's source column:

- Add an owning-column identity to the cross-column result so
  `showIndicatorFor` runs against the target group's column and maps the line
  through that column's viewport. A cross-column drop onto another group's tab
  bar, body, or boundary therefore draws in the target column (W1, W2).
- Position a tab insert at `tabInsertionX(tabIndex)`, so the rightmost tab draws
  at its own index and not at x=0 (W3).
- Position a bottom boundary at the last visible group's bottom edge (W6).
- In compact mode, the group-drag boundary line is drawn at the **top of the
  group's container (above the drag-handle grip)**, not above the first icon
  button (C2).

### 3.2 One cleanup path for emptied columns (W4)

`removeColumnIfEmpty` is invoked only from `commitDrop`. M45 routes every path
that can empty a dynamic column — drop, `closeGroup`, `showPanel(name,false)`,
and the flyout-restore path that detaches the last panel — through one
`maybeRemoveSelf()` call so a dynamic column can never persist with zero groups.
The primary column and non-dynamic columns keep their place. The
remove-then-insert and never-double-parent invariants are unchanged.

### 3.3 Minimize collapses the group, not just its tab widget (W5)

`applyMinimize` currently clamps `tabs_->maximumHeight`; the `PanelGroup` widget
still takes its expanded splitter share. M45 clamps the **group's** height to the
tab-bar height (and restores the saved maximum on expand), so a minimized group
occupies only its tab bar. The tab menu label is state-derived:
`Expand Panel` while minimized, `Minimize` otherwise, replacing the literal in
`buildTabMenu`/`kTabMenuTexts` in place.

### 3.4 Content size is recomputed from one formula (T1)

`contentWidth(columns)` already derives the width; M45 adds a matching
`contentHeight(columns)` and recomputes **both** locks in `updateContentMetrics`
from a freshly activated layout after `reflow()`, releasing the stale axis
before re-fixing it. `setColumns`, `dockLocationChanged`, and `topLevelChanged`
all funnel through it, so a 1↔2 column switch cannot leave a stale width or
height and neither mode is ever cut.

### 3.5 Tools docking: left/right only, placed relative to columns (T2/T3)

The allowed main-window areas revert to `Left | Right`. "Beside a widget panel
or column" is satisfied by hosting the floating toolbar as a fixed-width pane at
the resolved central-splitter boundary: the frame resolves the toolbar drop with
the same column grammar (`resolveDrop`/`columnEdgeAnchorAt`), shows the single
`#2a7fff` indicator at that boundary, and on drop inserts the toolbar pane
immediately before/after the anchor column, wherever the columns are docked.
With no column boundary under the pointer the drop falls back to the left/right
main-window dock areas. The M40 no-tabification, fixed-content-size, move,
float, and close contracts are preserved. This is a chosen interpretation
recorded as an honest limit; it is the only way to be "between columns".

### 3.6 One minimum width floor and no clipping (W7/W8)

Replace the per-column `widest`-derived, floor/cap-clamped minimum with a single
shared `kPanelMinWidth` floor used by **every** widget column in normal mode, so
all columns match and none can vanish. The column's horizontal scroll policy
becomes `AsNeeded` (it is currently `AlwaysOff`), tab text continues to elide,
and the corner button still reserves its width, so the right side is reachable
rather than clipped. The iconic strip keeps its own narrow `kIconStripMinWidth`.

### 3.7 The compact popup hosts the whole `PanelGroup` (C1)

`openIconFlyout` currently detaches one panel into a bespoke `PanelFlyout` with a
one-tab header. M45 takes the **whole `PanelGroup`** out of the column's splitter
(remembering its index), sets the clicked panel current, and hosts it in the
`Qt::Popup`; close restores the group at its original index. The popup, the
docked stack, and the `PanelFloat` overlay therefore present the same
`PanelGroup` — same tabs, styling, menu, minimize, and drag routing — with no
parity differences.

### 3.8 Self-tests: bounded pump, forced layout, codes 167–179 (frozen)

The `m45_*` steps run through `crates/pictura-app/cpp/main.cpp` at codes
**167–179**, pump `QCoreApplication::processEvents()` a bounded number of times
before reading geometry/visibility/text, force a layout (`adjustSize`/`resize` +
pump) because offscreen widgets get no show/resize events, and
`std::fflush(stderr)`.

| Code | Check | Covers |
|---|---|---|
| 167 | `m45_tools_sizing` | T1 |
| 168 | `m45_tools_sides` | T2 |
| 169 | `m45_tools_beside_column` | T3 |
| 170 | `m45_indicator_side` | W1 |
| 171 | `m45_indicator_cross_column` | W2 |
| 172 | `m45_indicator_rightmost_tab` | W3 |
| 173 | `m45_empty_column_removed` | W4 |
| 174 | `m45_minimize_collapse` | W5 |
| 175 | `m45_indicator_bottom` | W6 |
| 176 | `m45_no_clip` | W7 |
| 177 | `m45_min_width_floor` | W8 |
| 178 | `m45_popup_group` | C1 |
| 179 | `m45_compact_group_line` | C2 |

Suggested hooks: `toolbarContentSizeForTest`, `toolboxAllowedSidesForTest`,
`toolboxColumnDockForTest`, `dropIndicatorMatchesTargetForTest`,
`crossColumnIndicatorForTest`, `rightmostTabIndicatorForTest`,
`dynamicColumnCountForTest`, `minimizedGroupHeightForTest`,
`tabMenuTextsForTest`, `bottomIndicatorForTest`,
`contentInsideViewportForTest`, `minimumWidthFloorForTest`,
`iconFlyoutGroupForTest`, `compactGroupIndicatorForTest`.

## 4. Honest limits

- T3 is frozen as a fixed-width central-splitter pane beside a column; the
  main-window dock areas remain for the outer left/right edges. A pure
  `QDockWidget` cannot sit *between* two columns, so hosting the toolbar as a
  pane is the only faithful reading of "between columns".
- W7/W8 use chosen constants (`kPanelMinWidth`, the scroll policy); they are not
  sourced CS6 metrics and can be retuned.
- The indicator fix assumes the resolved `DropTarget` is authoritative; a
  target whose geometry is stale after a relayout is re-resolved rather than
  patched.
- Untested dock permutations (several columns on one side with a toolbar drag in
  flight) remain best-effort, as in M43.
- Forcing layout offscreen can yield zero geometry; every check forces a layout
  and pumps before reading, and fails loudly rather than passing vacuously.

## 5. Verification

- `openspec validate m45-panel-fixes --strict` and
  `openspec validate --all --strict`.
- `git status --porcelain` scoped to this proposal (the brief, the change
  directory, and the canonical deltas).
- Implementation (later) is verified by `cmake --build build`, both app
  self-tests, `cargo fmt`/`clippy`/`nextest`/doc, and the `m45_*` steps at
  codes 167–179.

## 6. No product code in this deliverable

This is a docs/proposal-only deliverable. No `.rs`, `.cpp`, `.h`,
`CMakeLists.txt`, or asset change lands here; `theme.cpp` is untouched; no
dependency is added. The frozen interfaces live in this file and in
`openspec/changes/m45-panel-fixes/`.

## 7. Layers-panel program shift (record only)

The M40–M44 interruptions pushed the Layers-panel program back by five; this
seventh pass pushes it once more. The renumbered program is:

- **M46** — layer filtering/search
- **M47** — remaining layer management operations
- **M48** — layer styles / effects
- **M49** — smart objects / vector masks / layers comps (artboards remain a
  CS6 non-goal)

STATE's program line is updated at close-out; this brief is the record.

## 8. Non-goals

Placeholder-panel content, workspace presets, pixel-exact CS6 metrics,
multi-monitor floating, OS-window float chrome, per-column compact modes,
persisted float positions, a second drag/drop or indicator system,
implementing any disabled per-panel menu entry, and any change to the tool
catalogue, the M40 flyout/shortcut contracts, the M41 tab menu list, or the M42
per-widget menu contents. No new dependency, no PSD/session/on-disk format
change, and no bridge-ABI change.
