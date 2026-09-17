## Why

M42 shipped the refined `PanelColumn` chrome, and the user has handed back a
fifth lived-in refinement list. Two of the requests are structural rather than
cosmetic: the workspace must support **more than one panel column** (a floating
group dropped beside a column, a group at the workspace edge, or the Tools
toolbar must allocate a new column, not a new group), and a tab drag must
separate from a group drag end-to-end (today a single-tab tear-off still floats
the whole group). The remaining items close smaller chrome gaps: active-tab
colour, corner-button clipping at minimum width, Tools fixed width and no width
dragging, a bigger compact icon, strict inner-side flyouts, and the `D`
default-colours reset. This is the fifth panel interruption; it pins the
multi-column architecture down before the Layers-panel program resumes (see
Non-Goals and §7 of `docs/dev/m43-panel-multicolumn.md`).

## What Changes

- **Tab drag vs group drag.** Dragging a tab title SHALL drag only that panel;
  dragging the empty header area to the right of the tabs SHALL drag the whole
  group. The press classification already exists (`PanelGroup::eventFilter`
  emits `tabDragStarted`/`groupDragStarted`); the gap is that tear-off currently
  floats the whole group even for a single-tab drag. A single-panel drag SHALL
  float a one-panel float; a group drag SHALL float the group. The same rule
  SHALL hold for a floating group: one tab moves one panel, the empty header
  moves the group.
- **Active-tab colour.** The active panel-group tab background SHALL match the
  panel/widget background (`${base}`, the `QTabWidget::pane` colour); inactive
  tabs SHALL use `${window}`/`${hover}`. The panel tab bar SHALL get an
  `objectName` (e.g. `panelTabBar`) and a scoped theme selector, so the document
  tab bar is unaffected.
- **Corner-button visibility.** At the column's minimum width the per-widget
  context `▾` button is pushed outside/clipped; the header SHALL always show the
  full tab bar AND the corner button (elide tab text, reserve the corner width,
  and/or account for it in the minimum-width calculation).
- **New-column drop targets.** A floating group/panel dropped **left or right of
  an existing column, of a group at the workspace edge, or of the Tools
  toolbar** SHALL create a **new `PanelColumn`** on that side of the workspace
  and place the group in it (the user explicitly chose "always create a new
  column" over a new group in the same column). Dropping **inside** an existing
  group SHALL insert as a tab **including in compact/iconic mode**; dropping
  **above** or **below** a group SHALL insert at that boundary **including in
  compact mode**.
- **Multi-column architecture.** The central widget becomes a horizontal
  splitter whose panes are an ordered set of left `PanelColumn`(s), the document
  `QTabWidget`, and right `PanelColumn`(s), with the document tabs keeping the
  stretch. A `PanelColumn` SHALL be creatable dynamically on a drop and removed
  when emptied. All existing single-column behaviour (groups, tabs, iconic mode,
  flyouts, float overlay, per-widget menus, M41 tab menu) SHALL work in every
  column. The column side is determined by layout order (splitter index relative
  to the document tabs), and `flyoutSide`/drop resolution generalise to N
  columns.
- **Unified drop resolution.** The M41 `DropTarget`/`resolveDrop` SHALL be
  extended with targets for `new-column-left`/`new-column-right`, `into-group`
  (tab index), `above-group`/`below-group` (boundary), for both normal and iconic
  modes and for float and non-float drags. One thick blue indicator
  (`panelDropIndicator`/strip indicator) marks the candidate; the frozen
  M41/M42 indicator look is kept.
- **Tools toolbar fixed width.** Width dragging SHALL be disabled (the dock
  separator must not resize it); the Tools toolbar SHALL always use the minimum
  space in both 1- and 2-column modes **including while floating**
  (min == max == content width, recomputed on column-count change; the M42
  float-height behaviour stays). It SHALL remain placeable on either side of the
  workspace or beside a panel column (left/right), without breaking the fixed
  width rule or the M40 standalone-dock contract (left/right only, movable,
  floatable, closable, refuses tab groups).
- **Compact widget panel.** The compact icon buttons SHALL be a bit bigger again
  (M42 set 30 px). Flyouts SHALL **strictly** open on the inner side (left of the
  column when the column is on the right, right of the column when it is on the
  left), using the **actual button geometry**, never crossing to the outer side.
- **Foreground/background `D`.** The `D` key SHALL reset the foreground/background
  colours to the defaults (black/white), complementing the M42 `X` swap.
- **Session schema v6.** The store SHALL persist the multi-column layout: per
  column its side/order plus its groups' order/visible/minimized/collapsed (the
  existing `panelGroups` compaction nested under a `panelColumns` array), with
  load-then-write and unknown-key preservation. A v5 store SHALL load with a
  single right-hand column. `schemaVersion` 6.
- **Self-tests.** New `m43_*` checks at exit codes **140–150**; the driver pumps
  the event loop bounded and pins geometry (xcb for `--self-test`).

## Capabilities

### New Capabilities

None. M43 refines requirements already owned by `panel-column`,
`application-shell`, `tool-framework`, and `workspace-persistence`.

### Modified Capabilities

- `panel-column`: the column host becomes an ordered multi-column set with
  dynamic create/remove; the drag grammar separates a single-panel drag from a
  group drag (and a single-panel tear-off floats one panel); the drop grammar
  gains `new-column-left`/right, `into-group` (including compact) and
  `above`/`below` (including compact); the panel-group tab bar is named and the
  active tab takes the pane `base` colour; the header reserves the corner-button
  width and elides tab text; the compact icon grows again and flyouts strictly
  use the actual button geometry on the inner side; the session state advances to
  schema v6 with the column layout.
- `application-shell`: the central splitter hosts an ordered set of panel columns
  with the document tabs keeping the stretch; a column can be created by a drop
  and removed when emptied.
- `tool-framework`: the Tools panel's width is fixed (min == max == content width)
  in one and two columns, while floating and beside a column, and width dragging
  through the dock separator is refused.
- `workspace-persistence`: the session store advances to schema version 6 and
  persists the per-column layout (side/order plus nested group state); a v5 store
  loads with a single right-hand column and unknown keys survive.

No capability is added or removed; the canonical count stays at **60** after
archive.

## Impact

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}` — the multi-column host
  and `PanelColumn` create/remove, the generalised `DropTarget`/`resolveDrop`
  (new-column-left/right, into/above/below, compact mode), the single-panel
  float, the `flyoutSide`/`placeFlyout` actual-geometry fix, the bigger compact
  icon constant, and the `panelTabBar` objectName.
- `crates/pictura-app/cpp/panels/panel_group.{h,cpp}` — route a single-tab drag
  as a one-panel float and a header drag as the group float; name the tab bar and
  reserve the corner-button width so it is never clipped at minimum width.
- `crates/pictura-app/cpp/panels/panel_float` (folded into `panel_column`) — the
  float hosts a one-panel or whole-group payload and re-docks through the
  generalised drop targets.
- `crates/pictura-app/cpp/toolbox.{h,cpp}` — fixed content width (min == max),
  refuse separator width dragging, keep the 1-/2-column reflow and the M42
  float-height behaviour.
- `crates/pictura-app/cpp/frame.{h,cpp}` — host the ordered column set in the
  central splitter, create/remove columns on drop, route `D` to the
  default-colours reset, and persist/restore the v6 layout.
- `crates/pictura-app/cpp/theme.cpp` — the scoped `panelTabBar` selector (active
  tab `${base}`, inactive `${window}`/`${hover}`); `QTabWidget::pane` stays the
  widget background.
- `crates/pictura-app/cpp/session.{h,cpp}` — schema v6: `panelColumns` (side/
  order plus nested per-group state) with load-then-write and unknown-key
  preservation; a v5 store loads a single right-hand column.
- `crates/pictura-app/cpp/main.cpp` — the `m43_*` self-test steps, exit codes
  **140–150**.
- `docs/dev/m43-panel-multicolumn.md` (brief). No Rust, bridge, document,
  compositor, PSD, or dependency change.
