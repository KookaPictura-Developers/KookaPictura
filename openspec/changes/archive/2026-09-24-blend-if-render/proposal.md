# Proposal: blend-if-render

## Why

Roadmap G6: the typed `BlendIf` view of a layer's blending-ranges body ships
(`knko-blend-if-model`) but the compositor ignores it, so a layer whose Blend If
sliders hide it over part of the tonal range composites as if the sliders were
at the full `0..65535` default. The GPU compositor has no shader for it either.

## What Changes

- **Apply Blend If in the CPU compositor.** For a layer whose typed `BlendIf`
  view is present and not at the full default, gate the layer's per-pixel blend
  weight: the composite source range gates on the source pixel's gray, the
  composite destination range gates on the running backdrop's gray, and each
  per-channel group gates on that channel's source/backdrop value. A value
  inside `[black, white]` blends; outside, the layer is skipped for that pixel.
- **GPU declines a non-default Blend If layer** with a new
  `GpuError::UnsupportedAdvancedBlending`, so `composite_active` falls back to
  the CPU oracle exactly as it does for `Dissolve`. Documents with no Blend If
  (or only the default ranges) are byte-identical, so existing GPU parity and
  goldens are unaffected.
- **Add `BlendIf::is_default()`** in `pictura-core` so a full-range view is a
  cheap no-op.
- **No UI.** The Layer Style > Blending Options dialog is not part of this
  change.
- **Deferred: knockout punch-through.** `Knockout::Shallow`/`Deep` need
  group-aware compositor restructuring (a knockout layer erases the layers
  beneath it within its group); this change leaves knockout a documented no-op
  and does not claim it.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `psd-advanced-blending`: add a requirement that the CPU compositor applies
  the Blend If ranges and the GPU declines a non-default layer.

## Impact

- `crates/pictura-core`: `BlendIf::is_default`.
- `crates/pictura-render`: a `blend_if_factor` gate in `composite.rs::blend_into`
  (and `GpuError::UnsupportedAdvancedBlending` + its `Display` arm in
  `gpu/mod.rs`); CPU unit tests.
- No new dependency; no app UI; no PSD byte-layout change.
- Ceiling: the exact Photoshop feather/split-slider curve and the composite-gray
  weighting are unverified (no Photoshop oracle) and marked `ponytail:`; the
  stored model has only `(black, white)` per range, so the gate is hard.
