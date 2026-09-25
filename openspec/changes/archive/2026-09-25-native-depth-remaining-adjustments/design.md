# Design: native-depth-remaining-adjustments

## Context

`native.rs` already mirrors the per-sample tonal/color kernels in unit `f64`.
`ColorLookup` (`lut.rs`) computes in `f32` unit space and only quantizes at the
end, so a native path is the same sampling without `to_u8`/`from_unit`. `Auto`
(`auto.rs`) is histogram-based: a 256-bin histogram, percentile bounds, a stretch
LUT, and a neutral-midtone gamma.

## Goals / Non-Goals

**Goals**

- `Auto` and `ColorLookup` in `apply_native`, byte-identical to `apply` on `u8`,
  full-precision on `u16`/`f32`.
- 8-bit kernels untouched.

**Non-Goals**

- The generative fill kinds (not per-pixel adjustments).
- App wiring, GPU, HDR.

## Decisions

**`ColorLookup`: reuse `lut::sample`.** For each pixel, feed the unit color to
the parsed LUT's trilinear `sample` and write the returned `[f32; 3]` back
through the store. The `u8` path's `to_u8` is `from_unit` at 8 bits, so the
results coincide; an unparseable/abstract/device-link LUT stays a no-op exactly
as today.

**`Auto`: histogram at native resolution.** Build one histogram per channel (or
a joint one for `Contrast`) over the store's levels — `65536` bins for `u16`, a
`65536`-bin `[0,1]` histogram for `f32`, `256` for `u8` — find the
`clip`-percentile bounds, and stretch each sample at native resolution. For `u8`
the bin math reduces to the existing `percentile_bounds`/`stretch_lut`, so the
byte result is identical. `snap_neutral_midtones` runs in unit space with the
same `64..=192` (of 255) midtone window and gamma clamp.

**Byte-identity is the gate.** The u8-equality test extends to `Auto`
(`Tone`/`Contrast`/`Color`) and `ColorLookup`; if a native mirror diverges, match
the exact 8-bit arithmetic as the earlier phases did.

## Risks / Trade-offs

- [`Auto` f32 histogram bin edges] → bins are `[i/65536, (i+1)/65536)`; the
  percentile/stretch use bin centers consistently, and the u8 test pins parity.
- [`snap_neutral_midtones` gamma in unit space] → clamp `0.1..=9.99` as today.
- [Memory for a 65536-bin histogram] → 512 KB per channel at `u64`, transient;
  acceptable and smaller than the sample store.

## Migration Plan

Additive; revert is a revert.
