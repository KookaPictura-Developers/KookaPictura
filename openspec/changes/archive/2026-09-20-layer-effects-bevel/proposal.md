## Why

The `lfx2` object-based layer-effects framework decodes and renders Drop Shadow
(`DrSh`), Outer Glow (`OrGl`), Inner Shadow (`IrSh`), Inner Glow (`IrGl`),
Stroke (`FrFX`), the Color/Gradient/Pattern Overlays and Satin (`ChFX`). A layer
with **Bevel & Emboss** still renders without it: the `ebbl` object is preserved
opaquely in `extra_blocks` and ignored. Bevel & Emboss is the most-used surface
effect for buttons, type and icons, so a layer with only a bevel is the largest
remaining hole in the common CS6 style set.

## What Changes

- `pictura-render`'s layer-effects module gains `layer_effects/bevel.rs` beside
  `shadows.rs`, `glows.rs`, `strokes.rs`, `overlays.rs` and `satin.rs`, with a
  typed `BevelEmboss { enabled, present, style, technique, direction, depth,
  size, soften, angle_deg, altitude_deg, use_global_angle, highlight: {mode,
  color, opacity}, shadow: {mode, color, opacity} }` and
  `decode_bevel_emboss(&Layer) -> Option<BevelEmboss>`, following the shipped
  finite/clamp/no-panic contract.
- **The effect object class id and its top-level `lfx2` key are both `ebbl`**
  (libpsd `src/effects.c` `case 'ebbl':` → `psd_get_layer_bevel_emboss2`;
  psd-tools `api/effects.py` `@register(Klass.BevelEmboss.value)` where
  `terminology.py` `Klass.BevelEmboss = b"ebbl"`). The object keys are `enab`,
  `present`, `showInDialog`, `hglM`/`hglC`/`hglO` (highlight mode/colour/
  opacity), `sdwM`/`sdwC`/`sdwO` (shadow mode/colour/opacity), `bvlS` (style,
  enum `BESl`), `bvlT` (technique, enum `bvlT`), `bvlD` (direction, enum
  `BESs`), `uglg` (use global light), `lagl` (local angle), `Lald` (altitude),
  `srgR` (depth), `blur` (size), `Sftn` (soften), `TrnS` (gloss contour),
  `MpgS` (contour), `AntA`, `Inpr`, `useShape`, `useTexture` and
  `antialiasGloss`. The enum values are style `InrB`/`OtrB`/`Embs`/`PlEb`/
  `strokeEmboss`, technique `SfBL` (Smooth)/`PrBL` (Chisel Hard)/`Slmt` (Chisel
  Soft), and direction `In  ` (Up)/`Out ` (Down). A psd-tools round-trip of an
  authored `ebbl` confirms the key list, types, stored units and decode.
- `pictura-render`'s CPU compositor renders the **Inner, Smooth** slice of
  Bevel & Emboss **above** the content but confined to the masked content matte
  `M`: a height field derived from the blurred content matte is lit from
  `Angle`/`Altitude` to produce a signed shading, `Direction` flips its sign,
  `Depth` scales it and `Soften` blurs it; the positive and negative parts are
  tinted by the highlight/shadow colour, opacity and blend mode and composited
  inside `M`. Other styles (`Outer`/`Emboss`/`Pillow`/`Stroke`) and the chisel
  techniques decode but render nothing in this slice (see Out of scope).
- GPU: `check_supported` also rejects an enabled and present `ebbl`, returning
  the existing `GpuError::UnsupportedLayerEffect` before dispatch; disabled,
  absent or malformed effects do not reject. No GPU shader is added.
- The app needs **no production change**: a bevel renders through the existing
  `composite_rgba` / `composite_active` path. No authoring UI is added.
- Tests: descriptor decode (full, defaults, disabled, absent, malformed, wrong
  typeIDs, non-finite, clamp, style/technique/direction enum values) and
  rendering (interior only, exterior unchanged, angle/altitude, size/soften/
  depth, direction, highlight/shadow colour+opacity+mode, mask, bbox, disabled
  byte-identical, non-Inner/non-Smooth no-op), plus GPU rejection and a
  psd-tools-authored `bevel.psd` fixture with a whole-document round-trip and a
  self-skipping psd-tools read oracle.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- none: bevel and emboss extends the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode an `lfx2` `ebbl` bevel into typed
  parameters and to render the Inner/Smooth slice on the CPU above the layer
  content. No existing requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer decoding to an enabled and present
  `BevelEmboss`, like the other effects, is rejected before dispatch.

## Impact

- `crates/pictura-render/src/layer_effects/bevel.rs` (new): the `BevelEmboss`
  struct and its enums, `decode_bevel_emboss` and `composite_bevel_emboss`.
- `crates/pictura-render/src/layer_effects/mod.rs`: declare `mod bevel;`,
  re-export the public types and function, call bevel from
  `composite_layer_effects_above` after Inner Glow and before Satin, add the
  `MAX_DEPTH`/`MAX_ALTITUDE` caps and the module doc paragraph.
- `crates/pictura-render/src/lib.rs`: re-export `decode_bevel_emboss` /
  `BevelEmboss` and the enums.
- `crates/pictura-render/src/gpu/mod.rs`: extend the `check_supported` effect
  rejection predicate.
- `crates/pictura-render/src/tests/layer_effects/bevel.rs` (new) declared from
  `tests/layer_effects.rs`: decode and render tests, under the 1400 LOC test cap.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/bevel.psd`,
  `crates/pictura-codec/tests/oracle.rs` + `tests/oracle/bevel.rs`: the builder,
  the new golden fixture, and its whole-document round-trip plus self-skipping
  psd-tools read oracle.
- `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and builder
  snippet.
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- The chisel techniques (`PrBL`, `Slmt`) render as no-ops in this slice; the
  Smooth (`SfBL`) height profile is the only one modelled.
- The `Outer`, `Emboss`, `Pillow` and `Stroke` styles render as no-ops; only
  `InrB` (Inner) is composited. The stroke-emboss style also depends on Stroke.
- The bevel edge **Contour** (`MpgS`), gloss **Contour** (`TrnS`), contour
  `Range` (`Inpr`), anti-alias (`AntA`, `antialiasGloss`), **Texture**
  (`useTexture`, `InvT`, `Algn`, `Scl `, `Ptrn`), `useShape` and `showInDialog`
  are decoded/ignored.
- The effective angle/altitude is the stored `lagl`/`Lald`, not the document
  global-light resource (1037); `uglg` is decoded but inert.
- libpsd's extra knock-out and mask passes, the legacy `lrFX` `bevl` block,
  styles on groups, `Blend Interior Effects As Group`, `Scale Effects`, the
  exact Photoshop inter-effect order, a GPU bevel shader and any authoring UI.
