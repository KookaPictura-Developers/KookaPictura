## Why

The `lfx2` object-based layer-effects framework now decodes and renders Drop
Shadow (`DrSh`), Outer Glow (`OrGl`), Inner Shadow (`IrSh`), Inner Glow (`IrGl`)
and Stroke (`FrFX`), but a layer with a **Color Overlay**, **Gradient Overlay**
or **Pattern Overlay** still renders without it: the block is preserved opaquely
and ignored. The overlays are the surface effects that fill the layer's own
content, so they are the last of the common CS6 style kinds and the ones most
visible on text and shapes. All three share one algorithm (fill the content
coverage with a source), so they land cleanly as one change.

## What Changes

- `pictura-render`'s layer-effects module gains a new `layer_effects/overlays.rs`
  beside `shadows.rs`, `glows.rs` and `strokes.rs`, with three typed params and
  three decoders, following the same finite/clamp/no-panic contract as the
  shipped kinds:
  - `ColorOverlay { enabled, present, blend_mode, color, opacity }` and
    `decode_color_overlay(&Layer)`;
  - `GradientOverlay { enabled, present, blend_mode, opacity, stops, reverse,
    kind, angle_deg, scale, align_with_layer }` (reusing
    `GradientStop`/`GradientKind`) and `decode_gradient_overlay(&Layer)`;
  - `PatternOverlay { enabled, present, blend_mode, opacity, pattern_id, scale,
    angle_deg, align_with_layer, origin }` and `decode_pattern_overlay(&Layer)`.
- **The effect object class ids are `SoFi`, `GrFl` and `patternFill`**, not the
  `SoCo`/`PtFl` fill-layer class ids named in the brief. Both independent
  implementations agree: psd-tools `api/effects.py:395-407` registers
  `Klass.SolidFill.value` (`b"SoFi"`), `b"GrFl"` and `b"patternFill"` against
  `ColorOverlay`/`GradientOverlay`/`PatternOverlay`, and ag-psd
  `src/descriptor.ts` reads/destructures `SoFi`, `GrFl`, `patternFill` in
  `parseEffects`. `SoCo`/`PtFl` are the **fill-layer** descriptors already
  decoded by `fill.rs`/`composite.rs`. A live psd-tools round-trip probe with
  the effect objects authored under `SoCo`/`PtFl` fails
  (`ValueError: Effect class not found for b'SoCo'`), while `SoFi`/`GrFl`/
  `patternFill` decode cleanly.
- `pictura-render`'s CPU compositor renders each overlay **above** the layer
  content, gated by the masked content coverage `M`: the source colour is
  composited with alpha `M · source_alpha · opacity/100` and the overlay's own
  `blend_mode`. Solid uses the stored colour; gradient reuses
  `fill::gradient_rgba` (angle/scale/reverse/kind) over the layer rect when
  `align_with_layer` is true and the canvas otherwise; pattern reuses the
  document pattern library and the existing tiling (`fill::pattern_tile_rgba`,
  placeholder grey when the id is absent), with `align_with_layer` mapping to
  the tile's linked origin. Exterior pixels (`M = 0`) are untouched.
- `pictura-render/src/fill.rs` exposes two `pub(crate)` descriptor helpers
  (`gradient_params_from_desc`, `pattern_params_from_desc`) extracted from the
  existing strict `decode_gradient_fill`/`decode_pattern_fill` bodies, so the
  overlay decoders share the stop/pattern decoding instead of re-implementing
  it. The public fill decoders keep their exact current contract.
- GPU: `check_supported` also rejects an enabled and present `ColorOverlay`,
  `GradientOverlay` or `PatternOverlay`, returning the existing
  `GpuError::UnsupportedLayerEffect` before dispatch; disabled, absent,
  malformed, or undecodable (non-solid pattern/gradient) effects do not reject.
  No GPU shader is added.
- The app needs **no production change**: overlays render through the existing
  `composite_rgba` / `composite_active` path. No authoring UI is added.
- Tests: per overlay, descriptor decode (full, defaults, disabled, absent,
  malformed, wrong typeIDs, non-finite, clamp) and rendering (confined to the
  content, exterior unchanged, colour/gradient/pattern source, opacity, blend,
  mask, bbox early-out, disabled byte-identical), plus GPU rejection and the
  missing-pattern placeholder fallback. Three psd-tools-authored fixtures
  (`color_overlay.psd`, `gradient_overlay.psd`, `pattern_overlay.psd`) prove the
  `lfx2` block survives read and decodes, with a whole-document round-trip and a
  self-skipping psd-tools read oracle per overlay.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- none: the overlay kinds extend the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: adds requirements to decode an `lfx2` `SoFi` color overlay,
  `GrFl` gradient overlay and `patternFill` pattern overlay into typed
  parameters, and to render each on the CPU as a source fill over the content
  coverage above the layer. No existing requirement changes.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement is extended so a layer decoding to an enabled
  `ColorOverlay`, `GradientOverlay` or `PatternOverlay`, like an enabled
  `DropShadow`, `OuterGlow`, `InnerShadow`, `InnerGlow` or `Stroke`, is rejected
  before dispatch.

## Impact

- `crates/pictura-render/src/layer_effects/overlays.rs` (new): the three params
  structs plus `decode_*`/`composite_*` for each overlay.
- `crates/pictura-render/src/layer_effects/mod.rs`: declare `mod overlays;`,
  re-export the public types, and call the three above-content overlays from
  `composite_layer_effects_above` before the stroke.
- `crates/pictura-render/src/fill.rs`: extract and expose
  `gradient_params_from_desc` / `pattern_params_from_desc` (`pub(crate)`); the
  public decoders keep their behavior.
- `crates/pictura-render/src/lib.rs`: re-export the three params structs.
- `crates/pictura-render/src/gpu/mod.rs`: extend the `check_supported` effect
  rejection predicate.
- `crates/pictura-render/src/tests/layer_effects/{color_overlay,gradient_overlay,pattern_overlay}.rs`
  (new) declared from `tests/layer_effects.rs`: decode and render tests; each
  file stays under the 1400 LOC test cap.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/{color_overlay,gradient_overlay,pattern_overlay}.psd`,
  `crates/pictura-codec/tests/oracle.rs` +
  `tests/oracle/{color_overlay,gradient_overlay,pattern_overlay}.rs`: the
  builders, the new golden fixtures, and their whole-document round-trip plus
  self-skipping psd-tools read oracles.
- `crates/pictura-codec/tests/fixtures/README.md`: the three fixture rows and
  builder snippets.
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- Gradient Overlay noise, `Dither` (`Dthr`, CS6), `Ofst` offset, stop midpoints
  and non-linear interpolation (inherited from `fill.rs`); Pattern Overlay
  rotation (`Angl` is decoded but not applied).
- The remaining effect kinds: `ebbl` (Bevel & Emboss) and `ChFX` (Satin); the
  legacy `lrFX` block; the isolated `Blend Interior Effects As Group` composite.
- Overlays on groups, adjustment layers and smart filters; `Scale Effects`; the
  exact Photoshop inter-effect order.
- A GPU overlay shader and any effect-authoring UI.
