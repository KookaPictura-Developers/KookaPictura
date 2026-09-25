# Design: native-depth-color-adjustments

## Context

`pictura-adjust/src/native.rs` already dispatches the tonal family to
unit-domain `f64` kernels. `color.rs` kernels already operate per pixel in
`f64`, mostly on `[0, 1]` values (`rgb_to_hsl(r/255, …)`), so the native port is
a change of read/write, not of formula.

## Goals / Non-Goals

**Goals**

- Add `HueSaturation`, `Vibrance`, `ColorBalance`, `BlackWhite`, `PhotoFilter`,
  `ChannelMixer`, `SelectiveColor` to `apply_native`, byte-identical to `apply`
  on a `u8` store and full-precision on `u16`/`f32`.
- Keep the 8-bit path and kernels untouched.

**Non-Goals**

- `Auto` (histogram-based), `ColorLookup` (3-D LUT), and the fill kinds stay
  `Unsupported` at native depth.
- No app wiring, no HDR tone map, no GPU.

## Decisions

**Mirror each color kernel in `native.rs`.** As in Phase 2, DRY is outranked by
not perturbing the 8-bit parity surface; the existing u8-equality test pattern
extends to the new kernels. Kernels that read `v as f64 / 255.0` become
`to_unit()`; kernels that work in the 0–255 domain (`color_balance`) scale by
`255.0` on read and `/255.0` on write. `skin_bump` and the HSL helpers are
reused as-is.

**One shared test.** `native_u8_equals_apply_for_every_covered_adjustment`
already loops the covered set; add the color variants (including
`preserve_luminosity` on/off, `selective_color` methods, `channel_mixer`).

## Risks / Trade-offs

- [Color kernels clamp or round at 0–255; a unit-domain mirror could round
  differently at exact `.5` boundaries] → the byte-identity test is the gate;
  if it diverges, mirror the exact arithmetic (as Desaturate did in Phase 2).
- [Outputs can exceed `[0,1]` before clamping] → `from_unit` clamps for
  `u8`/`u16`; `f32` keeps the value, consistent with Phase 2's HDR-leaning
  decision.

## Migration Plan

Additive; rollback is a revert. Phase 4 wires `apply_native` into the app and
adds app-level tests.
