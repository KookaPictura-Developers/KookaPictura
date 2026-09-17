## 1. Briefs and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m42-panel-refinements.md`: the twelve user-reported
  defects, the current-state inventory with file refs, the frozen design, the
  test hooks and 131–139 exit-code table, the honest limits, the non-goals, and
  the Layers-panel program shift (M43 filtering/search, M44 management, M45
  styles/effects, M46 smart objects)
- [x] 1.2 Write `docs/dev/m42-panel-menus.md`: the researched per-panel CS6
  panel menus with a provenance/confidence note, per-panel sources, the
  implemented/disabled mark per entry, and the Gradients/Patterns/Libraries
  non-goals
- [x] 1.3 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `panel-column`, `tool-framework`, and `application-shell` MODIFIED/ADDED
  deltas; freeze the constants, the compact-transition rule, the flyout
  placement/style, the in-window overlay semantics, and the per-widget menu
  contract in `design.md`
- [x] 1.4 Validate: `openspec validate m42-panel-refinements --strict` and
  `openspec validate --all --strict`

## 2. Phase A — minimum widths, icon sizes, Tools geometry, menu-bar overlay, fg/bg swap

- [x] 2.1 Give `PanelColumn` a bounded normal-mode minimum width and record the
  normal width on `setRailMode(true)`; set the column to the smallest strip
  width on entering `iconic` and restore the recorded width on returning to
  `normal`; amend the `m41_minwidth` check (code 121) to the new width floor
  while keeping the height/scroll clauses
- [x] 2.2 Raise the iconic-strip button and icon constants in
  `panel_column.cpp`, and the Tools slot and screen-mode icon pixmap sizes in
  `toolbox.cpp`; keep the existing `icons.h` asset ids
- [x] 2.3 Remove the body `addStretch(1)` from `Toolbox` so a floated dock hugs
  its content height, and trim the body margins so the one- and two-column
  widths carry no extra horizontal space; keep `updateContentMetrics()` as the
  width source
- [x] 2.4 Diagnose the widget over the menu bar: inspect
  `menuBar()->cornerWidget()` and every `findChildren<QToolBar*>()` for a global
  geometry intersecting the menu-bar row; remove the stray widget
- [x] 2.5 Guard `restoreState`: discard a persisted layout whose schema/version
  no longer matches the current chrome and fall back to the default arrangement,
  so nothing is restored over the menu bar
- [x] 2.6 Add the `ForegroundBackgroundWidget` swap control (double-headed
  arrow) exchanging foreground/background, and wire `X` in the frame to swap
  when no tool shortcut claims it; keep the existing corner reset
- [x] 2.7 C++ `m42_minwidth` (exit **131**): the normal-mode column cannot be
  squeezed below its minimum width, and the compact transition lands at the
  smallest strip width
- [x] 2.8 C++ `m42_tools` (exit **139**): the tool icons are larger, the
  one-/two-column widths differ from the content only by the body margins, and a
  floated Tools dock's height hugs its content
- [x] 2.9 C++ `m42_menubar` (exit **138**): no child widget geometry intersects
  the menu-bar row, and a stale persisted layout is discarded
- [x] 2.10 C++ `m42_fgbg` (exit **137**): the swap control exchanges the
  colours and the `X` key does too

## 3. Phase B — iconic-strip drag and reorder

- [x] 3.1 Make the strip icon buttons drag sources that reuse
  `beginPanelDrag`/`updateDrag`/`commitDrop`; a reorder writes the order through
  `PanelGroup::setPanelOrder` (the v5 `panelGroups` order)
- [x] 3.2 Route a strip-icon drop on the normal-mode group stack through the
  existing `resolveDrop` group/boundary targets so the panel joins the target
  group and the column shows the group stack
- [x] 3.3 Add test hooks: `beginStripDragForTest`, `stripOrderForTest`,
  `dropStripOnGroupForTest`
- [x] 3.4 C++ `m42_dragstrip` (exit **133**): a strip icon reorders within the
  strip and persists, and dropping a strip icon on the group stack moves the
  panel into that group

## 4. Phase C — compact flyout refinement

- [x] 4.1 Position the flyout on the inner side of the column (left of a
  right-hand strip, right of a left-hand strip), querying the column's side
- [x] 4.2 Render the icon whose flyout is open in the active/pressed state and
  clear it when the flyout closes
- [x] 4.3 Restyle the single `PanelFlyout` as a panel group: a header naming the
  panel above the detached content, with a close icon button (double right
  chevron) at the right end of the header; do not add a second flyout system
- [x] 4.4 Add test hooks: `flyoutSideForTest`, `activeIconForTest`,
  `flyoutHeaderTitleForTest`, `flyoutHeaderCloseForTest`
- [x] 4.5 C++ `m42_flyout` (exit **134**): the flyout opens on the inner side
  for a right-hand column, its header names the panel and carries the close
  button, and the open icon renders active
- [x] 4.6 C++ `m42_iconic` (exit **132**): the strip icons are larger than the
  M41 sizes and the open icon is the active/pressed one (the active/pressed
  clause is asserted in `m42_flyout`/134 rather than amending the green 132)

## 5. Phase D — per-widget header action button and menus

- [x] 5.1 Add the tab-header context-action button at the far right of each
  `PanelGroup`, showing the current panel's per-widget menu
  (`currentPanelName()`)
- [x] 5.2 Add the per-panel menu tables from `docs/dev/m42-panel-menus.md`
  (new `panel_menus.{h,cpp}` or folded into the header button) with an
  implemented flag per entry; disable unimplemented entries with the
  `<label> — not implemented yet` tooltip and disable submenu parents whose
  children are all disabled
- [x] 5.3 Keep `Close` and `Close Panel Group` off the per-widget menu (they stay
  on the tab context menu); route the Layers `Panel Options…` entry to the
  existing `LayersPanel::openPanelOptions` behaviour
- [x] 5.4 Add test hooks: `headerMenuButtonForTest`, `panelMenuTextsForTest`,
  `panelMenuEnabledForTest`, `triggerPanelMenuForTest`
- [x] 5.5 C++ `m42_widgetmenu` (exit **135**): the header button shows the
  current panel's menu, unimplemented entries are disabled with the tooltip,
  neither `Close` nor `Close Panel Group` appears, and Layers `Panel Options…`
  opens

## 6. Phase E — in-window floating overlay

- [x] 6.1 Replace `PanelFloat`'s `Qt::Tool` top-level window with a child
  overlay of the main window (frameless, raised, movable by its tab bar)
- [x] 6.2 Clip the overlay to the main window bounds and clamp its movement so
  it cannot be dragged outside; keep the existing `applyGroupDrop` re-dock path
- [x] 6.3 Add/update test hooks: `floatIsWindowForTest`, `floatClampedForTest`,
  `redockForTest` (reused)
- [x] 6.4 C++ `m42_float_overlay` (exit **136**): a torn-off group is an
  in-window child overlay (not a top-level window), stays within the main
  window, and re-docks on drop

## 7. Phase F — session additions and self-tests

- [x] 7.1 Confirmed no session-schema bump is needed: strip reorder writes the
  existing v5 `panelGroups` order; the menu-bar guard adds a `layoutRevision`
  key but the schema stays v5 (a persisted float position would be a v6)
- [x] 7.2 Wire the `m42_*` steps into `crates/pictura-app/cpp/main.cpp` at codes
  **131–139**, pumping the event loop bounded before reading geometry/visibility
  (the M40 Wayland-popup lesson; `--self-test` pins xcb), and `std::fflush(stderr)`
- [x] 7.3 Run `xvfb-run -a ./build/pictura --self-test` and the
  `two_layers.psd` variant, both exit 0; confirm every earlier m20–m41 check is
  unchanged except the amended `m41_minwidth` (121) assertion and any tear-off
  flags the new overlay changes

## 8. Verification and close-out

- [x] 8.1 `openspec validate m42-panel-refinements --strict`;
  `openspec validate --all --strict`
- [x] 8.2 `git status --porcelain` is scoped to this proposal (the M42 code plus
  docs and OpenSpec; no unrelated files)
- [x] 8.3 Record the M42 proposal in `docs/dev/STATE.md` (capability count stays
  60 after archive) — STATE is handled separately
- [ ] 8.4 Archive the change (`openspec archive m42-panel-refinements`) and
  commit — deferred; this task does not commit
