## 1. Outer glow decoder

- [x] 1.1 Add `OuterGlow { enabled, present, blend_mode, color, opacity, spread, size, technique }` and `GlowTechnique { Softer, Precise }` to `crates/pictura-render/src/layer_effects.rs`, beside `DropShadow`.
- [x] 1.2 Add `pub fn decode_outer_glow(layer: &Layer) -> Option<OuterGlow>` mirroring `decode_drop_shadow`: take the `lfx2` block via `Layer::extra_block`, require at least `8` bytes, hand `&data[4..]` to `pictura_codec::read_descriptor`, require the top-level object, find `OrGl` (an `Objc` with class id `OrGl`), and decode `enab`, `present`, `Md  ` (blend enum via `BlendMode::from_psd_key`), `Clr `/`RGBC` `Rd `/`Grn `/`Bl  `, `Opct`, `Ckmt` as `spread`, `blur` as `size`, and `GlwT` (typeID `BETE`, `SfBL`→Softer / `PrBL`→Precise). Accept `UnitFloat` or `Double` for numeric keys and apply the D2 defaults. Any parse error, unknown data version, wrong type, a non-`BlnM` blend typeID, a non-`BETE` technique typeID, a non-`RGBC` colour, or a non-finite value (including a finite `f64` that overflows `f32`) returns `None`; a finite out-of-range numeric is clamped (`opacity`/`spread` `0..=100`, `size` `0..=250`); an unknown technique value falls back to Softer; never panic.
- [x] 1.3 Re-export `OuterGlow` (and `GlowTechnique`) from `crates/pictura-render/src/lib.rs`; extend the module `ponytail:` ceiling comment for the deferred `Precise` technique, gradient mode, `Range`, contour, noise, jitter and anti-alias.
- [x] 1.4 Add decoder unit tests in `crates/pictura-render/src/tests/layer_effects.rs` using `pictura_codec::write_descriptor` with a `DescriptorBlock2` header (`1u32`, `16u32`): a full `OrGl` decodes to the expected `OuterGlow` (including `GlwT` `PrBL`); a minimal `OrGl` takes the defaults (Screen, `#FFFFBE`, opacity 75, Softer, spread 0, size 5); a `doub` (non-unit) numeric decodes; missing `lfx2`, missing `OrGl`, wrong data version, wrong-typed `Md  `, wrong `Md  ` typeID, wrong `GlwT` typeID, non-`RGBC` `Clr `, non-finite `blur`, and a truncated block each return `None` without panicking; `blur`/`Ckmt` `1e30` clamp and `1e300` rejects.

## 2. Renderer: outer glow composite

- [x] 2.1 Add `composite_outer_glow(canvas, layer, doc, glow)` to `layer_effects.rs`, reusing `content_matte`, `clip_rect`, `pad_rect`, `dilate_matte`, `blur_matte`, `clamp_finite` and `blend_parts`: build the masked content matte over `S = layer.rect ∩ canvas` padded by `round(spread/100 · size) + Gaussian support`, dilate it, Gaussian-blur it by `size`, multiply by the exterior mask `1 − matte`, tint by `color` and `opacity/100`, and blend into the canvas over `padded ∩ canvas` with the effect's own `blend_mode` and no offset. Early-out on an empty source, an all-zero matte, or an empty output region; never panic.
- [x] 2.2 Extend `composite_layer_effects` in `layer_effects.rs` to decode `decode_outer_glow` and, for an enabled present effect, call `composite_outer_glow`; keep the group and destructive-adjustment skips shared with the shadow.
- [x] 2.3 Add render tests in `tests/layer_effects.rs`: the glow tints the backdrop adjacent to a small opaque square and far pixels are byte-identical to no effect; the opaque content pixel is unchanged (exterior); a positive `spread` covers at least as many pixels as `spread` 0 and `spread` 100 is harder-edged; `size` 6 transitions across more pixels than `size` 0; `opacity` 50 with a red `colour` over white is a red tint; a mask hides the glow where zero and keeps it where 255; a crafted huge `spread`/`size` renders as a bounded no-op without panic; a disabled or not-present glow is byte-identical to no effect; the matte-source cases (solid fill follows payload alpha, pattern fill confined to the fill, channel-less smart object covers the rect) and the edge cases (opacity 0 no-op, off-canvas/zero-area no-op, `Softer` vs `Precise` identical) mirror the shadow suite.

## 3. GPU fallback

- [x] 3.1 Extend `check_supported`'s `walk` in `crates/pictura-render/src/gpu/mod.rs` so a visible layer whose `decode_outer_glow` is enabled and present also returns `GpuError::UnsupportedLayerEffect` before dispatch, ahead of the adjustment check. Keep disabled/absent/malformed glows from rejecting the document, and do not add a new error variant.
- [x] 3.2 Add tests: `composite_gpu` on an outer-glow document returns `UnsupportedLayerEffect` without panicking, `composite_gpu_or_cpu` equals `composite_rgba`, and a disabled or not-present glow does not produce the error. A fill layer with an enabled glow reports `UnsupportedLayerEffect` (the effect check precedes the adjustment check).

## 4. Fixture and oracle

- [x] 4.1 Add an `outer_glow()` builder to `scripts/generate-fixtures.py`: a `Base` pixel layer plus a `Glowing` layer whose record carries `DescriptorBlock2(Descriptor({ masterFXSwitch: Bool(True), OrGl: Descriptor({ enab, present, showInDialog, Md  : Enumerated(b"BlnM", b"scrn"), Clr : Descriptor(RGBC, Rd /Grn /Bl  doubles), Opct: UnitFloat(60.0, Percent), GlwT: Enumerated(b"BETE", b"PrBL"), Ckmt: UnitFloat(20.0, Pixels), blur: UnitFloat(10.0, Pixels), Nose: UnitFloat(0.0, Percent), ShdN: UnitFloat(0.0, Percent), AntA: Bool(True), TrnS: Descriptor(Linear, classID=b"TrnS"), Inpr: UnitFloat(50.0, Percent) }, classID=b"OrGl") }, classID=Klass.Null))` under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. Register `"outer_glow.psd": outer_glow` in `FIXTURES`.
- [x] 4.2 Regenerate with `python3 scripts/generate-fixtures.py`, add `crates/pictura-codec/tests/fixtures/outer_glow.psd`, and confirm the existing fixtures are byte-identical (`git status`).
- [x] 4.3 Register the fixture in the codec `FIXTURES` table and add an oracle test in `crates/pictura-codec/tests/oracle.rs`: the `lfx2` key is present in `extra_blocks` with version 1 / data version 16, the whole `Document` round-trips through `write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check reads the layer's effect as `OuterGlow` asserting `enabled`, `present`, `opacity`, `blend_mode`, `glow_type` and `choke` (spread), not the buggy `spread` property.
- [x] 4.4 Add a render test that loads `outer_glow.psd` with `include_bytes!`, asserts `decode_outer_glow` yields the authored parameters, and composites a glow that differs from the no-effect composite.
- [x] 4.5 Update `crates/pictura-codec/tests/fixtures/README.md`: the contents row and an `outer_glow()` snippet.

## 5. App

- [x] 5.1 Confirm no production app change is needed: the canvas composites through `pictura_render::composite_rgba` / `composite_active`, which now render the glow. Do not add an authoring command, panel, or `CMakeLists.txt` entry. No C++ self-test check is added.

## 6. Verification fixes

- [x] 6.1 Bound every decoded glow numeric to its documented range (`opacity`/`spread` `0..=100`, `size` `0..=250`), re-check `is_finite()` after the `f32` cast, and clamp again in `composite_outer_glow` so a hand-built `OuterGlow` cannot panic the dilate/blur.
- [x] 6.2 Confirm the exterior knock-out (`halo · (1 − matte)`) leaves opaque content unchanged and does not fill the content interior, and that the mask folds into the matte before the knock-out.
- [x] 6.3 Confirm the bbox confinement: a small layer with a large `size`/`spread` leaves far pixels byte-identical and completes within a generous wall-clock bound; a canvas-filling layer remains the documented `O(canvas · size)` ceiling.
- [x] 6.4 Keep `layer_effects.rs` under its file-size cap; if `tests/layer_effects.rs` approaches 1400 LOC, split the glow tests into `src/tests/layer_effects/outer_glow.rs` and declare it in `src/tests/layer_effects.rs` or `src/tests/mod.rs`.

## 7. Gates

- [ ] 7.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [ ] 7.2 `bash scripts/verify-fast.sh` and a headless self-test; record counts.
- [ ] 7.3 `openspec validate layer-effects-outer-glow --strict` and `openspec validate --all --strict`.
- [ ] 7.4 Commit with the new golden fixture; state the fixture addition in the commit message. Update `docs/dev/STATE.md` separately under `TASK-ALLOWS-DOCS` if the milestone anchor is advanced.
