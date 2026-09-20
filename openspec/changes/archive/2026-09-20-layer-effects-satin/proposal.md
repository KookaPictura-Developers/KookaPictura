## Why

The `lfx2` object-based layer-effects framework now decodes and renders Drop
Shadow (`DrSh`), Outer Glow (`OrGl`), Inner Shadow (`IrSh`), Inner Glow
(`IrGl`), Stroke (`FrFX`) and the Color/Gradient/Pattern Overlays. A layer with a
**Satin** still renders without it: the `ChFX` object is preserved opaquely and
ignored. Satin is the last common CS6 interior effect (the soft directional
sheen used on glass, metal and fabric), so shipping it closes the common style
set.

## What Changes

- `pictura-render`'s layer-effects module gains a new `layer_effects/satin.rs`
  beside `shadows.rs`, `glows.rs`, `strokes.rs` and `overlays.rs`, with a typed
  `Satin { enabled, present, blend_mode, color, opacity, angle_deg, distance,
  size, invert }` and `decode_satin(&Layer) -> Option<Satin>`, following the same
  finite/clamp/no-panic contract as the shipped kinds.
- **The effect object class id and its top-level `lfx2` key are both `ChFX`**
  (libpsd `effects.c:308` `case 'ChFX'` → `psd_get_layer_satin2`; psd-tools
  `api/effects.py:537` `@register(Klass.ChromeFX.value)` where
  `terminology.py:206` `Klass.ChromeFX = b"ChFX"`). The exact object keys are
  `enab`, `present`, `showInDialog`, `Md  ` (`BlnM`), `Clr ` (`RGBC`), `Opct`,
  `lagl` (local angle), `uglg`, `Dstn` (distance), `blur` (size), `Invr`
  (invert), `AntA` and `MpgS` (contour; **not** `TrnS`). A psd-tools round-trip
  of an authored `ChFX` confirms the key list and decodes back to a `Satin`.
- `pictura-render`'s CPU compositor renders satin **above** the layer content and
  confined to the masked content matte `M`: a Gaussian blur `B` of `M` by `size`
  is differenced against itself shifted by `distance` along `angle`, the absolute
  difference forms the directional band, `Invert` uses the complement, and the
  result is tinted by `color`/`opacity` and composited with the effect's own
  `blend_mode`. This is libpsd `src/satin.c`'s `psd_satin_blend_offset` model:
  `band(x, y) = |B(x − dx, y − dy) − B(x + dx, y + dy)|` with
  `dx = −round(distance·cos(angle))`, `dy = +round(distance·sin(angle))` — the
  docs' "bevel's distance field without the lighting".
- GPU: `check_supported` also rejects an enabled and present `ChFX`, returning
  the existing `GpuError::UnsupportedLayerEffect` before dispatch; disabled,
  absent or malformed effects do not reject. No GPU shader is added.
- The app needs **no production change**: satin renders through the existing
  `composite_rgba` / `composite_active` path. No authoring UI is added.
- Tests: descriptor decode (full, defaults, disabled, absent, malformed, wrong
  typeIDs, non-finite, clamp, invert) and rendering (interior only, exterior
  unchanged, angle/distance/size/opacity/colour/invert/`blend_mode` change the
  result, mask, bbox early-out, disabled byte-identical), plus GPU rejection and
  a psd-tools-authored `satin.psd` fixture with a whole-document round-trip and a
  self-skipping psd-tools read oracle.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- none: satin extends the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode an `lfx2` `ChFX` satin into typed
  parameters and to render it on the CPU as a directional interior band above the
  layer content. No existing requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer decoding to an enabled and present
  `Satin`, like an enabled `DropShadow`, `OuterGlow`, `InnerShadow`, `InnerGlow`,
  `Stroke` or Overlay, is rejected before dispatch.

## Impact

- `crates/pictura-render/src/layer_effects/satin.rs` (new): the `Satin` struct,
  `decode_satin` and `composite_satin`.
- `crates/pictura-render/src/layer_effects/mod.rs`: declare `mod satin;`,
  re-export the public type, call satin from `composite_layer_effects_above`
  after Inner Glow and before the overlays, and add the module doc paragraph.
- `crates/pictura-render/src/lib.rs`: re-export `decode_satin` / `Satin`.
- `crates/pictura-render/src/gpu/mod.rs`: extend the `check_supported` effect
  rejection predicate.
- `crates/pictura-render/src/tests/layer_effects/satin.rs` (new) declared from
  `tests/layer_effects.rs`: decode and render tests, under the 1400 LOC test cap.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/satin.psd`,
  `crates/pictura-codec/tests/oracle.rs` + `tests/oracle/satin.rs`: the builder,
  the new golden fixture, and its whole-document round-trip plus self-skipping
  psd-tools read oracle.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- Contour (`MpgS`) and anti-alias (`AntA`); the effective angle is the stored
  `lagl`, not the document global-light resource (1037); `uglg`, `showInDialog`
  and `AntA` are decoded/ignored.
- libpsd's extra knockout pass (it re-multiplies the content coverage, squaring
  `M` for an unmasked layer); this change confines the band to `M` once.
- Bevel & Emboss (`ebbl`), the legacy `lrFX` block, styles on groups, the
  isolated `Blend Interior Effects As Group` composite, `Scale Effects`, the
  exact Photoshop inter-effect order, a GPU satin shader and any authoring UI.
