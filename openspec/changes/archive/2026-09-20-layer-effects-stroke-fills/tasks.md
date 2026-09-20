## 1. Shared helpers (pure move / visibility)

- [x] 1.1 Move `with_gradient_defaults` from `crates/pictura-render/src/layer_effects/overlays.rs` to `crates/pictura-render/src/layer_effects/mod.rs` unchanged as `pub(crate)`, update the overlay call site to `super::with_gradient_defaults`, and move the doc comment with it. Pure move; the overlay decode contract is unchanged.
- [x] 1.2 In `crates/pictura-render/src/fill.rs`, extract the loop of `pattern_tile_rgba` into `pub(crate) fn pattern_tile_region(patterns: &[PatternPixels], params: &PatternFillParams, anchor: (i32, i32), region: (i32, i32, i32, i32)) -> Vec<[u8; 4]>` and make `pattern_tile_rgba` a thin wrapper with `anchor = (rect_left, rect_top)` and `region = (rect_left, rect_top, rect_left + w, rect_top + h)`. Confirm `composite_pattern_fill` and `composite_pattern_overlay` outputs are byte-identical.
- [x] 1.3 Re-export `StrokeFill` from `crates/pictura-render/src/layer_effects/mod.rs` (`pub use strokes::{decode_stroke, Stroke, StrokeFill, StrokePosition};`) and from `crates/pictura-render/src/lib.rs`'s `pub use layer_effects::{...}` list.

## 2. `StrokeFill` and the stroke decoder

- [x] 2.1 In `crates/pictura-render/src/layer_effects/strokes.rs`, add `pub enum StrokeFill { Solid([u8; 3]), Gradient { params: GradientFillParams, align_with_layer: bool }, Pattern { params: PatternFillParams, angle_deg: f32 } }` (`Debug, Clone, PartialEq`), importing `GradientFillParams`/`PatternFillParams` from `pictura_adjust`.
- [x] 2.2 Replace `Stroke.color: [u8; 3]` with `Stroke.fill: StrokeFill` and change the derive from `Clone, Copy, PartialEq` to `Clone, PartialEq` (the `Vec`/`String` payloads forbid `Copy`). Keep `StrokePosition` `Copy`.
- [x] 2.3 In `decode_stroke`, branch on `PntT` (typeID `FrFl`): absent → `Solid(clr)` (shipped default), `SClr` → `Solid(clr)` from `Clr ` (`RGBC`, default black, non-`RGBC` → `None`), `GrFl` → `Gradient` via `crate::fill::gradient_params_from_desc(&with_gradient_defaults(frfx))` plus `bool_or(frfx, b"Algn", true)`, `Ptrn` → `Pattern` via `crate::fill::pattern_params_from_desc(frfx)` with `link_with_layer` overridden from `Lnkd` when present (else the helper's value) and `angle_deg` from `num_or(frfx, b"Angl", 0.0)`. Unknown `PntT` → `None`. `Clr ` is ignored for `GrFl`/`Ptrn`. A missing/malformed `Grad`/`Ptrn` for the matching fill is `None`; never panic.
- [x] 2.4 Update the module `ponytail:` comment: gradient noise/`Dither`/`Ofst`/midpoints, pattern `Angl` rotation, the aligned-gradient edge clamp and the layer-rect pattern anchor (design D4), the black `Clr ` default, and the ignored contour/anti-alias/overprint.
- [x] 2.5 Update the existing decoder tests in `crates/pictura-render/src/tests/layer_effects/stroke.rs` from `stroke.color` to `stroke.fill` (`Solid(...)`) at the current assertions, and add: a full `GrFl` decodes to the expected `Gradient` params/`align_with_layer`; a full `Ptrn` decodes to the expected `Pattern` params/`angle_deg`; `Lnkd` overrides the link flag and an absent `Lnkd`/`Algn` defaults true; a minimal `FrFX` still yields `Solid([0, 0, 0])`; `PntT` `GrFl` without `Grad`, a non-object `Grad`, a `GrdF` that is not `CstS`, a single/non-increasing stop list, `PntT` `Ptrn` without `Ptrn`, a non-object `Ptrn`, and a missing `Idnt` each return `None` without panicking; non-finite `Angl`/`Scl ` reject.

## 3. Renderer: fill sources over the content-edge band

- [x] 3.1 In `composite_stroke`, keep the shipped band build (`content_matte`, `clip_rect`, `pad_rect`, `dilate_matte`, `erode_matte`, `clamp_finite`) and early-outs, and derive the per-pixel source from `stroke.fill` over `padded` (design D4/D7): `Solid` → the flat colour, alpha 1; `Gradient` → `gradient_rgba` over the layer rect when `align_with_layer` (sample `(x - left, y - top)` clamped to `[0, gw-1]`/`[0, gh-1]`) else over the canvas (sample `(x, y)`), alpha 1; `Pattern` → `pattern_tile_region` over `padded` anchored to the layer rect when `params.link_with_layer` else `(0, 0)`, alpha `tile_alpha/255`.
- [x] 3.2 Composite `band · source_alpha · opacity/100` with `blend_parts` and the stroke's `blend_mode`; clamp the numerics again so a hand-built `Stroke` cannot panic; never panic. Confirm the `Solid` path is byte-identical to the shipped renderer.
- [x] 3.3 Add render tests in `tests/layer_effects/stroke.rs`: a gradient stroke tints the outside band with varying colours and leaves the interior byte-identical; a pattern stroke tiles the band and shows the fixture pattern, and an unknown pattern id shows the grey placeholder; `align_with_layer` true/false differ over the band; a bad `Grad`/`Ptrn` is a bounded no-op; the existing position/size/opacity/blend/mask/bbox/disabled scenarios still hold for the solid path.

## 4. GPU fallback

- [x] 4.1 Confirm no `crates/pictura-render/src/gpu/mod.rs` production change is needed: the existing `effects.stroke` enabled-and-present predicate now fires for a gradient/pattern stroke because `decode_stroke` returns `Some`. Do not add a new error variant.
- [x] 4.2 Add tests: `composite_gpu` on a gradient-stroke and a pattern-stroke document returns `UnsupportedLayerEffect` without panicking, `composite_gpu_or_cpu` equals `composite_rgba` for each, a malformed/non-decodable fill does not produce the error, and a disabled stroke does not produce the error.

## 5. Fixtures and oracles

- [x] 5.1 Add `stroke_gradient()` and `stroke_pattern()` builders to `scripts/generate-fixtures.py` mirroring `stroke()`: a `Base` pixel layer plus a `Stroked` pixel layer with an `FrFX` object whose `PntT` is `GrFl`/`GradientFill` plus a `Grad` `Grdn` object (`GrdF` `CstS`, black→white `Clrs`, `Angl`, `Type` `GrdT`/`Lnr `, `Rvrs`, `Algn`, `Scl `), or `Ptrn`/`Pattern` plus a `Ptrn` object (`Nm  `/`Idnt` `pictura-pattern`, `Scl `, `Lnkd`, `Angl`), under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. The pattern builder writes `_fixture_pattern()` to the global `Patt` block as `pattern_overlay()` does. Register `"stroke_gradient.psd"` and `"stroke_pattern.psd"` in `FIXTURES`.
- [x] 5.2 Regenerate with `python3 scripts/generate-fixtures.py`, add the two fixtures, and confirm every existing fixture is byte-identical (`git status`).
- [x] 5.3 Register the two fixtures in the codec `FIXTURES` table (`("<name>.psd", 8, 8, ColorMode::Rgb)`) and add oracle tests in `crates/pictura-codec/tests/oracle/stroke_gradient.rs` / `stroke_pattern.rs` (declared with `#[path]` from `oracle.rs`): the `lfx2` key is present in `extra_blocks` with version 1 / data version 16, the whole `Document` round-trips `write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check reads the effect as a `Stroke` with `fill_type` `GrFl`/`Ptrn` and the type-specific accessors (`gradient`, `type`, `angle`, `reversed`; `pattern`).
- [x] 5.4 Add render tests that load each fixture with `include_bytes!` (via `../../../../pictura-codec/tests/fixtures/<name>.psd`), assert `decode_stroke` yields the authored `StrokeFill`, and composite a result that differs from the no-effect composite (the pattern fixture resolves the real tile).
- [x] 5.5 Update `crates/pictura-codec/tests/fixtures/README.md`: the two contents rows and the `stroke_gradient()`/`stroke_pattern()` snippets, noting the `Grd`/`Ptrn` content keys and that the stroke pattern link key is `Lnkd`.

## 6. App

- [x] 6.1 Confirm no production app change is needed: the canvas composites through `pictura_render::composite_rgba` / `composite_active`, which now render the gradient/pattern stroke. Do not add an authoring command, panel, or `CMakeLists.txt` entry. No C++ self-test check is added.

## 7. Verification fixes

- [x] 7.1 Bound every decoded numeric (opacity `0..=100`, size rounded then `1..=250`, gradient/pattern numerics finite), re-check `is_finite()` after the `f32` cast, and clamp again in `composite_stroke` so a hand-built `Stroke` cannot panic the gradient/pattern sampling.
- [x] 7.2 Confirm the gradient sample is clamped to the layer-rect buffer edges (or is in-range for the canvas buffer) and the pattern anchor is the layer rect when linked, so a stroke and an overlay with the same params agree over their shared region (design D4).
- [x] 7.3 Confirm the `Solid` composite is byte-identical to the previous solid-only renderer and every existing golden is unchanged; state "no golden change" in the commit message unless a regeneration proves otherwise.
- [x] 7.4 Confirm the bbox confinement: a small layer with a large `size` leaves outside-the-padded-rect pixels byte-identical; the `O(canvas · size)` max/min ceiling plus one gradient/pattern buffer is the documented cost.
- [x] 7.5 Keep `layer_effects/strokes.rs` under its 1200 LOC cap and `tests/layer_effects/stroke.rs` under 1400.

## 8. Gates

- [x] 8.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [x] 8.2 `bash scripts/verify-fast.sh` and a headless self-test; record counts.
- [x] 8.3 `openspec validate layer-effects-stroke-fills --strict` and `openspec validate --all --strict`.
- [x] 8.4 Commit with the two new golden fixtures; state the fixture additions and "no golden change to existing fixtures" in the commit message. Update `docs/dev/STATE.md` separately under `TASK-ALLOWS-DOCS` if the milestone anchor is advanced.
