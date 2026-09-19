## 1. Codec: recognise the block

- [x] 1.1 Add `*b"GdFl"` to `ADJUSTMENT_KEYS` in `crates/pictura-codec/src/common.rs` (19→20) and update its doc comment to name `GdFl` as gradient fill content, so a real gradient fill block is routed to `layer.adjustment` instead of being dropped to `extra_blocks`.

## 2. Adjustment model (`pictura-adjust`)

- [x] 2.1 Add `GradientKind { Linear, Radial, Angle, Reflected, Diamond }` and `GradientFillParams { stops: Vec<GradientStop>, reverse: bool, kind: GradientKind, angle_deg: f32, scale: f32 }` to `crates/pictura-adjust/src/types.rs`, reusing `GradientStop`, and add the `Adjustment::GradientFill(GradientFillParams)` variant after `GradientMap`.
- [x] 2.2 Dispatch `Adjustment::GradientFill(_) => Err(AdjustError::Unsupported("gradient fill is composited, not applied destructively".into()))` in `crates/pictura-adjust/src/apply.rs`; export `GradientKind`/`GradientFillParams` from `crates/pictura-adjust/src/lib.rs`.
- [x] 2.3 Add `GradientFill` to the `adjustments` array in `crates/pictura-adjust/src/tests.rs::alpha_is_never_modified` (assert `apply` errors and the buffer, including alpha, is unchanged) and add a unit test that `apply` returns `AdjustError::Unsupported` without mutating the buffer.
- [x] 2.4 Add a `GradientFill` row (no ImageMagick equivalent, tolerance 0) to `crates/pictura-adjust/tests/oracle.rs` and to `NO_EQUIVALENT`; set `MAPPING.len()` to 17 and the no-equivalent count to 14.

## 3. Renderer: decode `GdFl`

- [x] 3.1 Add `decode_gradient_fill(d: &[u8]) -> Option<Adjustment>` in `crates/pictura-render/src/composite.rs`: parse with `pictura_codec::read_descriptor`, require an object, read `Angl` as a finite `Double`, map the `Type` enum (`GrdT`) to `GradientKind` (`Lnr `/`Rdl `/`Angl`/`Rflc`/`Dmnd`), require `Grad` to be an object, require its `GrdF` enum (`GrdF`) to be `CstS`, read its `Clrs` list of at least two stop objects (`Clr ` `RGBC` `Rd  `/`Grn `/`Bl  ` finite `Double`s rounded/clamped to `0..=255`; `Lctn` `Double` or `Long` in `0..=4096`, strictly increasing), and read optional `Rvrs` (bool, default false) and `Scl ` (`Double`, default 100). Reject a colour-noise `ClNs`, missing/wrong-typed keys, non-finite values, and non-monotone/out-of-range stops with `None`.
- [x] 3.2 Wire `b"GdFl" => decode_gradient_fill(&data.data)` into `decode_adjustment` and add `GdFl` to its doc-comment key list.
- [x] 3.3 Add decoder unit tests: a hand-built `GdFl` (or `encode_gradient_fill` output) decodes to the expected `GradientKind`/angle/stops for each of the five kinds; a `ClNs` gradient, a missing `Grad`/`Clrs`, a single stop, and non-increasing/out-of-range locations each return `None` without panicking.

## 4. Renderer: encode and composite

- [x] 4.1 Add `pub fn encode_gradient_fill(kind: GradientKind, stops: &[GradientStop], angle_deg: f32) -> AdjustmentData` building the version-16 `GdFl` object (`Angl` `Double`, `Type` `Enum(b"GrdT", kind)`, `Grad` `Objc` class `Grdn` with `Nm  ` text, `GrdF` `Enum(b"GrdF", b"CstS")`, `Intr` `Enum(b"Intp", b"Lnr ")`, and a `Clrs` `VlLs` of `RGBC` stop objects with `Clr `/`Typ `/`Lctn`/`Mdpn`), serialized with `pictura_codec::write_descriptor`.
- [x] 4.2 Re-export `encode_gradient_fill` (and `GradientKind`/`GradientFillParams`) from `crates/pictura-render/src/lib.rs`.
- [x] 4.3 Add a formatter-equivalent unit test: `decode_adjustment(&encode_gradient_fill(GradientKind::Linear, &[black@0, white@4096], 0.0))` equals `Adjustment::GradientFill` with those stops, kind Linear, angle 0.0, `reverse` false, `scale` 100; and that `read_descriptor` sees the `Type`/`Grad`/`Clrs` shape.
- [x] 4.4 Add `composite_gradient_fill(canvas, layer, params)` in `composite.rs` following psd-tools' `draw_gradient_fill`: compute the scale-weighted `X`/`Y`, the per-kind `Z` (linear/radial/angle/reflected/diamond), clamp and reverse `Z`, sample the stops at `Z*4096` (linear, clamped), and `blend_into` each in-rect pixel with alpha 255. Route `Adjustment::GradientFill` to it from `composite_adjustment` (same early-return shape as `SolidFill`). Add `ponytail:` ceilings for noise gradients, transparency stops, midpoint, non-linear interpolation, and non-RGB.
- [x] 4.5 Add a composite test that a black-to-white Linear gradient fill layer over a backdrop produces the expected left-to-right ramp and differs from the backdrop-only composite, plus a reverse case, and a masked-out no-op case.

## 5. Renderer: fill content

- [x] 5.1 Rewrite `is_fill_content_layer` in `crates/pictura-render/src/document_ops/layer_ops/rasterize.rs` to `matches!(crate::decode_adjustment(data), Some(Adjustment::SolidFill(_) | Adjustment::GradientFill(_)))`, so both fill keys qualify through the one decoder.
- [x] 5.2 Extend `rasterize_fill_content` to bake a decoded gradient by generating the five-kind ramp over the layer rect (reuse the composite rasterizer or a shared helper), with opaque alpha; keep the refusal-without-mutation contract and update the module/function docs.
- [x] 5.3 Add `pub fn add_gradient_fill(doc, selection_path) -> String` in `crates/pictura-render/src/document_ops/layer_ops/create.rs`, inserting a document-sized layer named `"Gradient Fill N"` with `adjustment = Some(encode_gradient_fill(GradientKind::Linear, &[black@0, white@4096], 0.0))`, mirroring `add_solid_fill`; re-export it.
- [x] 5.4 Add rasterize tests in `crates/pictura-render/src/tests/rasterize.rs`: a `GdFl` layer is fill content and rasterizes to a non-uniform ramp with a cleared adjustment; a non-fill/colour-noise layer still refuses.

## 6. App wiring

- [x] 6.1 Declare `add_gradient_fill(self: Pin<&mut Self>) -> QString` in `crates/pictura-app/src/cxxqt_object.rs` and implement it in `impl_layers_rasterize.rs`: call `pictura_render::add_gradient_fill`, then `recomposite` and `record("Gradient Fill")`, returning the new path or empty without a document.
- [x] 6.2 Add `LayerNewFillGradient` to `crates/pictura-app/cpp/commands.h`, register `Layer > New Fill Layer > Gradient…` as an enabled command in `command_tree.cpp`, and add the handler + enabled provider in `frame_menus.cpp` calling `view->add_gradient_fill()`.
- [x] 6.3 Add a `Gradient…` action to the Layers-panel fill menu in `crates/pictura-app/cpp/panels/layers_panel.cpp` that calls `view_->add_gradient_fill()`.
- [x] 6.4 Add one C++ self-test check (next free code **286**) in `crates/pictura-app/cpp/selftest_layers_controls.cpp`: adding a gradient fill layer succeeds, `layer_is_fill_content` reports true, and the composite changes; keep the file inside its `scripts/file-size-allowlist.txt` ceiling.

## 7. Fixture and oracle

- [x] 7.1 Add a `gradient_fill()` builder to `scripts/generate-fixtures.py`: a document-sized `Base` pixel layer plus a channel-stripped document-sized `GdFl` layer authored with the `DescriptorBlock(Descriptor({b"Angl": ..., b"Type": Enumerated(Type.GradientType, Enum.Linear), b"Grad": Descriptor({... b"GrdF": Enumerated(Type.GradientForm, Enum.CustomStops), b"Clrs": List([RGBC stop @0 black, RGBC stop @4096 white])}, classID=b"Grdn")}, classID=b"GdFl"))` recipe, attached under `Tag.GRADIENT_FILL_SETTING`; keep the fill layer's rect document-sized (not collapsed to zero) so psd-tools composites the ramp. Register `"gradient_fill.psd": gradient_fill` in `FIXTURES`.
- [x] 7.2 Regenerate with `python3 scripts/generate-fixtures.py` and add `crates/pictura-codec/tests/fixtures/gradient_fill.psd`; confirm the existing fixtures are byte-identical.
- [x] 7.3 Register the fixture in the codec `FIXTURES` table and add an oracle test in `crates/pictura-codec/tests/oracle.rs`: the `GdFl` key is present, the payload parses as a version-16 object whose `Type` enum is `Lnr `, whose `Angl` is `0.0`, and whose `Clrs` holds the two `RGBC` stops, and the whole `Document` round-trips through `write_psd`/`read_psd`.
- [x] 7.4 Add a render test that loads `gradient_fill.psd` with `include_bytes!` and asserts `decode_adjustment` yields `Adjustment::GradientFill` with kind Linear, angle 0, and the black/white stops; where practical, assert the rasterized 8-pixel row matches psd-tools' `composite()` ramp `0,36,72,109,145,182,218,255`.
- [x] 7.5 Update `crates/pictura-codec/tests/fixtures/README.md`: the contents table row and a `gradient_fill()` snippet.

## 8. Gates

- [x] 8.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [x] 8.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [x] 8.3 `openspec validate gradient-fill-layer --strict` and `openspec validate --all --strict`.
- [x] 8.4 Commit with the new golden fixture and state the fixture addition in the commit message. No `docs/` change is expected; if the roadmap's P3 entry or `STATE.md` is updated, commit it separately with `TASK-ALLOWS-DOCS`.
