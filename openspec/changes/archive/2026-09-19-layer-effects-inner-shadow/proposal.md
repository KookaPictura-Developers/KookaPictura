## Why

The `lfx2` object-based layer-effects framework shipped for Drop Shadow (`DrSh`)
and Outer Glow (`OrGl`), but the next most common effect — an **Inner Shadow**
(`IrSh`) — is still preserved only opaquely: a real file with an inner-shadowed
layer renders flat. Decoding and rendering Inner Shadow is the natural next slice
of LAY-011 and makes buttons, embossed type and inset surfaces look right.

## What Changes

- `pictura-render`'s `layer_effects` module gains a typed
  `InnerShadow { enabled, present, blend_mode, color, opacity, angle_deg,
  distance, choke, size, use_global_angle, knocks_out }` and
  `decode_inner_shadow(&Layer) -> Option<InnerShadow>`, mirroring
  `decode_drop_shadow`. It decodes the `lfx2` `DescriptorBlock2` top-level `IrSh`
  object: `enab`, `present`, `Md  ` (typeID `BlnM`), `Clr ` (`RGBC`), `Opct`,
  `uglg`, `lagl`, `Dstn`, `Ckmt` (Choke, an erode), `blur` (size), and
  `layerConceals` (carried, no observable effect). A missing/malformed block or a
  missing `IrSh` is `None` and never panics; a disabled (`enab` false) or
  not-present (`present` false) effect decodes but is inert. Defaults follow
  Photoshop: Multiply, black, opacity 75, angle 120, distance 5, choke 0, size 5,
  use-global-angle on. Numeric values are clamped (`opacity`/`choke` `0..=100`,
  `distance` `0..=30000`, `size` `0..=250`) and a non-finite value rejects the
  effect.
- `pictura-render`'s CPU compositor renders the inner shadow **inside** the
  layer's coverage, **above** the layer content: it builds the content coverage
  matte (pixel alpha, fill alpha, or opaque gradient/smart coverage) times the
  layer mask, inverts it to `1 - M`, offsets it by `distance` along the effect
  angle, erodes it by `choke` (min filter), Gaussian-blurs it by `size`, then
  multiplies by the content matte `M` so the shadow is confined to the content
  interior, tints it by `color` and `opacity`, and blends it over the canvas with
  the effect's own blend mode. It reuses the drop-shadow bbox-restricted
  pipeline, matte, blur and `blend_parts`, and re-introduces a min-filter erode
  helper (removed from the drop-shadow slice); no new dependency.
- GPU: `check_supported` also rejects an enabled `IrSh`, returning the existing
  `GpuError::UnsupportedLayerEffect` before dispatch, so `composite_active` falls
  back to the CPU composite. No GPU shader is added.
- The app needs **no production change**: a document with an inner-shadowed layer
  renders through the existing `composite_rgba` / `composite_active` path. No
  authoring UI or command is added.
- Tests: descriptor decode (full, defaults, disabled, absent, malformed, wrong
  types, non-finite, out-of-range clamp), rendering (interior only / exterior
  byte-identical, `distance`/`angle`/`choke`/`size`/`opacity`/`colour` effects,
  mask, bbox bound, disabled byte-identical), and GPU fallback. A
  psd-tools-authored `inner_shadow.psd` fixture with a real `lfx2`/`IrSh` block
  proves the block survives read and decodes.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- none: the effect kind extends the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode an `lfx2` `IrSh` inner shadow into
  typed parameters and to render it on the CPU as an interior matte above the
  layer content. No existing requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer decoding to an enabled
  `InnerShadow`, like an enabled `DropShadow` or `OuterGlow`, is rejected before
  dispatch.

## Impact

- `crates/pictura-render/src/layer_effects.rs`: `InnerShadow`,
  `decode_inner_shadow`, the inner-shadow composite helper, and the `erode_matte`
  helper (reusing the shared matte, blur and bbox helpers).
- `crates/pictura-render/src/composite.rs`: `composite_layer_effects` is split
  into a below-content pass (drop shadow, outer glow) and an above-content pass
  (inner shadow) so the interior shadow composites after the layer's own content.
- `crates/pictura-render/src/lib.rs`: re-export `InnerShadow`.
- `crates/pictura-render/src/gpu/mod.rs`: extend the `check_supported` effect
  rejection predicate.
- `crates/pictura-render/src/tests/layer_effects/inner_shadow.rs` (new) and
  `crates/pictura-render/src/tests/layer_effects.rs`: decode, render and GPU
  tests for the inner shadow.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/inner_shadow.psd`,
  `crates/pictura-codec/tests/oracle.rs`: the `inner_shadow()` builder, the new
  golden fixture, and its whole-document round-trip plus self-skipping psd-tools
  read oracle.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- Every other effect kind: `IrGl` (Inner Glow), `ebbl` (Bevel & Emboss), `ChFX`
  (Satin), `SoFi` (Color Overlay), `GrFl` (Gradient Overlay), `patternFill`
  (Pattern Overlay), and `FrFX` (Stroke).
- The `layerConceals` fireback semantics: the key is decoded into `knocks_out`
  for structural symmetry with `DrSh`, but libpsd's inner-shadow struct and
  psd-tools' `InnerShadow` expose no knock-out control, and the interior
  confinement is unconditional.
- Shadow fidelity beyond the first slice: contour (`TrnS`), noise (`Nose`),
  anti-alias (`AntA`), layer opacity scaling the effect, `Layer Mask Hides
  Effects`, and the isolated `Blend Interior Effects As Group` composite.
- The document global-light resource (image resource 1037): `uglg` is decoded but
  the effective angle uses the effect's stored `lagl`.
- The legacy `lrFX` effects block; layer styles on groups, `Scale Effects`,
  `Create Layers`, `Rasterize Layer Style`, copy/paste styles, the Styles panel,
  and `.asl` presets.
- A GPU inner-shadow shader and any effect-authoring UI.
