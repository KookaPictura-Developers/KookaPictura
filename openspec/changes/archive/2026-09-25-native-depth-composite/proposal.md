# Proposal: native-depth-composite

## Why

The CPU compositor already accumulates in normalized `f32` (`Canvas { px: Vec<Px> }`),
but when it reaches an adjustment layer it converts the running canvas to an
8-bit `PixelBuffer`, applies the adjustment, and reads it back
(`composite.rs:1071-1096`). A 16/32-bit document therefore loses its extra
precision at every adjustment layer, and there is no native-depth composite
output for the save path to retain. Phase 4 closes that within the CPU
compositor.

## What Changes

- For a 16/32-bit document, `composite_adjustment` SHALL apply the adjustment to
  the `f32` canvas through `pictura_adjust::apply_native` (an `f32` sample
  store) instead of round-tripping through 8-bit. An adjustment outside
  `apply_native`'s covered set SHALL fall back to the existing 8-bit path.
- Add `pictura_render::composite_native(doc) -> Option<Samples>`: the document's
  composite at its source depth (`U16` at 16, `F32` at 32), or `None` for an
  8-bit/constructed document.
- The 8-bit `composite_rgba` output and all existing compositing SHALL be
  byte-identical.
- Layer *content* remains read from the 8-bit model for now (native read from
  `source_channels` is a later phase); the gain is adjustment and blend math,
  which is where multi-layer precision is lost.

## Capabilities

### New Capabilities

- `native-depth-composite`: native-precision adjustment compositing and a
  source-depth composite output.

### Modified Capabilities

<!-- None: layer-compositing's 8-bit behavior is unchanged. -->

## Impact

- `crates/pictura-render/src/composite.rs`: the `composite_adjustment` high-depth
  branch and `composite_native`.
- `crates/pictura-render/src/lib.rs`: export `composite_native`.
- Tests: a render test that a depth-16 document with an adjustment layer yields
  native samples that are not the 8-bit widening, and that the 8-bit suite is
  unchanged.
- No new dependency; no app change (the app can call `composite_native` later).
