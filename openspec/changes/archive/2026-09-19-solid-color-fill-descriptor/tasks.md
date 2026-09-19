## 1. Renderer: decode the descriptor

- [x] 1.1 Add `decode_solid_fill(d: &[u8]) -> Option<Adjustment>` in `crates/pictura-render/src/composite.rs`: parse with `pictura_codec::read_descriptor`, require the top level to be a `DescValue::Object`, read the `Clr ` item with `desc_item`, require it to be an object, and read `Rd  ` / `Grn ` / `Bl  ` as finite `DescValue::Double`. Round and clamp each to `0..=255` and return `Adjustment::SolidFill([r, g, b, 255])`; any missing key, wrong type, non-finite value, or parse error returns `None`.
- [x] 1.2 Rewrite the `b"SoCo"` match arm: a 4-byte slice keeps the existing `SolidFill([r, g, b, a])` result, otherwise call `decode_solid_fill`. Confirm the other arms and `decode_adjustment`'s no-op contract are unchanged.
- [x] 1.3 Update `decode_adjustment`'s doc comment: list `SoCo` as both the 4-byte in-house form and the standard descriptor, and drop "a real Photoshop `SoCo` descriptor" from the not-decoded sentence.

## 2. Renderer: encode the descriptor

- [x] 2.1 Add `pub fn encode_solid_color_fill(color: [u8; 3]) -> AdjustmentData` in `composite.rs`, building a `DescValue::Object` for `Clr `/`RGBC` with three `DescValue::Double` components on the `0..=255` scale and serializing it with `pictura_codec::write_descriptor` (which emits the version-16 header). Key `*b"SoCo"`.
- [x] 2.2 Re-export `encode_solid_color_fill` from `crates/pictura-render/src/lib.rs` alongside the other encoders.
- [x] 2.3 Add a unit test: `decode_adjustment(&encode_solid_color_fill([10, 20, 30]))` equals `Adjustment::SolidFill([10, 20, 30, 255])`, and `pictura_codec::read_descriptor` on the payload yields a `Clr ` object with the three doubles.

## 3. Renderer: fill-content recognition

- [x] 3.1 Rewrite `is_fill_content_layer` in `crates/pictura-render/src/document_ops/layer_ops/rasterize.rs` to `data.key == *b"SoCo" && matches!(decode_adjustment(data), Some(Adjustment::SolidFill(_)))`, so both payload forms qualify and there is one decoder.
- [x] 3.2 Rewrite `rasterize_fill_content` to obtain the RGBA from that same `decode_adjustment` result and pass it to `bake_solid`; keep the refusal-without-mutation contract.
- [x] 3.3 Update the module and function doc comments to say a descriptor-form `SoCo` is now fill content.

## 4. App authoring

- [x] 4.1 In `crates/pictura-render/src/document_ops/layer_ops/create.rs::add_solid_fill`, build `adjustment` from `encode_solid_color_fill([rgba[0], rgba[1], rgba[2]])` instead of the raw 4-byte vector. Keep the `[u8; 4]` signature; add a `// ponytail: SoCo descriptor is RGB-only, alpha is dropped (app callers pass 0xFF); thread alpha through if a non-opaque fill is ever needed` note and update the docstring.
- [x] 4.2 Add a test that `add_solid_fill` produces a version-16 descriptor payload and the layer still composites to the requested colour at full alpha.

## 5. Renderer: tests

- [x] 5.1 In `crates/pictura-render/src/tests/adjustment.rs`, replace `solid_fill_decodes_only_the_four_byte_payload` with: the 4-byte form still decodes; a descriptor built from `encode_solid_color_fill([10, 20, 30])` decodes to `SolidFill([10, 20, 30, 255])`; a non-4-byte, non-descriptor payload, a descriptor missing `Clr `, and a `Clr ` object with a wrong-typed component each return `None` without panicking.
- [x] 5.2 In `crates/pictura-render/src/tests/rasterize.rs`, change `rasterize_bakes_color_and_clears_fill` to an opaque fill (descriptor alpha is 255) and assert the baked `-1` channel is 255; update `rasterize_refuses_non_fill_targets` so its 6-byte non-descriptor `SoCo` still refuses and add a valid descriptor-form layer that is accepted and rasterizes.
- [x] 5.3 Add a composite test that a descriptor-form solid fill layer (via `encode_solid_color_fill`) composites to its colour, and a render test that loads `crates/pictura-codec/tests/fixtures/solid_fill.psd` with `include_bytes!`, reads it with `pictura_codec::read_psd`, and asserts `decode_adjustment` on the fill layer yields `SolidFill([10, 20, 30, 255])`.

## 6. Fixture and oracle

- [x] 6.1 Add a `solid_fill()` builder to `scripts/generate-fixtures.py` plus a document-sized `Base` pixel layer and a channel-stripped `SoCo` layer authored with `DescriptorBlock(Descriptor({b"Clr ": Descriptor({b"Rd  ": Double(10.0), b"Grn ": Double(20.0), b"Bl  ": Double(30.0)}, classID=b"RGBC")}, classID=b"SoCo"))` under `Tag.SOLID_COLOR_SHEET_SETTING` (import `Descriptor`/`DescriptorBlock`/`Double`), and register `"solid_fill.psd": solid_fill` in `FIXTURES`.
- [x] 6.2 Regenerate with `python3 scripts/generate-fixtures.py` and add `crates/pictura-codec/tests/fixtures/solid_fill.psd`; confirm the existing fixtures are byte-identical.
- [x] 6.3 Add an oracle test in `crates/pictura-codec/tests/oracle.rs`: the fixture has `Base` + the fill layer; the fill layer's `SoCo` payload parses with `pictura_codec::read_descriptor` as a version-16 object whose `Clr ` `RGBC` object carries `Double(10.0)`, `Double(20.0)`, `Double(30.0)`; and the whole `Document` round-trips through `write_psd`/`read_psd`. Register the fixture in the codec `FIXTURES` table.
- [x] 6.4 Update `crates/pictura-codec/tests/fixtures/README.md`: the contents table row and a `solid_fill()` snippet.

## 7. Gates

- [x] 7.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [x] 7.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [x] 7.3 `openspec validate solid-color-fill-descriptor --strict` and `openspec validate --all --strict`.
- [x] 7.4 Commit with the new golden fixture and state the fixture addition in the commit message. No `docs/` change is expected; if the roadmap's G8 entry or the `STATE.md` deferred line is updated, commit it separately with `TASK-ALLOWS-DOCS`.
