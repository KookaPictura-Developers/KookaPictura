# Proposal: native-depth-masks

## Why

A layer's raster mask gates compositing through `mask_alpha` (8-bit), even though
a 16/32-bit read retains the native `-2` mask plane in `Layer.source_channels`.
A high-depth layer's mask therefore quantizes before it gates the blend. This is
the last 8-bit chokepoint in the CPU compositor.

## What Changes

- For a 16/32-bit document whose layer has a retained native `-2` mask plane
  matching the mask rect, gate the blend by the native mask sample converted to
  unit `f32`, combined with the vector-mask coverage in unit space.
- Keep the 8-bit `mask_alpha`/`blend_into` result byte-identical for 8-bit
  documents and for masks without a native store.
- Thread the document into `blend_into` so the mask lookup can see
  `source_depth` and the store; update the call sites in `composite.rs`,
  `composite_native.rs`, `fill.rs`, and `text_render.rs`.

## Capabilities

### New Capabilities

- `native-depth-masks`: native-depth raster-mask gating in the CPU compositor.

### Modified Capabilities

<!-- None. -->

## Impact

- `crates/pictura-render/src/composite.rs` / `composite_native.rs`: `mask_alpha`
  unit variant and the `blend_into` signature.
- `crates/pictura-render/src/fill.rs`, `text_render.rs`: pass `doc` to
  `blend_into`; `composite_solid_fill`/`composite_gradient_fill` gain a `doc`
  parameter.
- Tests: a render test that a high-depth layer's native mask changes the
  composite versus the 8-bit mask, and the existing suite stays green.
- No new dependency; no app change.
