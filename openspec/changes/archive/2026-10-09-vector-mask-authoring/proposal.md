# Proposal

## Why

The engine already decodes a layer's `vmsk` block into a `VectorMask`
(`pictura_core::VectorMask`), the codec round-trips it, and the compositor
honours it. The codec and `shape_layer.rs` can even author one. But there is no
operation to add, delete, enable, link, or rasterize a *general* layer vector
mask, so the `Layer ▸ Vector Mask` menu, the row indicators, and the Properties
panel cannot offer vector-mask authoring. This change adds the engine and bridge
half of issue #110's vector-mask surface, mirroring the
`layer-mask-model-and-ops` + `layer-mask-ui` split (clipping masks already
shipped).

## What Changes

- Add vector-mask operations to `pictura_render::document_ops::layer_ops`:
  `add_vector_mask` (Reveal All / Hide All / Current Path), `delete_vector_mask`,
  `set_vector_mask_enabled`, `set_vector_mask_linked`, `rasterize_vector_mask`,
  and the read predicates `has_vector_mask`, `vector_mask_linked`,
  `vector_mask_disabled`.
- Author a new `vmsk` block for Reveal All / Hide All / Current Path, and patch
  the existing block's flag word in place for enable/disable and link/unlink so
  the original path geometry round-trips byte-for-byte.
- Record the PSD `vmsk` flag semantics: bit 1 invert, bit 2 not-linked, bit 3
  disabled (Adobe PSD spec, one-indexed), so `vector_mask_linked` reads
  `flags & 0x02 == 0`.
- `rasterize_vector_mask` folds the vector coverage into a raster `LayerMask`
  (multiplying an existing layer mask, per LAY-005) and drops the `vmsk` block.
- Expose all of the above through a new
  `cxxqt_object/impl_layers/vector_masks.rs` bridge module over the active layer,
  each mutation recompositing and recording one history state, plus per-row
  presence/link/disabled reads and a vector-mask thumbnail.
- Wire the `Layer ▸ Vector Mask` leaves and `Layer ▸ Rasterize ▸ Vector Mask`,
  the Layers delegate's vector-mask thumbnail / link glyph / red-X indicators,
  and a Properties panel Vector Mask section.
- No new dependencies. No `docs/` change.

## Capabilities

### New Capabilities

- `compositing/vector-masks`: creating (Reveal All / Hide All / Current Path),
  deleting, enabling/disabling, linking/unlinking, and rasterizing a layer vector
  mask, the PSD `vmsk` flag semantics, and the Qt bridge.
- `ui/properties-panel-vector-masks`: the Properties panel's Vector Mask section
  (name row, Enable/Disable, Link/Unlink, Delete, Rasterize, and disabled
  Density/Feather placeholders).

### Modified Capabilities

- `ui/layers-panel`: the `Layer ▸ Vector Mask` menu leaves and
  `Layer ▸ Rasterize ▸ Vector Mask` gain handlers and enablement, and the row
  delegate gains vector-mask thumbnail, link-glyph, and disabled red-X
  indicators with their hit-testing.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/vector_masks.rs` (new), its
  `mod.rs` module/re-export, and the `pictura_render` crate-root re-export.
- `crates/pictura-app/src/cxxqt_object/impl_layers/vector_masks.rs` (new), the
  `mod` line in `impl_layers.rs`, the bridge file list in
  `crates/pictura-app/build.rs`, and a `vector_mask_thumbnail_image` helper in
  `helpers_composite.rs`.
- `crates/pictura-app/cpp/commands.h`, `command_tree.cpp`, `frame.h`,
  `frame_menus.cpp`, and `frame_menus_layer_ops.cpp` — the menu ids, handlers,
  and enablement.
- `crates/pictura-app/cpp/panels/layers_panel_model.h`, `layers_panel_internal.h`,
  `layers_panel.cpp`, `layers_panel_test.cpp`, and `layers_panel.h` — the vector
  roles, delegate geometry, refresh, clicks, and test seams.
- `crates/pictura-app/cpp/panels/properties_panel.{h,cpp}` — the Vector Mask
  section.
- Qt tests: `tst_layers_panel.cpp`, `tst_command_tree.cpp`, and
  `tst_properties_panel.cpp`.
- No `Cargo.toml` or dependency changes; no `CMakeLists.txt` change (no new
  `.cpp`/`.h` files).
