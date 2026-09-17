## 1. Phase 0 — brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m44-panel-theme-polish.md`: the user's requests (E1,
  T1–T2, W1–W6, C1–C4, F1–F2, S1–S2), the current-state inventory with file
  refs, the frozen design (scoped theme by `objectName`, droplet/parity model,
  chevron/tab-colour/default-active corrections, float-drag continuation,
  any-side docking and compact group-relative drop placement, Tools fixed float
  height, the E1 diagnosis protocol), the test hooks, the **153–166** exit-code
  table, the honest limits, and the Layers-panel program shift to M45–M48
- [x] 1.2 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `panel-column`, `application-shell`, `tool-framework`, and `document-canvas`
  deltas; freeze the scoped `objectName` family, the corrected chevron/tab
  mapping, the compact group-relative drop grammar, the any-side dock target,
  and the E1 seam in `design.md`
- [x] 1.3 Validate: `openspec validate m44-panel-theme-polish --strict` and
  `openspec validate --all --strict`

## 2. Phase A — new-document canvas bug (E1: diagnose + fix + regression)

- [x] 2.1 Reproduce E1 empirically: run `--headless --self-test` and an
  `xvfb-run`/GPU run, capture the first present, and record the failing frame
- [x] 2.2 Narrow the seam: compare `view.image` immediately after
  `new_document` with a second `current_buffer`/`buffer_to_image` and with
  `doc.composite`; determine whether the garbage comes from the GPU
  first-use `composite_active`/`composite_gpu` path or the Interop present path
- [x] 2.3 Record the named root cause in the brief and design before editing
- [x] 2.4 Fix the root cause at the identified seam (`cxxqt_object.rs` or
  `gpu.rs`), not at a single caller; keep the CPU oracle unchanged
- [x] 2.5 Add one runnable Rust regression test that composites a fresh white
  document twice and asserts both frames are uniform opaque white and equal
  (skipping the GPU arm when no adapter exists)
- [x] 2.6 Keep the M17 pixel check as the lower-layer guard and confirm it
  still passes

## 3. Phase B — theme, borders, and style unification (W1, W2, W3, W6, C2, C4, F1, F2, S1, S2, C1)

- [x] 3.1 Swap the column collapse/expand chevron icons in
  `updateColumnToggle` so each mode shows the icon for the action it performs
- [x] 3.2 Investigate which artefact carries the last-active default (group
  default tab, column current panel, or default page) and make the **first**
  panel active on a fresh session
- [x] 3.3 Correct the scoped `#panelTabBar` active/inactive background swap in
  `theme.cpp` and correct the M43 `m43_tabcolors` direction in place
- [x] 3.4 Thicken and darken the inter-group splitter handle and re-point the
  compact `panelIconDivider` from white to the darker `${border}`
- [x] 3.5 Remove the inline flyout stylesheet and give `PanelGroup`,
  `PanelFlyout`, and `PanelFloat` the same scoped selectors, margins, and
  borders so all three presentations match
- [x] 3.6 Scope the document tab-strip selectors: a dark grey right border and
  no extra top border (`QTabWidget#documentTabs`/`QTabBar#documentTabBar`)
- [x] 3.7 Add the darker grey panel/column border to the Tools toolbar and the
  normal and compact widget panels, with the thickness/colour as single named
  constants (mark unsourced constants `ponytail:`)
- [x] 3.8 Make strip labels elide and appear as soon as any room exists,
  replacing the fixed `>= 120 px` gate

## 4. Phase C — drag and dock (T1, T2, W4, W5, C3)

- [x] 4.1 Keep the tab drag alive after `createFloat`: `updateDrag` continues to
  move the float, and the commit/re-dock happens only on `dragFinished`
  (release) or is cancelled on `dragCanceled`
- [x] 4.2 Extend `resolveDrop`/`resolveIconicDrop`/`commitDrop` so a whole
  widget column resolves to a new-column/boundary target on any side of the
  Tools toolbar, another widget panel or column, and the workspace, reusing the
  existing `DropTarget` kinds and the single indicator (no second drag system)
- [x] 4.3 Add compact group-relative placement: onto/just above/below a group
  inserts into it; between groups, above the top, or below the bottom creates a
  new group
- [x] 4.4 Add the compact drag-handle dots affordance above each group of icons
  that drags the whole group, and give each icon group one shared
  background/border so it reads as a unit
- [x] 4.5 Widen the Tools dock's allowed areas to left/right/top/bottom, keep
  the fixed content size per orientation, and fix the floating height so it is
  not drag-resizable; keep the M40 no-tabification fallback and the M42
  trailing-stretch behaviour

## 5. Phase D — self-tests and close-out prep

- [x] 5.1 Add the `m44_*` self-test steps in `crates/pictura-app/cpp/main.cpp`
  at exit codes **153–166** (newdoc 153, chevrons 154, defaultactive 155,
  tabswap 156, floatdrag 157, docksides 158, divider 159, popupstyle 160,
  compactdivider 161, compactdrop 162, draghandle 163, elide 164, filebar 165,
  panelborder 166); pump the event loop bounded, force layout so geometry is
  real offscreen, and `std::fflush(stderr)`
- [x] 5.2 Add the test hooks the checks need (e.g.
  `collapseIconRoleForTest`, `defaultActiveTabForTest`,
  `panelTabColoursForTest`, `groupDividerMetricsForTest`,
  `floatDragTrackingForTest`, `dockSidesForTest`, `toolboxFloatHeightForTest`,
  `compactDividerColourForTest`, `compactDropKindForTest`,
  `dragHandleForTest`, `stripLabelElideForTest`, `documentTabStripBordersForTest`,
  `panelBorderForTest`)
- [x] 5.3 Run `./build/pictura --headless --self-test` and confirm exit 0
  (or the expected code) and that every earlier `m20_*`–`m43_*` check is
  unchanged except the corrected `m43_tabcolors` direction
- [x] 5.4 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo nextest run --workspace` (the E1 Rust regression must
  run); `cargo test --workspace --doc`
- [x] 5.5 `cmake -S . -B build -G Ninja && cmake --build build --parallel`
- [x] 5.6 `openspec validate m44-panel-theme-polish --strict`;
  `openspec validate --all --strict`
- [x] 5.7 `git status --porcelain` is scoped to this proposal (the M44 code plus
  docs and OpenSpec; no unrelated files); record the capability count stays
  **60** after archive
- [x] 5.8 Record M44 in `docs/dev/STATE.md` and shift the program line to M45
  filtering/search, M46 management, M47 styles/effects, M48 smart objects —
  STATE is handled at close-out
- [ ] 5.9 Archive the change (`openspec archive m44-panel-theme-polish`) and
  commit — deferred; this task does not commit
