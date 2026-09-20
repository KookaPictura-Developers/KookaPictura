## Why

The `lfx2` object-based layer-effects framework now decodes and renders Drop
Shadow (`DrSh`), Outer Glow (`OrGl`), Inner Shadow (`IrSh`) and Inner Glow
(`IrGl`), but a real document with a **Stroke** (`FrFX`) still renders flat: the
block is preserved opaquely and ignored. Stroke is the last of the five common
CS6 layer-style kinds and the one most visible on text and shapes, so decoding
and rendering the solid-colour first slice closes the common-styles gap.

## What Changes

- `pictura-render`'s layer-effects module gains `Stroke { enabled, present,
  blend_mode, color, opacity, size, position }` and `decode_stroke(&Layer) ->
  Option<Stroke>`, in a new `layer_effects/strokes.rs` beside `shadows.rs` and
  `glows.rs`. It decodes the `lfx2` `DescriptorBlock2` top-level `FrFX`
  (class id `FrFX`): `enab`, `present`, `Md  ` (typeID `BlnM`) as `blend_mode`
  (default **Normal**), `Clr ` (`RGBC`) as `color` (default **black**), `Opct`
  as `opacity` (default **100**), `Sz  ` as `size` (default **3**, integer
  **1..=250**), and `Styl` (typeID **`FStl`**, values `OutF` Outside / `InsF`
  Inside / `CtrF` Center, default **Outside**) as `position`. A missing/malformed
  block, a wrong typeID, a non-finite value, or a non-solid fill type
  (`PntT`/`FrFl` value `GrFl` gradient or `Ptrn` pattern) is `None` and never
  panics; a disabled or not-present stroke decodes but is inert.
- `pictura-render`'s CPU compositor renders the stroke as a **band at the
  content edge**, composited **above** the layer content. From the masked
  content matte `M` and integer size `n`: **Outside** `band = dilate(M, n) − M`;
  **Inside** `band = M − erode(M, n)`; **Center** `out_r = ceil(n/2)`,
  `in_r = floor(n/2)`, `band = dilate(M, out_r) − erode(M, in_r)`. It tints the
  band with `color` and `opacity`, composites with the stroke's own `blend_mode`
  and no extra layer opacity or fill, and reuses the bbox/padded pipeline
  (`content_matte`, `clip_rect`, `pad_rect`, `rect_empty`, `dilate_matte`,
  `erode_matte`, `clamp_finite`, `blend_parts`). There is no blur; pixel widths
  are exact integer max/min filters.
- GPU: `check_supported` also rejects an enabled and present `FrFX`, returning
  the existing `GpuError::UnsupportedLayerEffect` before dispatch, so
  `composite_active` falls back to the CPU composite. No GPU shader is added.
- The app needs **no production change**: a stroked document renders through the
  existing `composite_rgba` / `composite_active` path. No authoring UI or command
  is added.
- Tests: descriptor decode (full, defaults, disabled, absent, malformed, wrong
  types, non-finite, out-of-range clamp, position, non-solid fill type deferral),
  rendering (outside/inside/centre bands, size, opacity, colour, blend mode,
  mask, bbox, disabled byte-identical), and GPU fallback. A psd-tools-authored
  `stroke.psd` fixture with a real `lfx2`/`FrFX` solid-colour block proves the
  block survives read and decodes.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- none: the effect kind extends the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode an `lfx2` `FrFX` stroke into typed
  parameters (including the `Styl` position enum and the `PntT` fill type) and to
  render it on the CPU as a content-edge band above the layer. No existing
  requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer decoding to an enabled `Stroke`,
  like an enabled `DropShadow`, `OuterGlow`, `InnerShadow` or `InnerGlow`, is
  rejected before dispatch.

## Impact

- `crates/pictura-render/src/layer_effects/strokes.rs` (new): `Stroke`,
  `StrokePosition`, `decode_stroke`, `composite_stroke`.
- `crates/pictura-render/src/layer_effects/mod.rs`: declare `mod strokes;`,
  re-export the public types, and call the stroke from
  `composite_layer_effects_above` after the interior effects.
- `crates/pictura-render/src/lib.rs`: re-export `Stroke` and `StrokePosition`.
- `crates/pictura-render/src/gpu/mod.rs`: extend the `check_supported` effect
  rejection predicate.
- `crates/pictura-render/src/tests/layer_effects/stroke.rs` (new) and
  `crates/pictura-render/src/tests/layer_effects.rs`: decode, render and GPU
  tests; the module stays within the 1400 LOC test cap.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/stroke.psd`,
  `crates/pictura-codec/tests/oracle.rs` + `tests/oracle/stroke.rs`: the
  `stroke()` builder, the new golden fixture, and its whole-document round-trip
  plus self-skipping psd-tools read oracle.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- Gradient (`PntT` `GrFl`) and pattern (`PntT` `Ptrn`) stroke fills; only the
  solid-colour fill is decoded and rendered. A non-solid `FrFX` decodes to `None`
  (a stated ceiling).
- Stroke fidelity beyond the first slice: contour (`TrnS`), anti-alias
  (`AntA`), `overprint`, and the `Scale Effects` multiplier.
- The other effect kinds: `ebbl` (Bevel & Emboss), `ChFX` (Satin), `SoFi`
  (Color Overlay), `GrFl` (Gradient Overlay) and `patternFill` (Pattern
  Overlay).
- Strokes on groups, adjustment layers and smart filters; the legacy `lrFX`
  block; the isolated `Blend Interior Effects As Group` composite; the exact
  Photoshop inter-effect order among the above-content effects.
- A GPU stroke shader and any effect-authoring UI.
