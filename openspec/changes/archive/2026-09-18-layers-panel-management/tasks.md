## 1. Merge and Flatten (land and verify first)

- [ ] 1.1 Add `layer_ops/merge.rs` with `MergeScope`, `MergeError`, `MergeOutcome`, `merge_scope`, `flatten`, `can_merge_target`, `is_visible_in_panel`; re-export from `layer_ops/mod.rs`.
- [ ] 1.2 Implement Merge Down: target validation, union-rect scratch composite reuse (`composite_rgba`), lower-layer name/blend/opacity inheritance, node replacement and sibling/parent repair.
- [ ] 1.3 Implement Merge Layers (selection, topmost placement, Normal/100 result) and Merge Visible (eye-visible inputs, hidden left in place, visible selection required).
- [ ] 1.4 Implement Merge Clipping Mask (raster-base check, clipping coverage folded into alpha) and Flatten (discard hidden, opaque white backdrop, single Background node).
- [ ] 1.5 Rust unit tests: node count/order/name/blend/opacity/rect and pixel equality with `composite_rgba` for Down/Layers/Visible/Flatten; refusal cases (adjustment target, no layer below, non-raster clipping base).
- [ ] 1.6 External-oracle composite test (psd-tools / ImageMagick) for a merged pair, self-skipping when the tool is absent.
- [ ] 1.7 Bridge qinvokables `merge_layers(paths)`, `merge_visible(path)`, `merge_clipping_mask(path)`, `flatten_image()` in `impl_layers.rs` + `cxxqt_object.rs`; each recomposite then one undo state.
- [ ] 1.8 Command ids in `commands.h`; wire the disabled `Layer` merge/flatten leaves in `command_tree.cpp`; handlers and enable providers in `frame_menus.cpp`.
- [ ] 1.9 App self-test for merge Down/Selected/Visible/Clipping and flatten (white backdrop, hidden discard, one undo).

## 2. New Layer and New Group dialog

- [ ] 2.1 `NewLayerSpec` and attribute-carrying `add_layer_full` / `add_group_full` in `layer_ops/create.rs`; mode-neutral color lookup by `BlendMode` with the documented transparent default.
- [ ] 2.2 `LayerNewDialog` collecting Name, Color, Mode, Opacity, Fill-with-mode-neutral, and Use-Previous-Layer-to-Create-Clipping-Mask (clipping hidden for groups).
- [ ] 2.3 Bridge `new_layer_dialog(...)` / `new_group_dialog(...)`; open from `Alt`-click and the `Layer > New > Layer…` / `Group…` menu items; one undo state each.
- [ ] 2.4 Rust test for the neutral-color lookup table; self-test for dialog create and clipping-not-offered-for-groups.

## 3. Background flag and conversion commands

- [ ] 3.1 Add `Layer.background: bool` to `pictura-core`, update every clone/literal site.
- [ ] 3.2 Codec: derive the flag on read (bottom top-level non-group named `Background`) and write the `Background` name for a flagged layer; round-trip and psd-tools oracle test.
- [ ] 3.3 `is_background` reads the flag instead of the index/name heuristic; update `is_background_uses_default_heuristic`.
- [ ] 3.4 Engine + bridge `layer_from_background(path)` (clear flag, unlock) and `background_from_layer(path)` (set flag, opaque background color, move to bottom; refuse group/already-background).
- [ ] 3.5 Wire `layer.new.layerFromBackground` / `layer.new.backgroundFromLayer`; self-test both directions and the flag independence from position/name.

## 4. Layer via Copy and Layer via Cut

- [ ] 4.1 Engine copy/cut of the active selection's pixels to a new layer above the current one; Cut clears the source; require a selection.
- [ ] 4.2 Bridge `layer_via_copy()` / `layer_via_cut()`; wire `layer.new.layerViaCopy` / `layer.new.layerViaCut` with `Ctrl+J` / `Shift+Ctrl+J`.
- [ ] 4.3 Rust test preserving pixels outside the selection; self-test copy, cut-clears-source, and no-selection refusal.

## 5. Select Similar / Linked, link sets, Delete Hidden, Hide Layers

- [ ] 5.1 Transient link-set map and `link_layers` / `unlink_layers` / `select_similar` / `select_linked` in the bridge; clear on structural operations (documented limitation).
- [ ] 5.2 `delete_hidden_layers()` (no history when nothing hidden) and `hide_layers(paths)`.
- [ ] 5.3 Wire `layer.select.similar`, `layer.select.linked`, `layer.link.layers`, `layer.unlink.layers`, `layer.delete.hiddenLayers`, `layer.hide.layers`.
- [ ] 5.4 Self-test: similar-kind selection, link → select-linked → unlink, delete hidden removes only hidden, hide hides the selection.

## 6. Rasterize subset and disabled variants

- [ ] 6.1 Fill-content rasterizer: render a decoded fill-content layer (`SoCo`/`GdFl`/`PtFl`) to a pixel node and clear the fill data; refuse non-decodable payloads.
- [ ] 6.2 `Rasterize Layer` / `All Layers` limited to existing kinds (pixel, group, adjustment, fill content); wire `layer.rasterize.fillContent`, `layer.rasterize.layer`, `layer.rasterize.allLayers`.
- [ ] 6.3 Leave Type / Shape / Vector Mask / Smart Object / Video / 3D in the table with no handler and disabled; document the missing-kind reason.
- [ ] 6.4 Rust test for fill-content pixels and adjustment-data clear; self-test each enabled rasterize and the disabled variants.

## 7. Drag-reorder drop rules

- [ ] 7.1 Validate candidate drops during `dragMoveEvent` via the engine refusal predicate; show the indicator only for valid targets; reject invalid drops without calling the bridge.
- [ ] 7.2 Support dropping a row on the empty viewport to move it to the document root (out of a group); confirm `move_path_to` accepts the root target.
- [ ] 7.3 Self-test `lpr_drop_out` and `lpr_drop_rules` (invalid target rejected before commit, no history).

## 8. Verification

- [ ] 8.1 `cargo fmt --all`; `cargo clippy --all-targets -- -D warnings`; `cargo nextest run --workspace`; `cargo test --workspace --doc`.
- [ ] 8.2 CMake app build; `./build/pictura --headless --self-test` and the `.psd` run exit 0 with the new step labels.
- [ ] 8.3 Fill-content and codec external-oracle tests run for real in the `oracles` job (and self-skip locally when absent).
- [ ] 8.4 `openspec validate layers-panel-management --strict`; `openspec validate --all --strict`.
