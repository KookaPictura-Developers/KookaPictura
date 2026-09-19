## Why

The `lfx2` object-based layer-effects framework now decodes and renders Drop
Shadow (`DrSh`), Outer Glow (`OrGl`) and Inner Shadow (`IrSh`), but the fourth
common kind — an **Inner Glow** (`IrGl`) — is still preserved only opaquely: a
real file with an inner-glowing layer renders flat. Decoding and rendering Inner
Glow is the natural next slice of LAY-011 and makes buttons, glows around type
and lit interiors match Photoshop.

## What Changes

- `pictura-render`'s layer-effects module gains a typed
  `InnerGlow { enabled, present, blend_mode, color, opacity, choke, size, source,
  technique }` and `decode_inner_glow(&Layer) -> Option<InnerGlow>`, mirroring
  `decode_outer_glow`. It decodes the `lfx2` `DescriptorBlock2` top-level `IrGl`
  object: `enab`, `present`, `Md  ` (typeID `BlnM`), `Clr ` (`RGBC`), `Opct`,
  `Ckmt` (Choke, an erode), `blur` (size), `GlwT` (typeID `BETE`, `SfBL`
  Softer / `PrBL` Precise), and `glwS` — the **Source** enum whose typeID is
  **`IGSr`**, with values `SrcE` (Edge) and `SrcC` (Center). A missing/malformed
  block or a missing `IrGl` is `None` and never panics; a disabled (`enab` false)
  or not-present (`present` false) effect decodes but is inert. Defaults follow
  Photoshop: Screen, white, opacity 75, choke 0, size 5, Source Edge, Technique
  Softer. Numeric values are clamped (`opacity`/`choke` `0..=100`, `size`
  `0..=250`) and a non-finite value rejects the effect.
- `pictura-render`'s CPU compositor renders the inner glow **inside** the layer's
  coverage, **above** the layer content, with no offset. It builds the masked
  content matte `M`, erodes `M` by `round(choke/100 · size)`, Gaussian-blurs the
  result into `B`, and forms the interior field `1 - B` for **Source = Edge** (a
  band at the content edge fading inward) or `B` for **Source = Center** (a
  center-weighted interior), then composites `M · field · opacity`, tinted by
  `color`, with the effect's own blend mode. It reuses the bbox/padded pipeline,
  `content_matte`, `erode_matte`, `blur_matte` and `blend_parts`; no new
  dependency. **Center** is a documented approximation: it lights the interior
  that is far from any edge (the libpsd "reverse the alpha after processing"
  behavior), not a bounded radius-`size` blob measured from the centroid.
- Module split by pure moves: `layer_effects.rs` becomes
  `layer_effects/{mod,shadows,glows}.rs` so the shared matte/blur/geometry
  plumbing is separated from per-effect decode and composite and no file
  approaches the 1200 LOC cap.
- GPU: `check_supported` also rejects an enabled `IrGl`, returning the existing
  `GpuError::UnsupportedLayerEffect` before dispatch, so `composite_active` falls
  back to the CPU composite. No GPU shader is added.
- The app needs **no production change**: a document with an inner-glowing layer
  renders through the existing `composite_rgba` / `composite_active` path. No
  authoring UI or command is added.
- Tests: descriptor decode (full, defaults, disabled, absent, malformed, wrong
  types, non-finite, out-of-range clamp), rendering (interior only / exterior
  byte-identical, Edge vs Center differ, `choke`/`size`/`opacity`/`colour`
  effects, mask, bbox bound, disabled byte-identical), and GPU fallback. A
  psd-tools-authored `inner_glow.psd` fixture with a real `lfx2`/`IrGl` block
  proves the block survives read and decodes, including the `glwS`/`IGSr`
  source key.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- none: the effect kind extends the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode an `lfx2` `IrGl` inner glow into
  typed parameters (including the `glwS` Source enum) and to render it on the CPU
  as an interior matte above the layer content. No existing requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer decoding to an enabled
  `InnerGlow`, like an enabled `DropShadow`, `OuterGlow` or `InnerShadow`, is
  rejected before dispatch.

## Impact

- `crates/pictura-render/src/layer_effects.rs` → the
  `layer_effects/{mod,shadows,glows}.rs` submodule: shared descriptor readers,
  matte/blur/geometry helpers and the below/above entry points in `mod.rs`;
  `DropShadow`/`InnerShadow` in `shadows.rs`; `OuterGlow`/`InnerGlow`,
  `GlowSource`, `decode_inner_glow` and `composite_inner_glow` in `glows.rs`.
- `crates/pictura-render/src/composite.rs`: the above-content pass also decodes
  and composites the inner glow; the below/above split is unchanged.
- `crates/pictura-render/src/lib.rs`: re-export `InnerGlow` and `GlowSource`.
- `crates/pictura-render/src/gpu/mod.rs`: extend the `check_supported` effect
  rejection predicate.
- `crates/pictura-render/src/tests/layer_effects/inner_glow.rs` (new) and
  `crates/pictura-render/src/tests/layer_effects.rs`: decode, render and GPU
  tests for the inner glow.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/inner_glow.psd`,
  `crates/pictura-codec/tests/oracle.rs` + `tests/oracle/inner_glow.rs`: the
  `inner_glow()` builder, the new golden fixture, and its whole-document
  round-trip plus self-skipping psd-tools read oracle.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- Every other effect kind: `ebbl` (Bevel & Emboss), `ChFX` (Satin), `SoFi`
  (Color Overlay), `GrFl` (Gradient Overlay), `patternFill` (Pattern Overlay),
  and `FrFX` (Stroke).
- Gradient glows: the `Grad`/`GrdT` fill and its `Jitter` (`ShdN`); only the
  solid-colour fill is decoded and rendered.
- Glow fidelity beyond the first slice: contour (`TrnS`), noise (`Nose`), the
  `Range` (`Inpr`) remap, anti-alias (`AntA`), the `Precise` technique (rendered
  as `Softer`), the layer opacity scaling the effect, `Layer Mask Hides Effects`,
  and the isolated `Blend Interior Effects As Group` composite.
- A true centroid-distance `Center` glow: this slice approximates Center as the
  complement of the edge field (the libpsd behavior).
- The document global-light resource (image resource 1037); `IrGl` carries no
  angle key, so this is unchanged for the glow.
- The legacy `lrFX` effects block; layer styles on groups, `Scale Effects`,
  `Create Layers`, `Rasterize Layer Style`, copy/paste styles, the Styles panel,
  and `.asl` presets.
- A GPU inner-glow shader and any effect-authoring UI.
