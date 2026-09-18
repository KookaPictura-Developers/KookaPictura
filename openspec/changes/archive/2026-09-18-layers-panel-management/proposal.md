## Why

The Layers panel can create, group, filter, and edit a stack, but it cannot
perform any of CS6's structural management operations. Merge Down/Layers/Visible,
Flatten, Layer via Copy/Cut, Convert Background, Select Similar/Linked, link
sets, Delete Hidden Layers, Hide Layers, the New Layer/Group dialogs, and the
Rasterize variants are all missing, and the `Layer` menu still exposes them as
disabled placeholders. This is the remaining "layer management" stage of the
layers-panel program; merge/flatten land first because they are the only
destructive operations and the rest of the stage recomputes the composite
around them.

## What Changes

- **Merge / Flatten.** `Layer > Merge Down` / `Merge Layers` is one
  selection-dependent command (`Ctrl+E`): one selected layer is Merge Down,
  several is Merge Layers. `Merge Visible` (`Shift+Ctrl+E`) merges exactly the
  eye-visible layers, `Merge Clipping Mask` collapses a clipping group into a
  raster base, and `Flatten Image` composites every visible layer over an opaque
  white full-document backdrop and replaces the tree with one Background node,
  discarding hidden layers without a prompt. Each is one undoable step.
- **New Layer/Group dialog.** Alt-clicking the New Layer/Group button, or the
  `Layer > New > Layer…` / `Group…` menu item, opens a modal collecting Name,
  Color label, blend Mode, Opacity, Fill with mode-neutral color, and
  Use-Previous-Layer-to-Create-Clipping-Mask (clipping not offered for groups).
- **First-class Background.** `pictura_core::Layer` gains an explicit background
  flag; the codec derives it on read and writes it on save, and `is_background`
  uses the flag instead of the index-0 + `name == "Background"` heuristic. Adds
  `Layer from Background…` and `Background From Layer`.
- **Layer via Copy / Cut.** `Ctrl+J` / `Shift+Ctrl+J` copy or cut the active
  selection's pixels to a new layer above the current one; Cut additionally
  clears the source. Both require an active selection.
- **Select / link / hide.** Select Similar matches the active layer's kind and
  attributes, Select Linked selects the active layer's link set, link/unlink
  commands manage set membership, Delete Hidden Layers removes every hidden
  layer, and Hide Layers hides the selected layers.
- **Rasterize subset.** `Fill Content` (a fill layer's content becomes pixels)
  is implemented. `Type`, `Shape`, `Vector Mask`, `Smart Object`, `Video`, and
  `3D` stay disabled with a documented reason: those layer kinds do not exist in
  the model yet. `Rasterize Layer` / `All Layers` are limited to the kinds that
  do.
- **Drag-reorder drop rules.** The deferred rules for dropping a row into and
  out of a group are completed: validate before commit, reject invalid drops,
  and make each accepted drop one undoable step.

## Capabilities

### New Capabilities

- `layer-management`: the structural management commands (merge, flatten, New
  Layer/Group dialog, Background conversion, Layer via Copy/Cut, select/link,
  hide/delete-hidden, the rasterize subset) and their merge algorithm contract.

### Modified Capabilities

- `layers-panel`: the panel and menus wire and enable the new management
  commands and the New Layer/Group dialog, and finish the drag-reorder drop
  rules.
- `command-registry`: the new `layer.*` command identifiers and their handlers
  and enablement providers are added to the declarative table, and the disabled
  Layer-menu placeholders become implemented.

## Impact

- New `crates/pictura-render/src/document_ops/layer_ops/merge.rs` —
  `merge_scope`, `flatten`, and the `can_merge_target` / `is_visible_in_panel`
  validators, reusing `crates/pictura-render/src/composite.rs::composite_rgba`.
- `crates/pictura-core/src/lib.rs` — the `Layer` background flag; clone/literal
  sites updated.
- `crates/pictura-codec/` — derive the background flag on read and write it on
  save.
- `crates/pictura-app/src/cxxqt_object/impl_layers.rs` + `cxxqt_object.rs` — the
  new bridge qinvokables.
- `crates/pictura-app/cpp/commands.h` — the new `layer.*` command ids.
- `crates/pictura-app/cpp/frame_menus.cpp` — handlers and enable providers.
- `crates/pictura-app/cpp/command_tree.cpp` — wire the disabled Layer-menu
  leaves.
- `crates/pictura-app/cpp/panels/layers_panel.*` — the New Layer/Group dialog
  and the drag-reorder drop rules.
- Tests: Rust unit tests for node structure and pixels, at least one
  external-oracle composite test (psd-tools / ImageMagick, self-skipping when
  absent), and app self-tests per command.
- No new dependencies. Link-set serialization is explicitly out of scope
  (`LAY-002` open question): link sets are session/local state only.
