## Why

The `lfx2` layer-effects framework shipped for Drop Shadow, but the next most
common effect — an **Outer Glow** — is still preserved only opaquely: a real file
with a glowing layer renders flat. A layer carrying `lfx2` `OrGl` draws as if it
had no style. Decoding and rendering Outer Glow is the natural next slice of
LAY-011 and makes common real files (buttons, type, neon) look right.

## What Changes

- `pictura-render`'s `layer_effects` module gains a typed
  `OuterGlow { enabled, present, blend_mode, color, opacity, spread, size,
  technique }` and `decode_outer_glow(&Layer) -> Option<OuterGlow>`, mirroring
  `decode_drop_shadow`. It decodes the `lfx2` `DescriptorBlock2` top-level `OrGl`
  object: `enab`, `present`, `Md  ` (typeID `BlnM`), `Clr ` (`RGBC`), `Opct`,
  `Ckmt` (spread), `blur` (size), and `GlwT` (typeID `BETE`, `SfBL` Softer /
  `PrBL` Precise). A missing/malformed block or a missing `OrGl` is `None` and
  never panics; a disabled (`enab` false) or not-present (`present` false) effect
  decodes but is inert. Defaults follow Photoshop: Screen, opacity 75, spread 0,
  size 5, colour `#FFFFBE`, technique Softer. Numeric values are clamped
  (`spread` `0..=100`, `size` `0..=250`) and a non-finite value rejects the
  effect. There is no offset, angle, or distance for a glow.
- `pictura-render`'s CPU compositor renders the glow before the layer's own
  content: it builds the content coverage matte (pixel alpha, fill alpha, or
  opaque gradient/smart coverage) times the layer mask, dilates it by `spread`
  (max filter), Gaussian-blurs it by `size`, multiplies by the exterior mask
  `1 - matte` so the glow is zero inside opaque content, tints it by `color` and
  `opacity`, and blends it into the canvas behind the content with the effect's
  own blend mode. It reuses the drop-shadow bbox-restricted pipeline, matte,
  dilate, blur and `blend_parts`; no new dependency. `Technique::Precise` is
  decoded but rendered as `Softer` (stated ceiling).
- GPU: `check_supported` also rejects an enabled `OrGl`, returning the existing
  `GpuError::UnsupportedLayerEffect` before dispatch, so `composite_active`
  falls back to the CPU composite. No GPU shader is added.
- The app needs **no production change**: a document with a glowing layer renders
  through the existing `composite_rgba` / `composite_active` path. No authoring
  UI or command is added.
- Tests: descriptor decode (full, defaults, disabled, absent, malformed, wrong
  types, non-finite, out-of-range clamp), rendering (glow surrounds the content,
  is exterior/absent inside, `spread`/`size`/`opacity`/`colour` effects, mask,
  bbox bound, disabled byte-identical), and GPU fallback. A psd-tools-authored
  `outer_glow.psd` fixture with a real `lfx2`/`OrGl` block proves the block
  survives read and decodes.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- none: the effect kind extends the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode an `lfx2` `OrGl` outer glow into
  typed parameters and to render it on the CPU as an exterior matte behind the
  layer content. No existing requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer decoding to an enabled
  `OuterGlow`, like an enabled `DropShadow`, is rejected before dispatch.

## Impact

- `crates/pictura-render/src/layer_effects.rs`: `OuterGlow`, `GlowTechnique`,
  `decode_outer_glow`, and the glow composite helper (reusing the shared matte,
  dilate, blur and bbox helpers).
- `crates/pictura-render/src/composite.rs`: `composite_layer_effects` also
  composites an enabled outer glow.
- `crates/pictura-render/src/lib.rs`: re-export `OuterGlow` (and
  `GlowTechnique`).
- `crates/pictura-render/src/gpu/mod.rs`: extend the `check_supported` effect
  rejection predicate.
- `crates/pictura-render/src/tests/layer_effects.rs`: decode, render and GPU
  tests for the glow.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/outer_glow.psd`,
  `crates/pictura-codec/tests/oracle.rs`: the `outer_glow()` builder, the new
  golden fixture, and its whole-document round-trip plus self-skipping psd-tools
  read oracle.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- Every other effect kind: `IrSh` (Inner Shadow), `IrGl` (Inner Glow), `ebbl`
  (Bevel & Emboss), `ChFX` (Satin), `SoFi` (Color Overlay), `GrFl` (Gradient
  Overlay), `patternFill` (Pattern Overlay), and `FrFX` (Stroke).
- Gradient-mode outer glows (`Grad`): only solid-colour glows are rendered.
- The `PrBL` (Precise) technique: decoded but rendered as the `SfBL` (Softer)
  blur, stated as a ceiling.
- Glow fidelity beyond the first slice: `Range` (`Inpr`), contour (`TrnS`),
  noise (`Nose`), jitter (`ShdN`), anti-alias (`AntA`), `Grad`, layer opacity
  scaling the effect, and `Layer Mask Hides Effects`.
- The legacy `lrFX` effects block and the document global-light resource.
- Layer styles on groups, `Scale Effects`, `Create Layers`, `Rasterize Layer
  Style`, copy/paste styles, the Styles panel, and `.asl` presets.
- A GPU glow shader and any effect-authoring UI.
