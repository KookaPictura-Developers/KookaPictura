## Why

M41–M44 built, then refined, the custom `PanelColumn`/`PanelGroup`/
`PanelFlyout`/`PanelFloat` workspace: a multi-column host, session v6, an
in-window float overlay, a compact/iconic strip, per-widget menus, and a
scoped panel/theme polish. The user has handed back a seventh lived-in
correction list: thirteen defects across the Tools toolbar's column sizing and
docking, the docked widget panels' drop indicator, minimize, emptying, and
sizing, and the compact strip's popup parity and group-drop indicator. This pass
is the seventh panel interruption; the Layers-panel program shifts again to
**M46** filtering/search, **M47** management, **M48** styles/effects, and
**M49** smart objects.

## What Changes

- **Tools toolbar (T1–T3).** Switching the toolbar between one and two columns
  SHALL size it exactly to content in both width and height, recomputing the
  fixed-axis locks on `setColumns` and on dock/float, so neither mode is cut or
  over-tall. Top/bottom docking SHALL be disallowed again (left/right only). The
  floating toolbar SHALL be placeable on any side of a widget panel/column,
  wherever the columns are docked (left of a right column, between columns,
  etc.), by resolving the drop through the existing column grammar and hosting
  it at that central-splitter boundary, with the single blue indicator shown.
- **Docked widget panels (W1–W8).** The drop indicator SHALL be positioned from
  the same resolved `DropTarget` the commit uses, in the column that owns the
  target: correct side (W1), shown for cross-column drops (W2), at the rightmost
  tab index rather than the far left (W3), and at the bottom boundary (W6). A
  dynamic column emptied by any path SHALL be removed (W4). A minimized group
  SHALL occupy only its tab-bar height with the content hidden, and its menu
  entry SHALL read **"Expand Panel"** while minimized (W5). The right side SHALL
  never be clipped (W7), and every widget column SHALL share one minimum-width
  floor it cannot shrink below (W8).
- **Compact/iconic panel (C1–C2).** Clicking an icon SHALL open the **whole
  `PanelGroup`** (all the group's tabs, clicked panel active) — the same widget
  docked, in the popup, and floating — with no parity differences. Dragging a
  whole group in compact mode SHALL draw the placement line above the group's
  drag-handle dots, not inside the group.
- **Self-tests.** New `m45_*` checks at exit codes **167–179**; the driver pumps
  the event loop bounded and forces layout so geometry is real while headless.

## Capabilities

### New Capabilities

None. M45 refines requirements already owned by `panel-column`,
`application-shell`, and `tool-framework`.

### Modified Capabilities

- `panel-column`: the drop indicator is rendered from the resolver's target in
  the owning column (correct side, cross-column, rightmost tab, bottom
  boundary, above compact group dots); a dynamic column emptied by any path is
  removed; minimize collapses the group to its tab bar and the menu label
  toggles; every widget column shares one minimum-width floor and never clips;
  the compact popup hosts the whole `PanelGroup` for docked/popup/float parity.
- `application-shell`: the headless self-test allocates the M45 checks at
  **167–179**; the Tools standalone dock is restricted to left/right main-window
  areas and can be placed beside any widget column through the central
  splitter.
- `tool-framework`: the Tools panel docks left/right only and beside any widget
  column, and its content sizing is recomputed from one content formula on every
  column-count or dock/float change.

No capability is added or removed; the canonical count stays at **60** after
archive.

## Impact

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}` — resolver-driven
  indicator placement and owning-column identity; the emptied-column cleanup
  path; the shared minimum-width floor and no-clip scroll policy; minimize
  height; the state-derived menu label; the whole-group compact popup; the
  `m45_*` test hooks.
- `crates/pictura-app/cpp/panels/panel_group.{h,cpp}` — the group-height
  minimize clamp and restore.
- `crates/pictura-app/cpp/toolbox.{h,cpp}` — the one-formula content width and
  height recompute, and the left/right-only allowed areas.
- `crates/pictura-app/cpp/frame.{h,cpp}` — the any-side toolbar pane resolution
  and the dynamic-column cleanup entry point.
- `crates/pictura-app/cpp/main.cpp` — the `m45_*` self-test steps, exit codes
  **167–179**.
- `docs/dev/m45-panel-fixes.md` (brief). No dependency, PSD, session, theme, or
  bridge-ABI change.
