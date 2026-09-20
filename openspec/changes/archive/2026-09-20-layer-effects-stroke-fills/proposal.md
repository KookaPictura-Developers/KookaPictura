## Why

The `lfx2` object-based layer-effects framework now decodes and renders a
solid-colour Stroke (`FrFX`), but a stroke whose fill is a **gradient** (`PntT`
`GrFl`) or a **pattern** (`PntT` `Ptrn`) renders nothing: `decode_stroke`
returns `None` for a non-solid fill (the deferred ceiling named at
`crates/pictura-render/src/layer_effects/strokes.rs:39`). The gradient and
pattern Overlay kinds already decode and composite those same two sources, so
the fill plumbing and geometry exist; only the stroke band needs to route into
it. Closing the deferred ceiling completes the CS6 Stroke fill-type enum and
stops silently dropping the two most common non-colour strokes on text and
shapes.

## What Changes

- `crates/pictura-render/src/layer_effects/strokes.rs` replaces `Stroke.color:
  [u8; 3]` with `Stroke.fill: StrokeFill`, where
  `StrokeFill { Solid([u8; 3]), Gradient { .. }, Pattern { .. } }` carries the
  reused `GradientFillParams` / `PatternFillParams` payloads and their
  alignment. `Stroke` becomes `Clone` (it is no longer `Copy`); `StrokePosition`
  is unchanged. The solid path composites **byte-identically** to today.
- `decode_stroke` decodes `PntT` (typeID `FrFl`): `SClr` → `Solid` (unchanged),
  `GrFl` → the gradient content under `Grad` (keys `Angl`/`Type`/`Rvrs`/`Scl `
  decoded by the reused `fill::gradient_params_from_desc`, `Algn` as
  `align_with_layer`), and `Ptrn` → the pattern content under `Ptrn` (the reused
  `fill::pattern_params_from_desc` for `Ptrn`/`Scl `/`phase`, the grounded
  stroke-pattern link key `Lnkd` as `link_with_layer`). An absent `PntT` still
  decodes as solid; an unknown value is still `None`. Absent gradient keys take
  the overlay defaults (injected by the promoted `with_gradient_defaults`), an
  absent `Grad`/`Ptrn` for the matching fill type is `None`, a wrongly-typed key
  is `None`, and the decoder never panics.
- `with_gradient_defaults` moves from `layer_effects/overlays.rs` to
  `layer_effects/mod.rs` as a shared `pub(crate)` helper (a pure move), so the
  stroke and the gradient overlay inject the same absent-`Angl`/`Type` defaults.
- The CPU compositor keeps the shipped content-edge band (`dilate`/`erode` max
  and min filters, position-selected) and swaps the flat tint for the fill
  source: the gradient colour sampled from a layer-rect buffer when
  `align_with_layer` (out-of-rect samples clamped to the nearest edge, matching
  Photoshop's gradient endpoint clamp) or a canvas buffer otherwise, and the
  pattern tile sampled through the shipped `Tile` sampler anchored to the layer
  rect (linked) or the canvas origin (unlinked), so the pattern tiles/repeats
  beyond the content rect. This stroke-fill extent is a stated approximation
  (`ponytail:` ceiling), not a parity claim.
- GPU: `check_supported` needs **no code change** — its existing enabled-and-
  present stroke predicate now fires for a gradient or pattern stroke because
  `decode_stroke` returns `Some`, so the document falls back to the CPU
  composite instead of silently dropping the stroke. The `gpu-compositing`
  requirement text changes because a non-solid stroke is no longer "not an
  effect".
- The app needs **no production change**: the canvas composites through the
  existing `composite_rgba` / `composite_active` path. No authoring UI is added.
- Fixtures: `stroke_gradient.psd` and `stroke_pattern.psd` are authored by the
  existing psd-tools `DescriptorBlock2` path (`stroke()`'s builder with the
  `Grad`/`Ptrn` keys), committed as codec golden fixtures, and covered by a
  whole-document round-trip plus a self-skipping psd-tools read oracle and a
  render test. The pattern fixture writes the existing 2×2 `Patt` pattern so the
  renderer resolves the real tile. Existing fixtures and goldens are unchanged.
- **BREAKING**: none for PSD compatibility. The Rust `Stroke` struct changes
  shape (`color` → `fill`) and drops `Copy`, an internal public-API change.

## Capabilities

### New Capabilities

<!-- none: the fill types extend the existing layer-effects capability -->

### Modified Capabilities

- `layer-effects`: the two existing Stroke requirements ("The object-based
  effects descriptor decodes a stroke" and "A stroke composites as a band at the
  content edge") are modified — the decoder now yields a `StrokeFill` for a
  gradient or pattern `PntT`, and the compositor fills the band from the
  gradient or pattern source. The non-solid deferral scenario is replaced.
- `gpu-compositing`: the existing "Layer effects are rejected before GPU
  dispatch" requirement changes so any enabled and present `Stroke`, not only a
  solid-colour one, is effect-bearing, and removes the clause that a non-solid
  stroke does not reject.

## Impact

- `crates/pictura-render/src/layer_effects/strokes.rs`: `StrokeFill`,
  `Stroke.fill`, `decode_stroke`, `composite_stroke`.
- `crates/pictura-render/src/layer_effects/mod.rs`: promote
  `with_gradient_defaults` (pure move), re-export `StrokeFill`.
- `crates/pictura-render/src/layer_effects/overlays.rs`: call the promoted
  helper (import change only).
- `crates/pictura-render/src/fill.rs`: a pure extraction of an anchor-aware
  `pattern_tile_region` from `pattern_tile_rgba` (design D6), which becomes a
  thin wrapper; both existing callers are byte-identical.
- `crates/pictura-render/src/lib.rs`: re-export `StrokeFill`.
- `crates/pictura-render/src/tests/layer_effects/stroke.rs`: update the
  `stroke.color` assertions to `stroke.fill`, add gradient/pattern decode and
  render tests; keep the file under the 1400 LOC test cap.
- `crates/pictura-render/src/gpu/mod.rs`: no production change; add tests that a
  gradient/pattern stroke rejects the GPU and a malformed/non-decodable fill
  does not.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/{stroke_gradient,stroke_pattern}.psd`,
  `crates/pictura-codec/tests/oracle.rs` +
  `tests/oracle/{stroke_gradient,stroke_pattern}.rs`: the builders, the golden
  fixtures, and their whole-document round-trip plus self-skipping psd-tools read
  oracles.
- `crates/pictura-codec/tests/fixtures/README.md`: the two fixture rows and
  builder snippets, noting the stroke-pattern link key is `Lnkd` (not `Algn`).
- No new dependency. No `docs/` change. No `CMakeLists.txt` change.

## Out of scope (deferred)

- Gradient noise, CS6 `Dither`, `Ofst` offset, stop midpoints and non-linear
  interpolation (inherited from `fill.rs`); pattern rotation (`Angl` decoded for
  symmetry but not applied).
- Contour (`TrnS`), anti-alias (`AntA`), `overprint`, `Scale Effects`, and the
  exact Photoshop inter-effect order.
- The exact Photoshop stroke gradient/pattern extent and anchor (the stated
  approximation): whether Photoshop clamps gradient samples at the padded band
  edge and where it anchors the pattern phase for a stroke is ungrounded without
  a CS6 pixel baseline.
- A GPU stroke shader and any effect-authoring UI.
