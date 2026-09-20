## 1. Satin decoder

- [x] 1.1 Add `crates/pictura-render/src/layer_effects/satin.rs` with `Satin { enabled, present, blend_mode, color, opacity, angle_deg, distance, size, invert }`.
- [x] 1.2 Add `pub fn decode_satin(layer: &Layer) -> Option<Satin>`: take `lfx2` via `Layer::extra_block`, require `>= 8` bytes, hand `&data[4..]` to `read_descriptor`, require the top-level object, find the `ChFX` object (an `Objc` with class id `ChFX`), and decode `enab`, `present`, `Md  ` (`BlnM`, default/unknown → `Multiply`), `Clr ` (`RGBC`, default black), `Opct` (default 50, clamp 0..=100), `lagl` (default 19), `Dstn` (default 11, clamp 0..=30000), `blur` (default 14, clamp 0..=250), `Invr` (default false). Ignore `MpgS`, `AntA`, `uglg`, `showInDialog`.
- [x] 1.3 Accept `UnitFloat` or `Double` numerics; reject a non-finite value or a finite `f64` that overflows `f32`. A missing `lfx2`, a missing `ChFX`, an unknown data version, a wrong class id, a wrong-typed numeric, a wrong `Md  ` typeID, a non-`RGBC` `Clr `, or a parse failure returns `None`; never panic.
- [x] 1.4 Re-export `decode_satin` / `Satin` from `crates/pictura-render/src/layer_effects/mod.rs` and `crates/pictura-render/src/lib.rs`; add the module `ponytail:` ceiling comment (contour `MpgS`, anti-alias `AntA`, `uglg`, global-light resource, the single `M` confinement vs libpsd's knockout, the wider pad region, inter-effect order).

## 2. Satin renderer: a directional interior band above the layer

- [x] 2.1 Add `composite_satin(canvas, layer, doc, satin)` to `satin.rs`: clamp `distance`/`size`/`opacity` with `clamp_finite`, treat a non-finite `angle` as 0, compute `dx = -round(distance·cos(angle))`, `dy = +round(distance·sin(angle))`, `blur_support = ceil(3·sigma_from_radius(size))`, and build `padded = pad_rect(source, |dx|max + |dy|max + blur_support)`; early-out on an empty `source`/`padded`, an all-zero `M`, or `opacity` 0.
- [x] 2.2 Build `M` over `padded` with `content_matte` multiplied by `mask_alpha/255`, blur it with `blur_matte(.., size)`, and for each pixel in `source` sample `B(x-dx, y-dy)` and `B(x+dx, y+dy)` with bounds checks (out-of-region = 0), form `band = |b1 - b2|` clamped `0..=1`, `field = invert ? 1 - band : band`, `alpha = M · field · opacity/100`, and composite the colour with `blend_parts` and the effect's own `blend_mode`.
- [x] 2.3 Call satin from `composite_layer_effects_above` in `mod.rs` when `enabled && present`, after Inner Shadow and Inner Glow and before the overlays and Stroke, keeping the shared group / destructive-adjustment skip. Every pixel outside the content rect is byte-identical to the no-effect composite; a hand-built struct cannot panic.
- [x] 2.4 Add decoder tests in `crates/pictura-render/src/tests/layer_effects/satin.rs`: a full `ChFX` decodes to the expected `Satin`; a minimal `ChFX` takes the defaults (Multiply, black, opacity 50, angle 19, distance 11, size 14, invert false); `Doub` numerics decode; a disabled `ChFX` decodes but renders nothing; missing `lfx2`, missing `ChFX`, wrong data version, wrong class id, wrong-typed `Md  `, non-`RGBC` `Clr `, non-finite `Opct`/`Dstn`/`blur`, and a truncated block return `None`; `blur`/`Dstn`/`Opct` 1e30 clamp to 250/30000/100 and 1e300 rejects; `Invr` true is read.
- [x] 2.5 Add render tests (same file): the satin is interior and every exterior pixel is byte-identical; two `angle_deg` values and two `distance` values change the result; a larger `size` widens the transition; `invert` true differs from false; `opacity` and a non-`Normal` `blend_mode` change the result; a layer mask hides the satin where zero; a small layer on a larger canvas leaves outside-the-rect pixels byte-identical; a disabled/absent satin and a `distance` 0 / `invert` false satin are byte-identical to no effect; a crafted non-finite satin is a bounded no-op.

## 3. GPU fallback

- [x] 3.1 Extend `check_supported`'s `walk` in `crates/pictura-render/src/gpu/mod.rs` so a visible layer whose `decode_satin` is enabled and present also returns `GpuError::UnsupportedLayerEffect` before dispatch, ahead of the adjustment check; keep disabled/absent/malformed from rejecting the document, and add no new error variant.
- [x] 3.2 Add tests: `composite_gpu` on the satin document returns `UnsupportedLayerEffect` without panicking, `composite_gpu_or_cpu` equals `composite_rgba`, a disabled satin does not produce the error, and a malformed satin does not produce the error.

## 4. Fixture and oracle

- [x] 4.1 Add `satin()` to `scripts/generate-fixtures.py`: a `Base` pixel layer plus a `ChFX`-bearing pixel layer whose record carries `DescriptorBlock2(Descriptor({ masterFXSwitch: Bool(True), ChFX: Descriptor({ enab, present, showInDialog, Md  : Enumerated(b"BlnM", b"mul "), Clr : RGBC(10,20,30), Opct: 50 %, uglg: false, lagl: 120°, Dstn: 8 px, blur: 6 px, Invr: true, AntA: true, MpgS: Descriptor({Name:"Linear"}, classID=b"TrnS") }, classID=b"ChFX") }, classID=Klass.Null))` under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. Register `"satin.psd"` in `FIXTURES`.
- [x] 4.2 Regenerate with `python3 scripts/generate-fixtures.py`, add the fixture, and confirm the existing fixtures are byte-identical (`git status`).
- [x] 4.3 Register the fixture in the codec `FIXTURES` table (`("satin.psd", 8, 8, ColorMode::Rgb)`) and add `crates/pictura-codec/tests/oracle/satin.rs` (declared with `#[path]` from `oracle.rs`): the `lfx2` key is present in `extra_blocks` with version 1 / data version 16, the whole `Document` round-trips `write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check reads the effect as a `Satin` asserting `enabled`, `present`, `opacity`, `blend_mode`, `inverted`, `angle`, `distance`, `size` and the colour rgb.
- [x] 4.4 Add a render test that loads the fixture with `include_bytes!`, asserts `decode_satin` yields the authored parameters, and composites a result that differs from the no-effect composite.
- [x] 4.5 Update `crates/pictura-codec/tests/fixtures/README.md`: the fixture row and the `satin()` snippet, noting the `ChFX` object class id, the `Invr` invert key and the `MpgS` (not `TrnS`) contour key.

## 5. App

- [x] 5.1 Confirm no production app change is needed: the canvas composites through `pictura_render::composite_rgba` / `composite_active`, which now render satin. Do not add an authoring command, panel, or `CMakeLists.txt` entry. No C++ self-test check is added.

## 6. Verification fixes

- [x] 6.1 Bound every decoded numeric to its documented range (`opacity` `0..=100`, `distance` `0..=30000`, `size` `0..=250`) and re-check `is_finite()` after the `f32` cast; clamp again in `composite_satin` so a hand-built struct cannot panic the blur or the sampling.
- [x] 6.2 Confirm `M` gates the band (multiply), so the satin is zero wherever the content is absent and confined to the interior where `M > 0`.
- [x] 6.3 Confirm the bbox confinement: a small satin layer on a larger canvas leaves outside-the-padded-content pixels byte-identical; the `dist_reach + blur_support` pad is the documented ceiling.
- [x] 6.4 Keep `layer_effects/satin.rs` under its 1200 LOC cap; keep `tests/layer_effects/satin.rs` under 1400 and declare it from `tests/layer_effects.rs` beside the overlays.

## 7. Gates

- [x] 7.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [x] 7.2 `bash scripts/verify-fast.sh` and a headless self-test; record counts.
- [x] 7.3 `openspec validate layer-effects-satin --strict` and `openspec validate --all --strict`.
- [x] 7.4 Commit with the new golden fixture; state the fixture addition in the commit message. Update `docs/dev/STATE.md` separately under `TASK-ALLOWS-DOCS` if the milestone anchor is advanced.
