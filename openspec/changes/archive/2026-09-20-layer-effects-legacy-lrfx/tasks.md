## 1. Legacy decoder

- [x] 1.1 Add `crates/pictura-render/src/layer_effects/legacy.rs` with `LegacyEffects { drop_shadow, inner_shadow, outer_glow, inner_glow, bevel, color_overlay }` (each `Option`, default `None`) and `pub(crate) fn decode_legacy_effects(layer: &Layer) -> Option<LegacyEffects>`.
- [x] 1.2 Parse the `lrFX` block from `layer.extra_block(b"lrFX")` as the fixed `EffectsLayer` struct over a bounds-checked big-endian cursor: `u16` version (require 0), `u16` count, then each record's `8BIM` + 4-byte `ostype` + `u32` body length. Decode `cmnS` first (last one wins) and AND its `visible` into every record's `enabled` (absent `cmnS` = visible true).
- [x] 1.3 Decode the shared `Color` struct as `u16` space (require 0 = RGB) + 4×`u16` channels; map each channel to `u8` with `value >> 8`.
- [x] 1.4 Decode `dsdw`/`isdw` (`ShadowInfo`: version 0/2, blur, intensity, angle, distance, Color, `8BIM`, blend 4s, enabled, use_global_angle, opacity, and version-2 native Color) into `DropShadow` / `InnerShadow`: `blend_mode` via `BlendMode::from_psd_key` (layer vocabulary), `opacity = byte as f32 * 100.0 / 255.0` clamped `0..=100`, `size` ← blur clamped `0..=250`, `spread`/`choke` ← intensity clamped `0..=100`, `distance` clamped `0..=30000`, `angle_deg` ← angle, `use_global_angle`; `present` true, `knocks_out` false.
- [x] 1.5 Decode `oglw`/`iglw` (version 0/2, blur, intensity, Color, `8BIM`, blend, enabled, opacity; v2 outer native Color, v2 inner invert byte + native Color) into `OuterGlow` / `InnerGlow`: `spread`/`choke` ← intensity, `size` ← blur, `technique` `Softer`, inner `source` `Center` when invert is non-zero else `Edge`; defaults on unknown blend key (Screen) and an absent color.
- [x] 1.6 Decode `bevl` (version 0/2, angle, depth, blur, two `8BIM`+mode pairs, highlight/shadow Color, bevel_style, highlight_opacity, shadow_opacity, enabled, use_global_angle, direction; v2 real highlight/shadow Color) into `BevelEmboss`: `style` from the byte (0 Outer, 1 Inner, 2 Emboss, 3 Pillow, 4 Stroke), `technique` `Smooth`, `direction` from the byte, `size` ← blur, `soften` 0, `depth` clamped `0..=1000`, `angle_deg` ← angle, `altitude_deg` 30, half opacities scaled `0..=255 → 0..=100`, half colours.
- [x] 1.7 Decode `sofi` (version must be 2, `8BIM`, blend, Color, opacity, enabled, native Color) into `ColorOverlay`; skip a `sofi` whose version is not 2.
- [x] 1.8 Error contract: a missing `lrFX`, an unsupported version, a truncated body, a non-`8BIM` record signature, or a count overrun returns `None`; an unknown `ostype` or an individually malformed record is skipped; every read is bounds-checked and the parser never panics.
- [x] 1.9 Add the `//!` module doc paragraph and a `ponytail:` ceiling comment (contour, noise, anti-alias, global-light resource, bevel only `Inner + Smooth`, libpsd's exact spread/blur split).
- [x] 1.10 Re-export `decode_legacy_effects` / `LegacyEffects` from `layer_effects/mod.rs` and `lib.rs` as appropriate.

## 2. Shared resolver and precedence

- [x] 2.1 Add `pub(crate) struct LayerEffects` in `mod.rs` holding all ten typed `Option`s, with `Default`, a `from_lfx2(layer)` constructor that calls the shipped `decode_*`, and a `from_legacy(LegacyEffects)` mapping.
- [x] 2.2 Add `pub(crate) fn decode_layer_effects(layer: &Layer) -> LayerEffects`: when `layer.extra_block(b"lfx2").is_some()` return `from_lfx2` and ignore `lrFX`; otherwise return `from_legacy(decode_legacy_effects(layer)?)` or all-`None`.
- [x] 2.3 Refactor `composite_layer_effects` and `composite_layer_effects_above` to call `decode_layer_effects(layer)` once and consume the fields, keeping the group / destructive-adjustment skip, the below/above split and the existing inter-effect order unchanged.
- [x] 2.4 Keep every existing `decode_*` function `lfx2`-only and unchanged, so their public contract and tests are untouched.

## 3. Render reuse

- [x] 3.1 Confirm no `composite_*` renderer changes: the resolved legacy params are the same typed structs, so the drop shadow / outer glow composite below and the inner shadow / inner glow / bevel / color overlay composite above through the shipped code.
- [x] 3.2 Confirm an unmapped or declined legacy effect is a no-op: a bevel whose style is not `Inner`, a `sofi` with version ≠ 2, or a skipped record leaves the composite byte-identical to the same document without the legacy block.
- [x] 3.3 Confirm `present` is true and `knocks_out` false for legacy shadows, so the render path is unchanged and `knocks_out` cannot affect pixels.

## 4. GPU fallback

- [x] 4.1 Change `check_supported`'s `walk` in `crates/pictura-render/src/gpu/mod.rs` to resolve `decode_layer_effects(layer)` once and return `GpuError::UnsupportedLayerEffect` for an enabled and present drop shadow, outer glow, inner shadow, inner glow, satin, solid-colour stroke or overlay, and a bevel only when `Inner + Smooth`. Keep the effect check ahead of the adjustment check and add no new error variant.
- [x] 4.2 Confirm the existing GPU tests still pass (the resolution is behaviour-preserving for `lfx2`).

## 5. Fixture and oracle

- [x] 5.1 Add `legacy_effects()` to `scripts/generate-fixtures.py`: a `Base` pixel layer plus a signed layer whose record carries `TaggedBlock(Tag.EFFECTS_LAYER, EffectsLayer({...}))` built from `CommonStateInfo`, a `dsdw` `ShadowInfo` (blur 5, intensity 0, angle 120, distance 5, a 16-bit `Color`, blend `mul `, enabled 1, use-global-angle 0, opacity 255) and an `oglw` `OuterGlowInfo` (blur 6, intensity 0, a 16-bit `Color`, blend `scrn`, enabled 1, opacity 191). Register `"legacy_effects.psd"` in `FIXTURES`.
- [x] 5.2 Regenerate with `python3 scripts/generate-fixtures.py` and confirm every existing fixture is byte-identical (`git status` shows only the new file).
- [x] 5.3 Register the fixture in the codec `FIXTURES` table and add `crates/pictura-codec/tests/oracle/legacy.rs` (declared with `#[path]` from `oracle.rs`): the `lrFX` key is present in `extra_blocks`, the whole `Document` round-trips `write_psd`/`read_psd` with `lrFX` preserved, and a self-skipping psd-tools check re-reads it as an `EffectsLayer` asserting the shadow and glow values.
- [x] 5.4 Update `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and the `legacy_effects()` snippet, noting the fixed struct layout, the layer blend vocabulary and the 16-bit `Color`.

## 6. Tests

- [x] 6.1 Add `crates/pictura-render/src/tests/layer_effects/legacy.rs` declared from `tests/layer_effects.rs`, under the 1400 LOC test cap.
- [x] 6.2 Decode tests: a full hand-built `lrFX` (all six mapped records) decodes to the expected typed params; each record decodes alone; missing-kind fields take defaults; the `cmnS` visible flag gates `enabled` and its absence defaults true; a non-zero inner-glow invert maps to `Center`; unknown `ostype`, a version-≠-2 `sofi`, a bad `Color` space and a bad version are handled (skip vs `None`); a truncated body, a non-`8BIM` signature and a count overrun return `None`; out-of-range numerics clamp; nothing panics.
- [x] 6.3 Precedence tests: with both blocks present the resolved drop shadow is the `lfx2` one and the composite equals the document with `lrFX` removed; with only `lrFX` the legacy shadow renders; an `lfx2` block that omits `DrSh` suppresses the legacy `dsdw`; the legacy-only resolver leaves satin/stroke/overlays absent.
- [x] 6.4 Render tests: a legacy drop shadow composites byte-identically to an `lfx2` equivalent whose render-affecting typed fields are equal; a legacy outer glow tints near the content and leaves a beyond-reach corner byte-identical to the no-effect composite; a legacy `sofi` changes interior pixels and leaves exterior pixels byte-identical; a legacy bevel with `Inner` renders and a non-`Inner` bevel is a no-op; a disabled record and a non-visible `cmnS` are byte-identical to no effect. (Legacy inner-shadow and inner-glow fields are covered by the decode tests in 6.2; no separate render test is added.)
- [x] 6.5 Fixture-driven test: load `legacy_effects.psd` with `include_bytes!`, assert `decode_legacy_effects` yields the authored params, and assert the composite differs from the no-effect composite.
- [x] 6.6 GPU tests: `composite_gpu` on a legacy-only effect document returns `UnsupportedLayerEffect` without panicking; `composite_gpu_or_cpu` equals `composite_rgba`; a disabled / malformed / unmapped legacy block does not produce the error.

## 7. App

- [x] 7.1 Confirm no production app change is needed: the canvas composites through `pictura_render::composite_rgba` / `composite_active`, which now render the resolved legacy effects. Do not add an authoring command, panel, or `CMakeLists.txt` entry. No C++ self-test check.

## 8. Gates

- [x] 8.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [x] 8.2 `bash scripts/verify-fast.sh` and a headless self-test; record counts.
- [x] 8.3 `openspec validate layer-effects-legacy-lrfx --strict` and `openspec validate --all --strict`.
- [x] 8.4 Commit with the new golden fixture; state the fixture addition in the commit message. Update `docs/dev/STATE.md` separately under `TASK-ALLOWS-DOCS` if the milestone anchor is advanced.
