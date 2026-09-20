## Why

The `lfx2` object-based effects decode and render fully, but Photoshop also
writes the **legacy `lrFX`** effects-layer block (Photoshop 5.0–6.0 files, and
CS for backward compatibility). Kooka Pictura preserves `lrFX` opaquely and
renders nothing from it, so an older PSD — or a CS6 file whose only effect data
is `lrFX` — loses its drop shadow, glows, bevel and color overlay on open.

## What Changes

- `pictura-render`'s layer-effects module gains `layer_effects/legacy.rs` with
  the legacy binary parser: `decode_legacy_effects(&Layer) -> Option<LegacyEffects>`
  reads the `lrFX` extra block and maps each present record (`dsdw`, `isdw`,
  `oglw`, `iglw`, `bevl`, `sofi`) into the **existing typed params** (`DropShadow`,
  `InnerShadow`, `OuterGlow`, `InnerGlow`, `BevelEmboss`, `ColorOverlay`),
  filling the fields `lrFX` carries and taking the effect defaults for the
  `lfx2`-only fields it does not.
- A single shared resolver, `decode_layer_effects(&Layer) -> LayerEffects`, is
  the decode path the compositor and the GPU check both use. When an `lfx2`
  block is present it is authoritative and the `lrFX` block is ignored entirely;
  otherwise the legacy block supplies the mapped set. The two are never combined,
  so no effect is applied twice.
- Render: the shipped `composite_*` functions are reused unchanged — the typed
  params are the same structs, so a legacy effect renders through the exact
  code path an equivalent `lfx2` effect uses. Legacy records that map to no
  renderer are a documented no-op.
- GPU: `check_supported` reads the same resolved set, so an enabled and present
  legacy effect that maps to a renderable one is rejected with the existing
  `GpuError::UnsupportedLayerEffect` before dispatch. No GPU shader is added.
- Fixture: `scripts/generate-fixtures.py` gains a `legacy_effects()` builder
  authored with psd-tools' `EffectsLayer`/`ShadowInfo`/`OuterGlowInfo`; the
  codec oracle gains an `lrFX`-preserved + whole-`Document` round-trip with a
  self-skipping psd-tools read, and render tests prove a legacy drop shadow and
  outer glow composite identically to the equivalent `lfx2` params.
- No new dependency. No app production change (the canvas already composites
  through `composite_rgba` / `composite_active`). No `CMakeLists.txt` change.

## Capabilities

### New Capabilities

<!-- none: the legacy block extends the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode a layer's legacy `lrFX` block
  into the existing typed effect params, to resolve `lfx2`-over-`lrFX`
  precedence without double-applying, and to render the mapped set through the
  shipped renderers. No existing requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer whose **resolved** effects
  (object-based `lfx2`, or legacy `lrFX` when `lfx2` is absent) include an
  enabled and present renderable effect is rejected before dispatch.

## Impact

- `crates/pictura-render/src/layer_effects/legacy.rs` (new): the `lrFX` parser,
  `LegacyEffects`, and the field mapping.
- `crates/pictura-render/src/layer_effects/mod.rs`: the shared `LayerEffects`
  set, `decode_layer_effects`, the `lfx2`-precedence gate, and the compositor
  passes consuming the resolved set. `LegacyEffects` / `decode_legacy_effects`
  stay `pub(crate)`, so there is no `lib.rs` re-export.
- `crates/pictura-render/src/gpu/mod.rs`: `check_supported` consumes
  `decode_layer_effects` instead of the per-effect `lfx2` decoders.
- `crates/pictura-render/src/tests/layer_effects/legacy.rs` (new), declared from
  `tests/layer_effects.rs`: decode, precedence and render tests, under the test
  LOC cap.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/legacy_effects.psd`,
  `crates/pictura-codec/tests/oracle.rs` + `tests/oracle/legacy.rs`: the builder,
  the golden fixture, and its round-trip plus self-skipping psd-tools read.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change. No app production change.

## Out of scope (deferred)

- The legacy block carries only `cmnS`, `dsdw`, `isdw`, `oglw`, `iglw`, `bevl`,
  `sofi`; the extended effects (satin, stroke, gradient overlay, pattern
  overlay) have no `lrFX` form and stay `lfx2`-only.
- Contour, noise, anti-alias, `intensity` beyond the spread slot, and the
  document global-light resource (1037) are not modelled; the effective angle is
  the record's stored angle.
- Bevel renders only as `Inner` + `Smooth`; a legacy bevel in another style is
  a documented CPU no-op.
- `Scale Effects`, styles on groups, the isolated interior-effect composite, a
  GPU shader, and any authoring UI.
