# Proposal

## Why

`LayersPanel::populateRowMenu` (`crates/pictura-app/cpp/panels/layers_panel_menu.cpp:70`)
receives the row's `kind` but uses it for exactly one decision (whether to show
Export As / Quick Export). Every layer-type-specific action — masks, styles,
smart objects, shape, clipping, rasterize — therefore has no kind-aware home in
the row menu, so each upcoming Layers-panel feature would hand-edit the same
34-line function and conflict. This is the seam the rest of the Layers-panel
program plugs into.

## What Changes

- Replace the hardcoded row-menu body with a declarative per-kind row table:
  each entry carries an id, label, submenu, and a handler dispatch id.
- The row menu becomes type-aware: pixel, background, group, adjustment, type,
  shape, and smart-object rows each compose the rows that apply to their kind.
- Rows whose operation is not yet implemented are rendered **disabled** with the
  existing `— not implemented yet` tooltip convention (as the panel menu and
  bottom strip already do), so later changes enable them by flipping a flag
  rather than editing `populateRowMenu`.
- No new engine or bridge work: the table dispatches to existing
  `view_` operations and to the panel's own helpers.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities

- `ui/layers-panel`: the row context menu is assembled per layer kind, and
  planned-but-unimplemented rows appear disabled with a tooltip instead of
  being absent.

## Impact

- `crates/pictura-app/cpp/panels/layers_panel_menu.cpp` (rewrite of
  `populateRowMenu` and a new row-spec table + dispatcher).
- `crates/pictura-app/cpp/panels/layers_panel_menu.h` (table/dispatcher types).
- `crates/pictura-app/cpp/panels/layers_panel_test.cpp` (the enumeration test
  seam gains kind coverage).
- No dependency change.
