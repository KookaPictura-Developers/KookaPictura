# Proposal

## Why

The engine models a raster layer mask (`LayerMask`, `crates/pictura-core/src/lib.rs`)
and the compositor already honours it, PSD read/write round-trips it, and the app
can build one from a selection (`selection_to_mask`,
`crates/pictura-app/src/cxxqt_object/helpers_composite.rs`). But there is no
operation to create, delete, apply, enable, or link a mask, so the Layers panel
cannot offer any mask authoring. This change adds the engine and bridge half so a
follow-up `layer-mask-ui` can wire the panel.

## What Changes

- Add layer-mask operations to `pictura_render::document_ops::layer_ops`:
  `add_layer_mask`, `delete_layer_mask`, `apply_layer_mask`,
  `set_layer_mask_enabled`, `set_layer_mask_linked`, and the read predicates
  `has_layer_mask`, `layer_mask_linked`.
- `add_layer_mask` takes a `LayerMaskKind` (`RevealAll`, `HideAll`,
  `RevealSelection`, `HideSelection`, `FromTransparency`). The selection-driven
  variants take caller-supplied coverage as an `Option<&LayerMask>`; the engine
  does not depend on app types.
- `apply_layer_mask` folds mask coverage permanently into the layer's alpha and
  clears the mask, and is refused on a smart-object layer (CS6 cannot apply a
  mask to a smart object).
- Give the mask `flags` byte a typed link accessor: the PSD "position relative
  to layer" bit (bit 0). The raw `flags` byte keeps round-tripping.
- Expose all of the above through a new `cxxqt_object/layer_masks.rs` bridge
  module over the active layer, each mutation recompositing and recording one
  history state, mirroring `clipping.rs`.
- No C++/Qt panel or `CMakeLists.txt` changes (the follow-up `layer-mask-ui`
  owns those). No new dependencies.

## Capabilities

### New Capabilities

- `compositing/layer-masks`: creating, deleting, applying, enabling/disabling,
  and linking a raster layer mask, and the PSD link-flag semantics.

### Modified Capabilities

<!-- none -->

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/layer_masks.rs` (new), its
  `mod.rs` module/re-export, and the `pictura_render` crate-root re-export.
- `crates/pictura-app/src/cxxqt_object/layer_masks.rs` (new), the `mod` line in
  `cxxqt_object.rs`, and the bridge file list in `crates/pictura-app/build.rs`.
- No `docs/`, `CMakeLists.txt`, C++ panel, or dependency changes.
