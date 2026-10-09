# Proposal: shape-layer-actions

## Why

Shape-layer creation and rendering already work (`add_shape_layer`,
`set_shape_fill` / `set_shape_stroke`, the shape tools and the Layers badge),
but the remaining CS6 shape-layer actions do not: there is no way to copy a
shape's Fill+Stroke attributes to another shape layer, `Layer > Rasterize >
Shape` is a disabled stub, and a shape layer (like a type layer) does not force
Transparent and Image locks in the Layers panel (`docs/02-ui-ux/panels/layers-panel.md`,
`docs/03-tools/shape-tools.md`). Issue #111 tracks these gaps.

## What Changes

- **Copy / Paste Shape Attributes.** Capture a shape layer's fill (on/off +
  colour) and stroke (on/off + colour + width + align) and apply them to
  another shape layer, reusing the existing `shape_fill` / `shape_stroke`
  getters and `set_shape_fill` / `set_shape_stroke` setters. Paste refuses a
  non-shape target. Wired as `Layer > Copy Shape Attributes` /
  `Layer > Paste Shape Attributes` and as shape-row entries in the Layers row
  context menu.
- **Rasterize Shape.** `Layer > Rasterize > Shape` and a shape-row
  context-menu entry convert a shape layer's rendered fill+stroke to ordinary
  pixels and drop its shape/vector definition (the solid-fill adjustment, the
  `vmsk` vector mask, the live-shape `vogk` block, and the stroke effect).
- **Forced locks.** A shape layer, like a type layer, shows Lock Transparency
  and Lock Image forced on in the panel lock strip and cannot have them
  deselected, while remaining engine-enforced so the bridge refuses the clear.
- **Tool-reflecting names.** Confirm a shape drawn by a tool is named for the
  tool (`"Rectangle 1"`); the current `kind.layer_name()` + `next_layer_name`
  path already does this, so this is a verification only.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `compositing/layer-management`: add Copy/Paste Shape Attributes and a
  rasterizable Shape entry to the Rasterize subset.
- `ui/layers-panel`: add the shape row-menu entries and the type/shape forced
  Transparent+Image lock rule.

## Impact

- `pictura-render`: `shape_style.rs` (attribute capture/apply, forced-lock
  predicate), `layer_ops/rasterize.rs` (`rasterize_shape`), `properties.rs`
  (forced-lock refusal), `mod.rs` exports.
- `pictura-app`: `cxxqt_object/shapes.rs` (bridge invokables), `state.rs`
  (the copied-attributes clipboard), `impl_layers.rs` (forced-lock projection +
  refusal), C++ `command_tree.cpp` / `commands.h` / `frame_menus.cpp` /
  `panels/layers_panel{,_menu,_test}.cpp` / `panels/layers_panel.h`.
- Tests: Rust unit tests in `shape_style.rs` / `rasterize.rs`; Qt Test in
  `tst_shape_tools.cpp`, `tst_layers_panel.cpp`, `tst_command_tree.cpp`.
- No new dependency, no `docs/` change.
