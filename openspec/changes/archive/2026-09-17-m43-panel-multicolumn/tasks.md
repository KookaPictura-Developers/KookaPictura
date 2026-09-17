## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m43-panel-multicolumn.md`: the twelve user-reported
  items, the current-state inventory with file refs, the frozen design
  (multi-column host, tab-vs-group drag, unified `DropTarget`/`resolveDrop`, new
  column create/remove, tab-bar objectName + scoped theme, corner-button
  reservation, compact icon size + actual-geometry flyout, Tools fixed width,
  `D` reset, session v6), the test hooks, the 140–150 exit-code table, the honest
  limits, the non-goals, and the Layers-panel program shift to M44 filtering,
  M45 management, M46 styles/effects, M47 smart objects
- [x] 1.2 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `panel-column`, `application-shell`, `tool-framework`, and
  `workspace-persistence` MODIFIED/ADDED deltas; freeze the side-determination
  rule, the `DropTarget` kinds, the session v6 shape, and the fixed-width
  mechanism in `design.md`
- [x] 1.3 Validate: `openspec validate m43-panel-multicolumn --strict` and
  `openspec validate --all --strict`

## 2. Phase A — tab-vs-group drag, single-panel float, D key, compact icon, flyout geometry, corner button, tab colours, Tools fixed width

- [x] 2.1 Thread the `tabDragStarted`/`groupDragStarted` classification through
  the whole drag: `beginPanelDrag`/`beginGroupDrag` carry a payload (panel vs
  group), and `createFloat` builds a one-panel float for a tab drag or a group
  float for a header drag; apply the same rule to a floating group's own tab bar
- [x] 2.2 Give `PanelGroup`'s tab bar `objectName` `panelTabBar`, and add the
  scoped `theme.cpp` selectors (active tab `${base}`, inactive `${window}`,
  hover `${hover}`) while `QTabWidget::pane` stays the widget background; leave
  the document tab bar's unscoped rules untouched
- [x] 2.3 Reserve the corner-button width: `QTabBar::setElideMode(Qt::ElideRight)`,
  `setExpanding(false)`, a fixed corner-button width, and add the corner width
  to the column's minimum-width calculation so the `▾` button is never clipped
- [x] 2.4 Raise the compact strip button/pixmap constants again (34/24) for the
  M43 icon-size request; keep the existing `icons.h` asset ids
- [x] 2.5 Fix `placeFlyout` to use the actual button geometry
  (`button->mapToGlobal` + `button->size()`) and derive the side from the
  column's `side()`, clamping only the inner-side coordinate so the flyout never
  crosses to the outer side
- [x] 2.6 Fix the Tools panel width: `setFixedWidth(content)` in
  `updateContentMetrics()` plus a `Fixed` horizontal size policy on the body,
  recomputed on column-count change and while floating; clamp any separator
  resize back through the existing event filter; keep the M42 float-height
  behaviour and the M40 left/right-only dock contract
- [x] 2.7 Reset fg/bg to black/white on `D` through the existing `ColorState`
  default-colours path when no tool shortcut claims it; keep the `X` swap
- [x] 2.8 C++ `m43_tabdrag` (exit **140**): a tab drag floats one panel, a header
  drag floats the group, and a floating group routes both
- [x] 2.9 C++ `m43_tabcolors` (exit **141**): the active panel tab is the pane
  `${base}` colour, an inactive tab is `${window}` and differs, and the document
  tabs are unchanged
- [x] 2.10 C++ `m43_corner` (exit **142**): at the column minimum width the
  corner button is fully inside the header row and the tab text elides
- [x] 2.11 C++ `m43_icon` (exit **148**): the strip icon button/pixmap are larger
  than the M42 sizes
- [x] 2.12 C++ `m43_flyout` (exit **149**): the flyout's inner edge meets the
  clicked button's actual edge and is never placed on the outer side
- [x] 2.13 C++ `m43_tools` (exit **147**): minimum == maximum == content width in
  one and two columns and while floating, and a separator drag does not change
  the width
- [x] 2.14 C++ `m43_dreset` (exit **150**): `D` resets the foreground to black and
  the background to white

## 3. Phase B — multi-column host, dynamic create/remove, unified drop targets

- [x] 3.1 Turn the `centerSplitter` into an ordered column host: an ordered set
  of left `PanelColumn`s, the document `QTabWidget`, and right `PanelColumn`s,
  with the document tabs keeping the stretch and every column factor 0; add
  `createPanelColumn(side)`, `sideOf(column)`, `columnCount()`, and
  `removeColumnIfEmpty(column)` to the frame
- [x] 3.2 Add `PanelColumn::side()` (layout order relative to `tabs_`) and route
  `flyoutSide()` through it, so the inner side generalises to N columns
- [x] 3.3 Generalise `DropTarget` with the frozen kinds (`reorder`, `into-group`,
  `above-group`/`below-group`, `on-strip`, `new-column-left`/`new-column-right`,
  `outside`) and extend `resolveDrop` with the column pass (edge/new-column
  margin, into-group, above/below), for normal and iconic modes and for float
  and non-float drags
- [x] 3.4 Route `commitDrop` by payload and target kind: a `new-column-*` target
  allocates a `PanelColumn` on that side through the frame and places the group;
  `into-group` inserts a tab (compact included); `above`/`below` inserts a
  boundary group (compact included)
- [x] 3.5 Extend the single indicator (`panelDropIndicator`/strip indicator) with
  the new-column full-height mark at the workspace edge; clear it on
  commit/cancel/leave; keep the M41/M42 look
- [x] 3.6 Remove a column when it holds no groups, re-apply the splitter stretch
  factors, and wire each new column's `stateChanged`/menu signals like the
  initial one; target `Window > Panels` at the owning column
- [x] 3.7 C++ `m43_newcolumn` (exit **143**): dropping left/right of a column,
  beside a group at the edge, or beside the Tools toolbar creates a new column on
  that side and places the group; an emptied column is removed
- [x] 3.8 C++ `m43_intogroup` (exit **144**): a drop inside a group inserts a tab
  in normal mode and in compact mode
- [x] 3.9 C++ `m43_boundary` (exit **145**): a drop above/below a group inserts a
  boundary group in normal mode and in compact mode
- [x] 3.10 C++ `m43_singlefloat` (exit **146**): a one-panel float carries only
  its panel and re-docking restores the group with the other panels

## 4. Phase C — session v6 and self-tests

- [x] 4.1 Extend `session.{h,cpp}` to schema v6: `panelColumns` (per column
  `side`/`order` with the v5 `panelGroups` shape nested under `groups`) with
  per-key defaults; synthesise a single right-hand column from the legacy
  top-level `panelGroups` when `panelColumns` is absent; keep the load-then-write
  and unknown-key-preservation path
- [x] 4.2 Persist the column layout on every change (each column's
  `stateChanged`) and restore it at startup; keep `panelRailMode`, `railWidth`,
  `autoCollapseIconic`, `autoShowHidden`, and the v4 keys intact
- [x] 4.3 C++ `m43_session` (exit **151**): the per-column layout round-trips, a
  v5 store loads a single right-hand column, and an unknown key survives
- [x] 4.4 Wire the `m43_*` steps into `crates/pictura-app/cpp/main.cpp` at codes
  **140–151**, pumping a bounded event loop before reading geometry/visibility
  (the M40 Wayland-popup lesson; `--self-test` pins xcb) and `std::fflush(stderr)`
- [x] 4.5 Run `xvfb-run -a ./build/pictura --self-test` and the
  `two_layers.psd` variant, both exit 0; confirm every earlier m20–m42 check is
  unchanged except any tear-off/dock assertions the multi-column host necessarily
  adapts

## 5. Verification and close-out

- [x] 5.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo test --workspace` (expected unchanged — no Rust change)
- [x] 5.2 `cmake -S . -B build && cmake --build build`
- [x] 5.3 `openspec validate m43-panel-multicolumn --strict`;
  `openspec validate --all --strict`
- [x] 5.4 `git status --porcelain` is scoped to this proposal (the M43 code plus
  docs and OpenSpec; no unrelated files)
- [x] 5.5 Record the M43 proposal in `docs/dev/STATE.md` (capability count stays
  60 after archive) — STATE is handled separately
- [ ] 5.6 Archive the change (`openspec archive m43-panel-multicolumn`) and
  commit — deferred; this task does not commit
