# Proposal: native-depth-remaining-adjustments

## Why

`apply_native` covers the tonal and color-preserving families but still refuses
`Auto` and `ColorLookup`, so those two adjustments force a 16/32-bit document
back to 8-bit. Closing them makes the native adjustment set complete (only the
generative fill kinds remain, which are not per-pixel adjustments).

## What Changes

- Extend `pictura_adjust::apply_native` to cover `Auto` (`Tone`/`Contrast`/
  `Color`) and `ColorLookup`, mirroring the 8-bit kernels in the unit domain:
  - `ColorLookup`: trilinear-sample the parsed 3-D LUT on native unit values and
    write the result back at the store depth (the 8-bit path already computes in
    `f32` and quantizes only at the end).
  - `Auto`: build the histogram at native resolution, derive the
    percentile bounds and stretch in native units, and apply
    `snap_neutral_midtones` in unit space.
- Keep the 8-bit `apply` byte-identical.
- After this, the only `Unsupported` adjustments at native depth are the
  generative fill kinds.

## Capabilities

### New Capabilities

<!-- None: this extends the existing native-depth-adjustments entry point. -->

### Modified Capabilities

- `native-depth-adjustments`: the covered-adjustment list grows to include
  `Auto` and `ColorLookup`; the unsupported set shrinks to the fill kinds.

## Impact

- `crates/pictura-adjust/src/native.rs`: the `Auto` and `ColorLookup` kernels.
- Tests: extend the u8-equality test to both; add a depth-16 precision test for
  `ColorLookup` with a non-identity cube.
- No new dependency, no app change.
