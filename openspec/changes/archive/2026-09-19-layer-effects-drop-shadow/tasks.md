## 1. Core accessor and effects decoder

- [x] 1.1 Add `Layer::extra_block(&self, key: &[u8; 4]) -> Option<&LayerBlock>` to `crates/pictura-core/src/lib.rs` as a thin find over `extra_blocks`, and a unit test that it finds a present key and returns `None` for an absent one.
- [x] 1.2 Add `crates/pictura-render/src/layer_effects.rs` with `DropShadow { enabled, present, blend_mode, color, opacity, angle_deg, distance, spread, size, use_global_angle, knocks_out }` and `pub fn decode_drop_shadow(layer: &Layer) -> Option<DropShadow>`: take the `lfx2` block via `Layer::extra_block`, require at least `8` bytes, hand `&data[4..]` to `pictura_codec::read_descriptor`, require the top-level object, find `DrSh` (an `Objc` with class id `DrSh`), and decode `enab`, `present`, `Md  ` (blend enum via `BlendMode::from_psd_key`, default Normal), `Clr `/`RGBC` `Rd `/`Grn `/`Bl  ` doubles clamped to `0..=255`, `Opct`, `uglg`, `lagl`, `Dstn`, `Ckmt`, `blur`, `layerConceals`. Accept `UnitFloat` or `Double` for numeric keys and apply the D2 defaults. Any parse error, unknown data version, wrong type, a non-`BlnM` blend typeID, a non-`RGBC` colour, or a non-finite value (including a finite `f64` that overflows `f32`) returns `None`; a finite out-of-range numeric is clamped to its documented range; never panic.
- [x] 1.3 Declare `pub mod layer_effects;` and the `DropShadow` re-export in `crates/pictura-render/src/lib.rs`; add a `ponytail:` ceiling comment for the deferred global-light resource and the unmodeled contour/noise/anti-alias.
- [x] 1.4 Add decoder unit tests using `pictura_codec::write_descriptor` with a `DescriptorBlock2` header (`1u32`, `16u32`): a full `DrSh` decodes to the expected `DropShadow`; a minimal `DrSh` takes the defaults; `uglg` false plus a local angle decodes; a `doub` (non-unit) numeric decodes; missing `lfx2`, missing `DrSh`, wrong data version, wrong-typed `Md  `, and a non-finite `Dstn` each return `None` without panicking.

## 2. Renderer: drop shadow composite

- [x] 2.1 Factor the source-over blend in `composite.rs` into a `blend_parts(canvas, x, y, cs, alpha, mode)` helper and make `blend_into` call it with the layer's alpha already weighted by opacity/fill/mask, so the effect can supply its own blend mode and opacity.
- [x] 2.2 Expose the layer content alpha for the matte: a `layer_content_alpha(layer, doc, x, y) -> f32` that returns the pixel alpha, the fill payload alpha (`SoCo`/`GdFl`/`PtFl`, reusing `fill.rs`), or `1.0` for an embedded smart source; multiply by `mask_alpha(layer, x, y) / 255`. Add `ponytail:` notes for groups/adjustment layers being skipped.
- [x] 2.3 Add `composite_drop_shadow(canvas, layer, doc, shadow)` to `layer_effects.rs`: build the coverage matte over the padded source region `S = layer.rect ∩ canvas` (reached by `spread` and the Gaussian support), dilate it by `round(spread/100 · size)` with a max filter, Gaussian-blur a three-plane `PixelBuffer` matte through `pictura_filters::blur::gaussian(size)`, then offset by `dx = -distance·cos(angle)`, `dy = +distance·sin(angle)` (integer sampling, stated) and `blend_parts` each pixel with the shadow colour and `opacity/100` alpha. Early-out on an empty source or shifted output region, and skip the blur on an all-zero matte; guard zero-area/empty mattes and clip to the canvas; never panic.
- [x] 2.4 Call `composite_drop_shadow` from `composite_layer` before the pixel/fill/smart content for a non-group, non-adjustment layer whose `decode_drop_shadow` yields an enabled, present effect; leave adjustment layers and groups unchanged.
- [x] 2.5 Add render tests in `crates/pictura-render/src/tests/layer_effects.rs` (declared in `src/tests/mod.rs`): angle 0 casts left and 180 casts right; `distance` scales the darkened extent; a larger `size` widens the transition; a red `opacity`-50 shadow over white is a red tint; positive `spread` covers no fewer than zero spread (dilate); a mask hides the shadow where it is zero; the shadow under the content shows the content colour; a disabled/no-effect document is byte-identical to the plain composite.

## 3. GPU fallback

- [x] 3.1 Add `GpuError::UnsupportedLayerEffect` to `crates/pictura-render/src/gpu/mod.rs` with its `Display` arm.
- [x] 3.2 In `check_supported`'s `walk`, reject a visible layer whose `decode_drop_shadow` is enabled and present with `GpuError::UnsupportedLayerEffect` before any dispatch. Keep disabled/absent/malformed effects from rejecting the document.
- [x] 3.3 Add a test in `crates/pictura-render/src/tests/layer_effects.rs` or the existing GPU test module: `composite_gpu` on a drop-shadow document returns `UnsupportedLayerEffect` without panicking, `composite_gpu_or_cpu` equals `composite_rgba`, and a disabled effect does not produce the error.

## 4. Fixture and oracle

- [x] 4.1 Add a `drop_shadow()` builder to `scripts/generate-fixtures.py`: a `Base` pixel layer plus a signed layer whose record carries a `DescriptorBlock2(Descriptor({ masterFXSwitch: Bool(True), DrSh: Descriptor({ enab, present, showInDialog, Md  : Enumerated(b"BlnM", b"mul "), Clr : Descriptor(RGBC, Rd /Grn /Bl  doubles), Opct: UnitFloat(75.0, Percent), uglg: Bool(False), lagl: UnitFloat(120.0, Angle), Dstn: UnitFloat(5.0, Pixels), Ckmt: UnitFloat(0.0, Percent), blur: UnitFloat(5.0, Pixels), TrnS, AntA, Nose, layerConceals }, classID=b"DrSh") }, classID=Klass.Null))` under `Tag.OBJECT_BASED_EFFECTS_LAYER_INFO`. Register `"drop_shadow.psd": drop_shadow` in `FIXTURES`.
- [x] 4.2 Regenerate with `python3 scripts/generate-fixtures.py` and add `crates/pictura-codec/tests/fixtures/drop_shadow.psd`; confirm the existing fixtures are byte-identical.
- [x] 4.3 Register the fixture in the codec `FIXTURES` table and add an oracle test in `crates/pictura-codec/tests/oracle.rs`: the `lfx2` key is present in `extra_blocks`, the whole `Document` round-trips through `write_psd`/`read_psd` with `lfx2` preserved, and a self-skipping psd-tools check reads the layer's effect as `DropShadow` with the authored values.
- [x] 4.4 Add a render test that loads `drop_shadow.psd` with `include_bytes!`, asserts `decode_drop_shadow` yields the authored parameters, and composites a shadow that differs from the no-effect composite.
- [x] 4.5 Update `crates/pictura-codec/tests/fixtures/README.md`: the contents row and a `drop_shadow()` snippet.

## 5. App

- [x] 5.1 Confirm no production app change is needed: the canvas composites through `pictura_render::composite_rgba` / `composite_active`, which now render the shadow. Do not add an authoring command, panel, or `CMakeLists.txt` entry. No C++ self-test check is added (the coverage is Rust; the next free exit code 293 is untouched).

## 6. Verification fixes

- [x] 6.1 Bound every decoded numeric to its documented range (`spread` `0..=100`, `size` `0..=250`, `distance` `0..=30000`, `opacity` `0..=100`), re-check `is_finite()` after the `f32` cast, and reject a finite `f64` that overflows to infinity; clamp again in `composite_drop_shadow` so a hand-built `DropShadow` cannot panic the dilate/blur.
- [x] 6.2 Correct Drop Shadow's `Ckmt` to Spread: rename the field to `spread` and apply a max-filter dilate (`round(spread/100 · size)`) before the blur, not a min-filter erode.
- [x] 6.3 Fall back to `1.0` inside the layer rect when a pixel layer has no `-1` channel, matching the content path's opaque default.
- [x] 6.4 Require the `Md  ` enum typeID `BlnM` and the `Clr ` class `RGBC`, rejecting otherwise.
- [x] 6.5 Reorder `check_supported` so the effect check precedes the adjustment check, and reject only an enabled, present effect.
- [x] 6.6 Author the fixture with `uglg` false so the stored `lagl` is the effective angle, and regenerate.
- [x] 6.7 Add tests: bounded/rejected huge numerics, spread dilate/hard edge, no-`-1` pixel layer, `SoCo` payload alpha, `PtFl` confined coverage, channel-less smart object, and the fill-layer GPU effect error.
- [x] 6.8 Restrict the shadow pipeline to the content bbox: build the matte over `S = layer.rect ∩ canvas` padded by `dilate + Gaussian support`, dilate/blur there, composite only over `(padded + offset) ∩ canvas`, and early-out on an empty source/output region or an all-zero matte.
- [x] 6.9 Document the remaining `O(canvas · size)` ceiling for a canvas-filling layer at the maximum `size`; keep the public size cap at 250.
- [x] 6.10 Add tests: crafted off-canvas shadow is a bounded no-op, a small layer casts only within the reach and leaves far pixels byte-identical, and a golden content-adjacent pixel set.

## 7. Gates

- [ ] 7.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [ ] 7.2 `bash scripts/verify-fast.sh` and a headless self-test; record counts.
- [ ] 7.3 `openspec validate layer-effects-drop-shadow --strict` and `openspec validate --all --strict`.
- [ ] 7.4 Commit with the new golden fixture; state the fixture addition in the commit message. Update `docs/dev/STATE.md` separately under `TASK-ALLOWS-DOCS` if the milestone anchor is advanced.
