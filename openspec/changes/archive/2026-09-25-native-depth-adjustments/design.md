# Design: native-depth-adjustments

## Context

`pictura_adjust::apply` dispatches every `Adjustment` to an 8-bit kernel:
tonal.rs uses 256-entry `[u8; 256]` LUTs (`map_lut`) or per-pixel `u8` math.
The 8-bit results are a parity surface (ImageMagick oracle, known-value tests,
`tests/README.md`), so they must not move. Phase 1 added `Samples`
(`u8`/`u16`/`f32`) as the native store; this change applies the tonal family to
it.

## Goals / Non-Goals

**Goals:**

- Apply `Invert`, `Desaturate`, `Levels`, `Curves`, `BrightnessContrast`,
  `Exposure`, `Posterize`, `Threshold`, `GradientMap` to a `Samples` store at
  native precision.
- Byte-identical 8-bit output from the unchanged `apply`.
- A codec-level proof that native editing preserves >8-bit precision.

**Non-Goals:**

- No color-preserving adjustments (Hue/Saturation, Vibrance, B&W, Channel
  Mixer, Photo Filter, Color Balance, Selective Color), `Auto`, `ColorLookup`,
  or the fill kinds — they return `Unsupported` on a native store.
- No app/UI wiring, no compositor change, no 32-bit HDR tone map, no GPU.
- No change to the 8-bit kernels' math or rounding.

## Decisions

**A `Sample` trait in `pictura-core::samples`, in the `[0, 1]` domain.**
`Sample { fn to_unit(self) -> f64; fn from_unit(f64) -> Self }` with impls for
`u8` (`v/255`), `u16` (`v/65535`), `f32` (clamped to `[0,1]`). Kernels are
written once against `f64` unit-domain values and a generic
`map_color_planes<T: Sample>`; `Samples` is decoded into a
`PixelBuffer<f64>`-equivalent working form, adjusted, and re-encoded.

**Keep the u8 path untouched; add the native path beside it.** The existing
`apply` keeps its integer LUT kernels so the ImageMagick oracle and known-value
tests cannot move. `apply_native(adjustment, samples, w, h, channels)` is a new
entry that only accepts native storage and is implemented with unit-domain
`f64` math that mirrors each kernel's documented formula. DRY here is
outranked by not perturbing the parity surface; the formulas are cross-checked
by a test that the native `u8` result equals the 8-bit `apply` result for the
same input.

**Precision is the acceptance signal.** A depth-16 edit that quantizes to 8-bit
then widens yields `low == high` for every sample (`v = high*257`). The e2e test
asserts at least one sample has `low != high` and that the value is the
native-domain rounding, not the widen.

## Risks / Trade-offs

- [Two implementations of the same formula can drift] → a test pins native-u8
  `==` `apply` for each covered adjustment over a fixed image.
- [f32 clamping/NaN in `to_unit`] → `to_unit` clamps and maps NaN to 0; tested.
- [The color family still quantizes] → explicit `Unsupported` return, documented
  ceiling; Phase 3.

## Migration Plan

Additive: `apply` is unchanged, `apply_native` is new. Rollback is a revert.
Phase 3 wires `apply_native` into the app and covers the color family.
