# Proposal: native-depth-layer-content

## Why

`native-depth-composite` made adjustment layers composite at native precision,
but a pixel layer's color is still read from its 8-bit `channels`. For a 16/32-bit
RGB/Grayscale document the codec retains the true source samples in
`Layer.source_channels`, so the compositor can read those instead and avoid
throwing away the low bits before the first blend.

## What Changes

- When the document is a 16/32-bit Grayscale/RGB read (`source_depth` is
  `Some`, `source_mode` is `None`) and a pixel layer's `source_channels` store
  matches the layer rect and depth, `composite_pixels` SHALL read the layer's
  color and alpha from the typed native samples (converted to unit `f32`)
  instead of `channel(layer, id)`.
- A layer without a matching store, and any converted-mode document (Lab/CMYK,
  whose retained store holds source-mode planes, not working RGB), SHALL keep
  the existing 8-bit content path.
- The 8-bit composite SHALL be byte-identical.

## Capabilities

### New Capabilities

- `native-depth-layer-content`: native-depth pixel-layer content in the CPU
  compositor.

### Modified Capabilities

<!-- None. -->

## Impact

- `crates/pictura-render/src/composite.rs` / `composite_native.rs`: native
  channel lookup and the `composite_pixels` branch.
- `crates/pictura-render/src/lib.rs` if a helper is exported (not required).
- Tests: a render test that a depth-16 RGB pixel layer whose native samples are
  not the 8-bit widening yields native composite samples that differ from the
  widening of `composite_rgba`.
- No new dependency; no app change.
