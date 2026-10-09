# Proposal

## Why

The Layer menu still ships several ordering/merge commands as disabled stubs:
the four `Layer > Arrange` leaves (`command_tree.cpp`) have no handlers,
`Arrange > Reverse` is absent entirely, `Delete Layer` is greyed, and there is
no distinct `Merge Down` or any stamp command. Users can reorder, delete, and
stamp visible content in CS6; the engine already has the ordering and merge
primitives, so the remaining work is wiring and two small ops.

## What Changes

- Wire the four `Layer > Arrange` leaves to the existing within-container move
  primitive with frozen ids: Bring to Front, Bring Forward, Send Backward, Send
  to Back. Each is one undo state.
- Add `Layer > Arrange > Reverse`: reverse the stacking order of the selected
  contiguous run inside its container (new engine op + bridge + handler).
- Add a distinct `Layer > Merge Down` leaf that merges the active layer with the
  layer directly below it, reusing `MergeScope::Down`. `Ctrl+E` remains Merge
  Layers.
- Wire the greyed `Layer > Delete Layer` leaf to the existing `delete_layers`
  bridge command over the panel selection.
- Add `Layer > Stamp Visible` (`Ctrl+Alt+Shift+E`) and `Layer > Stamp Selected`
  (`Ctrl+Alt+E`): composite the visible (or selected) layers into a NEW raster
  layer above the active layer, leaving the originals intact. Non-destructive;
  built from the existing composite + `add_raster_layer_from_rgba` primitives.
  Each is one undo state.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `compositing/layer-management`: add requirements for within-container
  arrange/reverse, a distinct Merge Down, layer deletion, and non-destructive
  Stamp Visible / Stamp Selected.

## Impact

- Engine: `crates/pictura-render/src/document_ops/layer_ops/` (new
  `arrange_path`/`reverse_paths` in `properties.rs`, `stamp_scope` in
  `merge.rs`).
- Bridge: `crates/pictura-app/src/cxxqt_object.rs` +
  `src/cxxqt_object/impl_layers.rs` / `impl_layers_merge.rs`.
- C++: `cpp/commands.h` (frozen ids), `cpp/command_tree.cpp` (leaves),
  `cpp/frame_menus.cpp` (handlers + enabled providers).
- Tests: engine unit tests, a Qt Test case under `cpp/tests/`.
- No new dependencies. No `docs/` changes.
