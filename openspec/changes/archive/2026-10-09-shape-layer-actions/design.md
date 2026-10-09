# Design: shape-layer-actions

## Context

Shape layers already exist as a solid `SoCo` fill cut to a `vmsk` vector mask,
with the shape tools' Stroke stored as a Stroke layer effect (`lfx2` `FrFX`) the
compositor draws along the mask (`shape_layer.rs`, `shape_style.rs`). The
remaining actions reuse that model rather than introducing a new one.

## D1. Attribute capture is an engine value, not a PSD block

`shape_style.rs` gains `ShapeAttributes { fill: Option<[u8; 3]>, stroke:
Option<ShapeStroke> }`, `copy_shape_attributes(layer) -> Option<ShapeAttributes>`
(`None` when not a shape layer) and `paste_shape_attributes(layer, &attrs) ->
bool`. Paste returns `false` for a non-shape target and otherwise applies
`set_shape_fill` then `set_shape_stroke` with a non-short-circuiting `|`, so a
changed fill is not lost when the stroke is unchanged and vice versa. Copy adds
no history; paste records one state only when it changed something. The
clipboard lives on `PictureViewRust` (`shape_attributes: Option<ShapeAttributes>`),
so it survives between the Layer menu and the row menu and is never serialized.

## D2. Rasterize Shape bakes the rendered appearance

`rasterize_shape(doc, path)`:

1. Resolve the layer and require `is_shape_layer`.
2. Render just that layer through the real compositor: clone it into a scratch
   `Document` with `opacity = 255`, `mask = None`, `blend = Normal` and the same
   shape data, so the baked pixels are the intrinsic fill-cut-to-outline plus
   stroke, while opacity, blend, and the layer mask keep applying once to the
   raster result. `fill` is left as-is because a shape's fill is binary (`0` or
   `255`), so it bakes correctly.
3. Copy the scratch composite's four planes within the layer rect into the
   layer's `0/1/2/-1` channels (bounds-safe; out-of-canvas reads as 0).
4. Drop the shape definition: `adjustment = None`, remove the `vmsk` and `vogk`
   blocks, and remove the `lfx2` stroke/effects block (it is already baked, so
   keeping it would double-apply). Keep name, rect, opacity, fill, blend, mask.

The result recomposites to the same image the shape showed. `rasterize_all_layers`
also walks shape layers. This is a full-document scratch composite per shape;
rasterize is not a hot path (`ponytail:`).

## D3. Forced locks: one engine predicate, projected and enforced

`shape_layer.rs` exports `has_forced_locks(layer) -> bool` =
`is_shape_layer(layer) || layer.is_type()`. Two call sites use it:

- `impl_layers::layer_row_lock` ORs `TRANSPARENCY | PIXELS` into the reported
  bits for such a layer, so the panel strip renders them checked.
- `properties::set_lock_paths` (and the single-layer `set_layer_lock` bridge)
  mask those two bits out of an `on = false` request; clearing only Position
  still succeeds, and a request that changes nothing returns 0/false so no
  history is recorded.

On the C++ side `syncControls` disables the Transparency and Image toggles when
the current row is a shape (`LayerRowShapeRole`) or a type layer (`KindRole ==
"type"`). The engine rule is the source of truth; the panel's disabled buttons
are the presentation.

## D4. Menus

- `command_ids`: `LayerRasterizeShape`, `LayerCopyShapeAttributes`,
  `LayerPasteShapeAttributes`. `command_tree.cpp` adds the two attribute leaves
  under `Layer` and switches the `Rasterize > Shape` leaf to implemented.
- `frame_menus.cpp` resolves the current shape path and wires handlers and
  enabled providers for all three (attributes need a current shape; paste also
  needs the clipboard non-empty).
- `layers_panel_menu.cpp` gains a `Shape` kind mask and three rows. Shape rows
  are detected from `LayerRowShapeRole` rather than the adjustment kind, so a
  shape row gets the shape rows and not `Edit Adjustment…`.

## D5. Naming

`shape_add_layer` already names a tool-drawn layer from `ShapeKind::layer_name`
via `next_layer_name` (`"Rectangle 1"`); the existing `tst_shape_tools` already
asserts it. No code change; verification only.

## Risks

- Baking unrelated layer styles into the pixels is a deliberate simplification:
  a rasterized shape freezes its appearance. Documented in the delta.
- The forced-lock projection changes `LayerRowLock` for type layers that a PSD
  did not lock; this matches CS6 and the docs contract.
