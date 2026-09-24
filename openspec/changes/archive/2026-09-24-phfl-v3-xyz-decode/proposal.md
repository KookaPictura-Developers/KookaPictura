# Proposal: phfl-v3-xyz-decode

## Why

Roadmap P3 / G8: version-2 `phfl` (Photo Filter) decodes and renders, but version-3 payloads (three `u32` CIE XYZ values) still return `None`, so those adjustment layers composite as a no-op. This is the last deferred adjustment key named in `adjustment-layer-rendering`.

## What Changes

- `decode_photo_filter` in `crates/pictura-render/src/composite.rs` accepts version 3: read `u16` version, three big-endian `u32` XYZ values, `u32` density, `u8` luminosity; map XYZ → sRGB filter colour; return `Adjustment::PhotoFilter`.
- Version 2 continues to decode exactly as today (components `0..=255`, density `0..=100`).
- Truncated payloads, versions other than 2/3, density `> 100`, or non-finite XYZ remain `None` (no panic).
- `encode_photo_filter` stays version-2 (app authoring path unchanged); an unmodified open→save of a v3 layer re-emits v2 with the decoded colour (same pattern as other adjustments).
- Update unit tests: `deferred_keys_still_none` drops the v3 `phfl` case; add a v3 decode + round-trip check.
- **BREAKING**: none for on-disk preservation (P2 opaque path unchanged); only `decode_adjustment` behaviour for v3 changes from no-op to render.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `adjustment-layer-rendering`: version-3 `phfl` is no longer deferred — the
  committed-decode requirement covers both versions (MODIFIED); the encode
  round-trip requirement notes that encoding stays version 2 (MODIFIED); the
  deferred-payload requirement is removed because v3 was its only subject
  (REMOVED).

## Impact

- `crates/pictura-render/src/composite.rs`: `decode_photo_filter` branch for version 3 + local XYZ→sRGB helper (reuse the D50 matrix already used by `pictura-codec`'s `lab_to_rgb`, or a thin copy if cross-crate reuse is not worth a public export).
- `crates/pictura-render/src/tests/adjustment/part_decode.rs`: drop v3 from `deferred_keys_still_none`; add `phfl_decodes_version_three` (and any existing “v3 → None” assert in `phfl_decodes_version_two`).
- `openspec/specs/adjustment-layer-rendering/spec.md` via delta: MODIFIED committed-decode requirement (v3 layout + scenarios), MODIFIED encode round-trip (stays v2), REMOVED deferred requirement (v3 was its only subject).
- No `pictura-adjust` change (`PhotoFilterParams` already has `color: [u8; 3]`).
- No new dependency; no GPU shader (still CPU fallback, as today).
- Fixture: optional psd-tools-authored v3 `phfl` block only if the generator can emit it without churning unrelated goldens; otherwise unit-level payload fixtures suffice (layout already grounded on psd-tools).
