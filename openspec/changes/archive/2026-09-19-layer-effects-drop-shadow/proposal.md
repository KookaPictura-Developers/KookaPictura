## Why

Layer effects ("layer styles") are preserved opaquely but never decoded or
rendered, so a layer carrying the most common style — a Drop Shadow — renders
flat. A real Photoshop file stores the object-based effect set in an `lfx2`
additional-layer-info block; today `read.rs` drops that block to `extra_blocks`,
neither `decode_adjustment` nor the compositor has an arm for it, and the layer
draws as if it had no style. Decoding and rendering Drop Shadow is the first
slice of LAY-011 and makes common real files look right.

## What Changes

- `pictura-core`: `Layer` gains `extra_block(&self, key: &[u8; 4]) -> Option<&LayerBlock>`,
  a typed getter over the preserved `extra_blocks`.
- `pictura-render` gains `layer_effects.rs` with a typed
  `DropShadow { enabled, present, blend_mode, color, opacity, angle_deg, distance,
  spread, size, use_global_angle, knocks_out }` and
  `decode_drop_shadow(&Layer) -> Option<DropShadow>`. It reads the `lfx2`
  `DescriptorBlock2` and its `DrSh` object. A missing/malformed block or a
  missing `DrSh` is `None` and never panics; a disabled (`enab` false) or
  not-present (`present` false) effect decodes but is inert. Numeric values are
  clamped to their documented ranges (`spread` `0..=100`, `size` `0..=250`,
  `distance` `0..=30000`) and a non-finite value rejects the effect.
- `pictura-render`'s CPU compositor renders the drop shadow before the layer's
  own content: it builds a coverage matte from the layer content alpha
  (pixel alpha, fill-content alpha, or smart-object coverage) times the layer
  mask, offsets the matte by `distance` along the effect angle, dilates it by
  `spread`, Gaussian-blurs it by `size`, tints it by `color` and `opacity`, and
  blends it into the canvas behind the content with the effect's own blend mode.
  It reuses the existing `blend_into` and the `pictura-filters` Gaussian blur;
  no new dependency.
- GPU: a visible layer with a decodable effect makes `composite_gpu` return
  `GpuError::UnsupportedLayerEffect` before dispatching, so `composite_active`
  falls back to the CPU composite, mirroring how an unsupported adjustment is
  rejected. No GPU shader is added.
- The app needs **no production change**: a document with a drop-shadowed layer
  renders correctly through the existing `composite_rgba` / `composite_active`
  path. No authoring UI or command is added.
- Tests: descriptor decode (present, defaults, disabled, absent, malformed),
  rendering (offset direction, distance, blur, opacity, colour, mask, disabled
  no-op), and GPU fallback. A psd-tools-authored `drop_shadow.psd` fixture with
  a real `lfx2`/`DrSh` block proves the block survives read and decodes.
- **BREAKING**: none.

## Capabilities

### New Capabilities

- `layer-effects`: decode the `lfx2` object-based layer-effects descriptor's Drop
  Shadow into typed parameters, and render it on the CPU behind the layer
  content.

### Modified Capabilities

- `gpu-compositing`: a new requirement that a visible layer carrying a decodable
  object-based effect is rejected before GPU dispatch and falls back to the CPU
  composite without panicking.

## Impact

- `crates/pictura-core/src/lib.rs`: `Layer::extra_block`.
- `crates/pictura-render/src/layer_effects.rs` (new): `DropShadow`,
  `decode_drop_shadow`, and the shadow composite helpers.
- `crates/pictura-render/src/lib.rs`: module declaration and re-exports.
- `crates/pictura-render/src/composite.rs`: call the shadow composite before a
  non-group layer's content; factor the source-over blend so the shadow can carry
  its own blend mode and opacity.
- `crates/pictura-render/src/fill.rs`: expose the fill content's coverage alpha
  for the matte.
- `crates/pictura-render/src/gpu/mod.rs`: `GpuError::UnsupportedLayerEffect` and
  the `check_supported` effect rejection.
- `crates/pictura-render/src/tests/layer_effects.rs` (new) and
  `crates/pictura-render/src/tests/mod.rs`: decode and render tests.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/drop_shadow.psd`,
  `crates/pictura-codec/tests/oracle.rs`: the `drop_shadow()` builder, the new
  golden fixture, and its read/decode oracle.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change.

## Out of scope (deferred)

- Every other effect kind: `IrSh` (Inner Shadow), `OrGl` (Outer Glow), `IrGl`
  (Inner Glow), `ebbl` (Bevel & Emboss), `ChFX` (Satin), `SoFi` (Color
  Overlay), `GrFl` (Gradient Overlay), `patternFill` (Pattern Overlay), and
  `FrFX` (Stroke).
- The legacy `lrFX` effects block; the docs model `lrFX` but modern Photoshop
  writes `lfx2`.
- The document global-light resource (image resource 1037): `uglg` is decoded
  but the effective angle uses the effect's stored `lagl`.
- Layer styles on groups, `Scale Effects`, `Create Layers`, `Rasterize Layer
  Style`, copy/paste styles, and the Styles panel / `.asl` presets.
- Drop Shadow fidelity beyond the first slice: the `layerConceals` knock-out
  semantics, layer opacity scaling the effect, `Layer Mask Hides Effects`,
  contour (`TrnS`), noise (`Nose`), and anti-alias (`AntA`).
- A GPU drop-shadow shader and any effect-authoring UI.
