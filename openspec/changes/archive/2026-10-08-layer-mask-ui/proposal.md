# Proposal

## Why

The layer-mask model and its Qt bridge landed (`layer-mask-model-and-ops`): a
layer can now carry a raster mask, and the C++-callable free functions
`layer_mask_add/delete/apply/set_enabled/set_linked/present/linked` exist. The
authoring surface, however, is still inert: the Layers strip mask button, the
row context-menu mask rows, the `Layer ▸ Layer Mask` menu leaves, and the
Properties panel expose none of it. This change wires the already-implemented
bridge to the UI.

## What Changes

- The Layers action-strip **Add Layer Mask** button becomes active: a click adds
  a `reveal-selection` mask when a selection exists, else `reveal-all`; an
  `Alt`-click adds `hide-all`, matching CS6.
- The Layers **row context menu** enables its `Add Layer Mask`, `Delete Layer
  Mask`, and enable/disable mask rows and dispatches them to the bridge.
- The **Layer ▸ Layer Mask** menu leaves (Reveal All, Hide All, Reveal
  Selection, Hide Selection, From Transparency, Delete, Apply, Enable, Disable,
  Link, Unlink) gain handlers and enablement so they light up.
- The **Properties** panel shows a Mask section for a layer that carries a mask:
  a name row plus Enable/Disable, Link/Unlink, Delete, and Apply actions, and
  read-only Density/Feather/Invert rows marked `— not implemented yet`.
- Row mask indicators in the Layers delegate (the link glyph between the layer
  and mask thumbnails, the red X over a disabled mask, and their hit-testing)
  are implemented: a `layer_mask_disabled` engine read plus per-row
  `layer_row_mask_disabled` / `layer_row_mask_linked` bridge reads populate new
  `MaskDisabledRole` / `MaskLinkedRole` model roles.

## Capabilities

### New Capabilities

- `ui/properties-panel-masks`: the Properties panel's layer-mask section — the
  mask name row and the Enable/Disable, Link/Unlink, Delete, and Apply actions
  for the active layer's mask.

### Modified Capabilities

- `ui/layers-panel`: the action strip's Add Layer Mask button becomes
  implemented; the row context menu's mask rows become implemented; the row
  delegate gains layer-mask link/disabled indicators (see above).

## Impact

- `crates/pictura-app/cpp/panels/layers_panel.cpp` — the `layersStripMask`
  button wiring.
- `crates/pictura-app/cpp/panels/layers_panel_menu.cpp` — the mask rows of
  `kRowSpecs` and `performRowAction`.
- `crates/pictura-app/cpp/panels/properties_panel.cpp` +
  `properties_panel.h` — the Mask section.
- `crates/pictura-app/cpp/command_tree.cpp`, `commands.h`,
  `frame_menus.cpp` — the `Layer ▸ Layer Mask` command ids and handlers.
- `crates/pictura-app/cpp/selftest.cpp` — the strip-button asset check no
  longer expects the mask button disabled.
- Qt tests: `tst_command_tree.cpp`, `tst_properties_panel.cpp`, and the
  `layers_panel.h`/`layers_panel_test.cpp` seams.
- `crates/pictura-render/src/document_ops/layer_ops/layer_masks.rs` — the
  `layer_mask_disabled` read and its re-exports.
- `crates/pictura-app/src/cxxqt_object/impl_layers/layer_masks.rs` (moved from
  `cxxqt_object/layer_masks.rs`) — the per-row `layer_row_mask_disabled` /
  `layer_row_mask_linked` reads.
- No `Cargo.toml` or dependency changes.
