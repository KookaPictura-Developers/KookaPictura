## 1. Phase 0 — brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m45-panel-fixes.md`: the user's requests (T1–T3,
  W1–W8, C1–C2), the current-state inventory with file refs, the frozen design
  (resolver-driven indicator, one emptied-column cleanup path, group-height
  minimize, one toolbar content formula, left/right-only toolbar placed beside a
  column, one shared minimum-width floor with no clipping, the whole-group
  compact popup), the test hooks, the **167–179** exit-code table, the honest
  limits, and the Layers-panel program shift to M46–M49
- [x] 1.2 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `panel-column`, `application-shell`, and `tool-framework` deltas; freeze the
  owning-column indicator rule, the cleanup path, the minimize height/label,
  the toolbar content formula and central-pane hosting, the shared minimum
  width, and the whole-group popup in `design.md`
- [x] 1.3 Validate: `openspec validate m45-panel-fixes --strict` and
  `openspec validate --all --strict`

## 2. Phase A — Tools toolbar (T1–T3)

- [x] 2.1 Add a `contentHeight(columns)` derived from the same grid metrics as
  `contentWidth(columns)`, and recompute both fixed axes in
  `Toolbox::updateContentMetrics` after `reflow()` and a layout activation,
  releasing the stale axis before re-fixing it
- [x] 2.2 Funnel `setColumns`, `dockLocationChanged`, and `topLevelChanged`
  through the recompute so a 1↔2 switch cannot leave a stale width or height
- [x] 2.3 Restrict the Tools dock's allowed areas to left/right only, removing
  top/bottom
- [x] 2.4 Resolve a floating-Toolbar drop through the existing column grammar
  (`resolveDrop`/`columnEdgeAnchorAt`) in `PicturaMainWindow`, show the single
  `#2a7fff` indicator at the resolved boundary, and place the toolbar as a
  fixed-width central-splitter pane beside the anchor column; fall back to the
  left/right dock areas when no column boundary is under the pointer
- [x] 2.5 Keep the M40 no-tabification, fixed-content-size, move, float, and
  close contracts

## 3. Phase B — widget panel indicator, minimize, emptying, sizing (W1–W8)

- [x] 3.1 Carry an owning-column identity on the cross-column `DropTarget` and
  render `showIndicatorFor` through that owner, so the line is drawn in the
  target column (W1, W2)
- [x] 3.2 Position a tab insert at `tabInsertionX(tabIndex)` so the rightmost
  tab draws at its own index, not x=0 (W3)
- [x] 3.3 Position a bottom-boundary insert at the last visible group's bottom
  edge (W6)
- [x] 3.4 Add a single `maybeRemoveSelf()` that calls
  `frame->removeColumnIfEmpty(this)` and invoke it after every path that can
  empty a dynamic column: commit, `closeGroup`, `showPanel(name,false)`, and the
  flyout restore (W4)
- [x] 3.5 Clamp the `PanelGroup` widget's height to the tab-bar height when
  minimized and restore the saved maximum on expand (W5)
- [x] 3.6 Make the tab menu entry state-derived: `Expand Panel` while
  minimized, `Minimize` otherwise, replacing the literal in `buildTabMenu` and
  `kTabMenuTexts` in place (W5)
- [x] 3.7 Replace the per-column `widest`-derived minimum with one shared
  `kPanelMinWidth` floor used by every normal-mode widget column; keep the
  iconic strip's narrow minimum (W8)
- [x] 3.8 Change the column scroll area's horizontal policy to `AsNeeded` and
  keep tab eliding and the reserved corner-button width so the right side is
  reachable rather than clipped (W7)

## 4. Phase C — compact parity and indicator (C1–C2)

- [x] 4.1 Change `openIconFlyout` to take the whole `PanelGroup` out of the
  column splitter (remembering its index), set the clicked panel current, and
  host it in the `Qt::Popup`; restore the group at its original index on close
  (C1)
- [x] 4.2 Shrink `PanelFlyout` to a frameless popup host and delete the
  bespoke one-tab header, so docked, popup, and floating all present the same
  `PanelGroup` with no parity differences (C1)
- [x] 4.3 In compact mode draw a whole-group boundary line at the top of the
  group's container, above the drag-handle dots, not above the first icon
  button (C2)

## 5. Phase D — self-tests and close-out prep

- [x] 5.1 Add the `m45_*` self-test steps in `crates/pictura-app/cpp/main.cpp`
  at exit codes **167–179** (sizing 167, sides 168, beside-column 169,
  indicator-side 170, cross-column 171, rightmost-tab 172, empty-column 173,
  minimize 174, bottom 175, no-clip 176, min-width 177, popup-group 178,
  compact-line 179); pump the event loop bounded, force layout so geometry is
  real offscreen, and `std::fflush(stderr)`
- [x] 5.2 Add the test hooks the checks need (e.g.
  `toolbarContentSizeForTest`, `toolboxAllowedSidesForTest`,
  `toolboxColumnDockForTest`, `dropIndicatorMatchesTargetForTest`,
  `crossColumnIndicatorForTest`, `rightmostTabIndicatorForTest`,
  `dynamicColumnCountForTest`, `minimizedGroupHeightForTest`,
  `tabMenuTextsForTest`, `bottomIndicatorForTest`,
  `contentInsideViewportForTest`, `minimumWidthFloorForTest`,
  `iconFlyoutGroupForTest`, `compactGroupIndicatorForTest`)
- [x] 5.3 Run `./build/pictura --headless --self-test` and confirm exit 0 (or
  the expected code) and that every earlier `m20_*`–`m44_*` check is unchanged
- [x] 5.4 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo nextest run --workspace`; `cargo test --workspace --doc`
- [x] 5.5 `cmake -S . -B build -G Ninja && cmake --build build --parallel`
- [x] 5.6 `openspec validate m45-panel-fixes --strict`;
  `openspec validate --all --strict`
- [x] 5.7 `git status --porcelain` is scoped to this proposal (the M45 code plus
  docs and OpenSpec; no unrelated files); record the capability count stays
  **60** after archive
- [x] 5.8 Record M45 in `docs/dev/STATE.md` and shift the program line to M46
  filtering/search, M47 management, M48 styles/effects, M49 smart objects —
  STATE is handled at close-out
- [ ] 5.9 Archive the change (`openspec archive m45-panel-fixes`) and commit —
  deferred; this task does not commit
