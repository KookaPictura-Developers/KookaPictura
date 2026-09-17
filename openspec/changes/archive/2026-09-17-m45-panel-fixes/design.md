## Context

M41 (archived) replaced the right-hand `QDockWidget` area with a custom
`PanelColumn` (`crates/pictura-app/cpp/panels/panel_column.{h,cpp}`): a vertical
`QSplitter` of `PanelGroup : QWidget` top-tab groups in a `QScrollArea`, a
`panelColumnToggle`, an iconic icon strip, `Qt::Popup` flyouts, and tab
drag/regroup/insert with a blue drop indicator. M42–M44 refined it and added the
in-window `PanelFloat`, the ordered multi-column host, session v6, the compact
strip with group drop targets, and the scoped theme. M45 is the seventh
user-requested pass and is pure correction: no new subsystem.

The relevant current state:

- `panel_column.cpp`: `resolveDrop` (`:1195`) delegates cross-column only for
  `IntoGroup` (`:1231–1240`) and returns a `DropTarget` with no owning-column
  identity; `showIndicatorFor` (`:1453`) always maps coordinates through the
  **source** column's `scroll_->viewport()` and, in compact mode, anchors the
  line to the first icon button (`:1467–1483`); `commitDrop` (`:1618`) is the
  only caller of `frame->removeColumnIfEmpty` (`:1669`); `updateMinimumWidth`
  (`:407`) derives a per-column minimum from the widest group's hint clamped to
  `[kNormalMinWidthFloor (180), kNormalMinWidthCap (320)]`; the scroll area's
  horizontal policy is `Qt::ScrollBarAlwaysOff` (`:136`); `openIconFlyout`
  (`:753`) detaches **one** panel into a bespoke `PanelFlyout`; `buildTabMenu`
  (`:908`) hard-codes `Minimize` (`:928`, and `kTabMenuTexts` `:58`).
- `panel_group.cpp`: `applyMinimize` (`:658`) clamps only
  `tabs_->maximumHeight`, leaving the `PanelGroup` widget at its expanded
  splitter height; `contentHiddenForTest` (`:847`) reads that clamp.
- `toolbox.cpp`: `contentWidth(columns)` (`:548`); `updateContentMetrics`
  (`:559`) fixes width for left/right and height for top/bottom (both while
  floating); `setColumns` (`:535`) calls it; `setAllowedAreas(Qt::AllDockWidgetAreas)`
  (`:328`).
- `frame.cpp`: `columnEdgeAnchorAt` (`:875`), `newColumnSideAt` (`:842`),
  `columnAtGlobal` (`:831`), `removeColumnIfEmpty` (`:804`), the toolbox as a
  left `QDockWidget` (`:1551`).
- `main.cpp`: exit **152** is the headless platform mismatch; M44 owns
  **153–166**; M45 allocates from **167**.

Constraints: this change is docs/proposal only. No `.rs`, `.cpp`, `.h`,
`CMakeLists.txt`, or asset change lands here; `theme.cpp` is untouched and
`docs/` is unchanged except the new brief. No dependency is added. The frozen
interfaces live in `docs/dev/m45-panel-fixes.md`.

## Goals / Non-Goals

**Goals:**

- Make the drop indicator a render of the resolved `DropTarget` in the owning
  column, so the line and the commit can never disagree, including
  cross-column, rightmost-tab, bottom-boundary, and compact group-drag cases.
- Recompute the Tools toolbar's content width and height from one formula on
  every column-count and dock/float change, and restrict it to left/right with
  any-side-of-a-column placement and a visible indicator.
- Remove a dynamic column emptied by any path, collapse a minimized group to its
  tab bar with a state-derived menu label, and give every widget column one
  minimum-width floor and no clipping.
- Make the compact popup host the whole `PanelGroup`, so docked, popup, and
  floating are the same widget with no parity differences.
- Ship runnable checks: the `m45_*` self-test steps at exit codes **167–179**,
  headless and layout-forced.

**Non-Goals:**

- A second drag/drop system or indicator; the M41–M44 `DropTarget`/
  `resolveDrop`/`commitDrop` path and its one `#2a7fff` indicator stay the only
  grammar.
- Real content for the placeholder panels; pixel-exact CS6 metrics; a sourced
  CS6 dock metric.
- Persisting a float's position, a per-column compact mode, or
  multi-monitor/OS-window float chrome.
- Changing the tool catalogue, the M40 flyout/shortcut/dock contracts beyond
  the allowed dock sides, the M41 tab menu list, or the M42 per-widget menu
  contents.
- The Layers-panel program (filtering/search, management, styles/effects, smart
  objects) — it shifts again behind this pass.

## Decisions

### 1. The indicator is a render of the resolved target (W1/W2/W3/W6/C2) (frozen)

The resolver already computes the exact pending insertion; the bug is that the
indicator re-derives geometry from the drag's **source** column. M45 changes the
data flow, not the grammar:

- The resolver carries an owning-column identity alongside the `DropTarget` (the
  target group's column for a cross-column result). `updateDrag` shows the
  indicator through that owner, so a cross-column tab/body/boundary target is
  drawn in the target column at the target group's coordinates.
- A tab insert is drawn at `tabInsertionX(tabIndex)`, so the rightmost tab draws
  at its own index rather than x=0.
- A boundary insert at the bottom is drawn at the last visible group's bottom
  edge.
- In compact mode a group-drag boundary is drawn at the top of the group's
  container (above the `panelIconGroupGrip` dots), not above the first icon
  button.

`commitDrop` keeps consuming the same `dropTarget_`, so line and commit share
one source of truth. *Alternative considered:* a second geometry pass in
`showIndicatorFor`. Rejected: it is exactly the drift this item reports.

### 2. One cleanup path for emptied dynamic columns (W4) (frozen)

`removeColumnIfEmpty` is only called from `commitDrop`. M45 introduces a single
`PanelColumn::maybeRemoveSelf()` that calls it, and invokes it after every path
that can empty the column: commit, `closeGroup`, `showPanel(name,false)`, and
the flyout-restore path that detaches the last panel. The primary and
non-dynamic columns are never removed; the remove-then-insert and
never-double-parent invariants are unchanged. *Alternative considered:* call
`removeColumnIfEmpty` in each setter. Rejected: it is the same bug in more
places.

### 3. Minimize clamps the group, not only its tab widget (W5) (frozen)

`applyMinimize` clamps `tabs_->maximumHeight`; the `PanelGroup` still fills its
splitter share. M45 clamps the **group's** maximum height to the tab-bar height
when minimized and restores the saved maximum on expand (kept symmetric with the
existing `savedMaxHeight_`). The tab menu entry becomes a function of
`isMinimized()` — `Expand Panel` while minimized, `Minimize` otherwise —
replacing the literal in `buildTabMenu` and the `kTabMenuTexts` table in place.
The `m45_minimize_collapse` check reads the group height, the content-hidden
flag, and the menu text.

### 4. One content formula for both toolbar axes (T1) (frozen)

`contentWidth(columns)` already exists. M45 adds `contentHeight(columns)` from
the same grid metrics and recomputes **both** locks in `updateContentMetrics`
after `reflow()` + layout activation, releasing the stale axis before re-fixing
it. `setColumns`, `dockLocationChanged`, and `topLevelChanged` all funnel
through it. A 1↔2 switch therefore cannot leave a stale width (cut one-column) or
a stale height (over-tall two-column). *Alternative considered:* fix each mode
separately. Rejected: one formula is the fix.

### 5. Tools: left/right only, placed beside a column (T2/T3) (frozen)

The allowed main-window areas revert to `Left | Right`; top/bottom are removed.
"Beside a widget panel/column, wherever the columns are docked" is implemented
by the frame resolving a floating-toolbar drop through the existing column
grammar (`resolveDrop`/`columnEdgeAnchorAt`) and hosting the toolbar as a
fixed-width central-splitter pane at that boundary; the single blue indicator is
shown for that target. With no column boundary under the pointer, the drop falls
back to the left/right main-window dock areas. The M40 no-tabification,
fixed-content-size, move, float, and close contracts are preserved. *Honest
limit:* a pure `QDockWidget` cannot sit between two columns; hosting it as a
pane is the faithful reading of the request.

### 6. One minimum-width floor; no clipping (W7/W8) (frozen)

Replace the per-column `widest`-derived, floor/cap-clamped minimum with a single
shared `kPanelMinWidth` used by every widget column in normal mode, so all
columns match and none can vanish. Change the scroll area's horizontal policy
from `AlwaysOff` to `AsNeeded`; tab text keeps eliding and the corner button
keeps its reserved width, so the right side is reachable instead of clipped. The
iconic strip keeps its own `kIconStripMinWidth`. Constants are chosen, marked
`ponytail:`. *Alternative considered:* keep the content-derived per-column
minimum. Rejected: unequal minimums are what let one column disappear.

### 7. The compact popup hosts the whole `PanelGroup` (C1) (frozen)

`openIconFlyout` currently detaches one panel into a bespoke flyout. M45 takes
the whole `PanelGroup` out of the column's splitter (remembering its index),
sets the clicked panel current, and hosts it in the `Qt::Popup`; close restores
the group at its original index. Docked, popup, and `PanelFloat` then present
the same `PanelGroup` — same tabs, styling, menu, minimize, and drag routing.
`PanelFlyout` shrinks to a frameless popup host and the bespoke one-tab header
is deleted. *Alternative considered:* a fourth presentation or a clone. Rejected:
the parity request is precisely to stop presenting the group differently.

### 8. Self-tests: bounded pump, forced layout, codes 167–179 (frozen)

The `m45_*` steps run through `crates/pictura-app/cpp/main.cpp` at codes
**167–179**, pump `QCoreApplication::processEvents()` a bounded number of times
before reading geometry/visibility/text, force a layout (`adjustSize`/`resize` +
pump) because offscreen widgets get no show/resize events, and
`std::fflush(stderr)`. **152** stays the platform mismatch and **153–166** stay
M44.

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

*Alternative considered:* reuse the M44 codes. Rejected: the headless self-test
keeps one addressable code per check.

## Risks / Trade-offs

- **T3's "between columns" cannot be a `QDockWidget`.** → Host the toolbar as a
  fixed-width central-splitter pane beside the anchor column, with the
  left/right dock areas for the outer edges; recorded as an honest limit.
- **Re-hosting the whole group in the popup can lose its splitter index.** →
  Remember the index before the take and restore it on close; the check asserts
  the group and its tab order round-trip.
- **A shared minimum width may be wider than a very narrow window.** → The
  column scrolls horizontally (`AsNeeded`) rather than clipping, keeping the
  no-clip contract.
- **Indicator ownership across columns can still race a relayout.** → Re-resolve
  the target each move and position from the resolved target, never a cached
  rect; the checks force layout before reading.
- **Forcing layout offscreen can give zero geometry.** → `adjustSize`/`resize`
  plus a bounded pump before every geometry read; checks assert non-empty rects
  and fail loudly rather than passing vacuously.
- **Chosen constants are unsourced CS6 metrics.** → One named constant per
  concern, marked `ponytail:`; a screenshot can retune them.

## Migration Plan

Additive and app-local. Rollback: restore the source-column indicator mapping,
the single `commitDrop` cleanup call, the tab-widget-only minimize clamp and the
literal `Minimize`, the per-column minimum, the `AlwaysOff` scroll policy, the
single-panel flyout, the `AllDockWidgetAreas`, and the un-recomputed toolbar
locks. No document, codec, session, theme, bridge-ABI, or on-disk change, so no
data migration is needed. Sequence: freeze the brief → Phase A (toolbar) →
Phase B (widget panel) → Phase C (compact) → Phase D (`m45_*` self-tests and
close-out prep).

## Open Questions

- **How wide is the toolbar pane beside a column?** It keeps its M43/M44
  fixed content width; the pane adds no extra width. If a future CS6 baseline
  says otherwise, the pane width changes in one place.
- **Should a minimized group persist its collapsed look across a dock round
  trip?** It already persists in the session v6 per-group `minimized` field;
  M45 only fixes the live geometry.
- **Should the shared minimum width apply to an iconic column?** No; the iconic
  strip keeps its own narrow minimum, and normal-mode columns share the floor.
