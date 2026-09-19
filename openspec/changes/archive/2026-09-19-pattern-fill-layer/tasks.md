## 1. Codec: recognise the block and decode patterns

- [x] 1.1 Add `*b"PtFl"` to `ADJUSTMENT_KEYS` in `crates/pictura-codec/src/common.rs` (21→22) and update its doc comment to name `PtFl` as pattern fill content, so a real pattern fill block is routed to `layer.adjustment` instead of being dropped to `extra_blocks`.
- [x] 1.2 Add `crates/pictura-codec/src/patterns.rs` with `PatternPixels { pattern_id: String, width: u32, height: u32, rgba: Vec<u8> }` and `pub fn decode_patterns(doc: &Document) -> Vec<PatternPixels>`: walk the global `8BIM` tagged blocks in `doc.layer_section_extra` with the same shape as `smart_object::collect_linked_records` (signature, 4-char key, `u32` length / `u64` for PSB big keys, external 4-byte padding), and for each `Patt`/`Pat2`/`Pat3` parse the `Patterns` list (`read_length_block(padding=4)` entries; version 1, image mode, `2 × i16` point, Unicode name, Pascal ASCII id with padding 1, optional indexed table, `VirtualMemoryArrayList` version 3 with `num_channels + 2` channel arrays). Decompress each written channel's `u8` plane with the existing channel decoder; map the first written colour channels to RGB (replicate for Grayscale), and a written alpha-region channel to transparency (255 when absent). Skip malformed blocks keeping what parsed; never panic.
- [x] 1.3 Declare `mod patterns;` and `pub use patterns::{decode_patterns, PatternPixels};` in `crates/pictura-codec/src/lib.rs`. Do NOT touch the image-resource section (`Document.image_resources`); image resource 1039 is the ICC profile, not patterns.
- [x] 1.4 Add unit tests in `patterns.rs` (or `crates/pictura-codec/src/tests.rs`): a synthetic `Patt` block with an RGB pattern decodes to the expected id/size/RGBA; a Grayscale pattern replicates its channel; a truncated block is skipped without panicking.

## 2. Adjustment model (`pictura-adjust`)

- [x] 2.1 Add `PatternFillParams { pattern_id: String, scale: f32, link_with_layer: bool, origin: (i32, i32) }` to `crates/pictura-adjust/src/types.rs` and add the `Adjustment::PatternFill(PatternFillParams)` variant after `GradientFill`.
- [x] 2.2 Dispatch `Adjustment::PatternFill(_) => Err(AdjustError::Unsupported("pattern fill is composited, not applied destructively".into()))` in `crates/pictura-adjust/src/apply.rs`; export `PatternFillParams` from `crates/pictura-adjust/src/lib.rs`.
- [x] 2.3 Add `PatternFill` to the `adjustments` array in `crates/pictura-adjust/src/tests.rs::alpha_is_never_modified` and a unit test that `apply` returns `AdjustError::Unsupported` without mutating the buffer.
- [x] 2.4 Add a `PatternFill` row (no ImageMagick equivalent, tolerance 0) to `crates/pictura-adjust/tests/oracle.rs` and to `NO_EQUIVALENT`; set `MAPPING.len()` to 18 and the no-equivalent count to 15.

## 3. Renderer: decode `PtFl`

- [x] 3.1 Add `decode_pattern_fill(d: &[u8]) -> Option<Adjustment>` in `crates/pictura-render/src/fill.rs`: parse with `pictura_codec::read_descriptor`, require an object, read the `Ptrn` object (class `Ptrn`) and its `Idnt` text (strip trailing NULs; missing/wrong-typed is `None`), optionally read `Nm  ` (ignored), optional `Scl ` as `Double` or `UnitFloat` (default 100, non-finite → `None`), optional `Algn` `Bool` (default true), and optional `phase` `Pnt ` object `Hrzn`/`Vrtc` doubles as `origin` (default `(0, 0)`). Return `Adjustment::PatternFill(PatternFillParams { .. })`. Never panic.
- [x] 3.2 Wire `b"PtFl" => crate::fill::decode_pattern_fill(&data.data)` into `decode_adjustment` in `composite.rs` and add `PtFl` to its doc-comment key list.
- [x] 3.3 Add decoder unit tests: a hand-built `PtFl` descriptor (serialized with `pictura_codec::write_descriptor`) decodes to the expected id/scale/link flag; missing `Scl `/`Algn` apply the defaults; a missing `Ptrn`, a wrong-typed `Idnt`, a non-finite `Scl `, and a truncated payload each return `None` without panicking. Include a test that a `Scl ` written as a unit float decodes.

## 4. Renderer: composite and rasterize pattern fills

- [x] 4.1 Add `composite_pattern_fill(canvas, layer, doc, params)` in `crates/pictura-render/src/fill.rs`: resolve `params.pattern_id` in `pictura_codec::decode_patterns(doc)`; tile the `tw × th` tile (`tw = max(1, round(width × scale / 100))`, nearest-neighbour resample when `scale != 100`) over the layer rect clamped to the canvas; anchor the tile at the layer top-left when `link_with_layer` else the document origin; offset by `origin`; and blend each in-rect sample at the pattern alpha through `blend_into`. When no pattern matches, composite the grey 50 %-grey placeholder over the rect. Add `ponytail:` ceilings for the resample filter, non-RGB modes, 16/32-bit planes, the inferred `phase` origin, the missing warning, and per-layer re-decoding.
- [x] 4.2 Change `composite_adjustment` to take `doc: &Document` and route `Adjustment::PatternFill(params)` to `composite_pattern_fill(canvas, layer, doc, params)` (the same early-return shape as `SolidFill`/`GradientFill`); update the call in `composite_layer`.
- [x] 4.3 Add composite tests in `crates/pictura-render/src/tests/fill.rs` (or `.../tests/adjustment.rs`): a 2×2 pattern tiled at scale 100 over the layer rect produces the expected repeating cells and differs from the backdrop-only composite; adjacent cells differ and a pixel one tile width across repeats; a masked-out layer is a no-op; a missing `pattern_id` yields the placeholder rather than a no-op; `link_with_layer` false anchors at the document origin.
- [x] 4.4 Rewrite `is_fill_content_layer` in `crates/pictura-render/src/document_ops/layer_ops/rasterize.rs` to also match `Adjustment::PatternFill(_)`, and extend `rasterize_fill_content` to bake a decoded pattern (decode the document's patterns once before the mutable layer borrow, then bake the tiled pattern or the placeholder into the layer's `0/1/2/-1` channels), keeping the refusal-without-mutation contract and updating the module/function docs.
- [x] 4.5 Add rasterize tests in `crates/pictura-render/src/tests/rasterize.rs`: a `PtFl` layer is fill content and rasterizes to the tiled pattern with a cleared adjustment; a pattern fill with a missing pattern rasterizes to the placeholder; a non-fill layer still refuses.

## 5. App wiring

- [x] 5.1 Confirm no production app change is needed: `Layer > Rasterize > Fill Content` and the Layers-panel entry call `pictura_render::is_fill_content_layer` / `rasterize_fill_content`, which now accept `PtFl`. Do not add a pattern authoring command. Regression guard: the two functions keep their signatures and the render tests cover the added `PtFl` acceptance; no C++ self-test check was added (would need an untested CMake build).

## 6. Fixture and oracle

- [x] 6.1 Add a `pattern_fill()` builder to `scripts/generate-fixtures.py`: a `Base` pixel layer plus a channel-stripped document-sized layer whose tagged-block key is `PtFl`, and a 2×2 RGB pattern stored in the global `Patt` tagged block (`Tag.PATTERNS1`) with the Photoshop slot layout (three written colour slots plus the two mask slots). The `PtFl` descriptor is `DescriptorBlock(Descriptor({ b"Ptrn": Descriptor({b"Nm  ", b"Idnt"}, classID=b"Ptrn"), b"Scl ": UnitFloat(100.0, Unit.Percent), b"Algn": Bool(True) }, classID=b"PtFl"))` attached under `Tag.PATTERN_FILL_SETTING`; initialise the global tagged blocks with `TaggedBlocks()` before setting `Patt`. Register `"pattern_fill.psd": pattern_fill` in `FIXTURES`.
- [x] 6.2 Regenerate with `python3 scripts/generate-fixtures.py` and add `crates/pictura-codec/tests/fixtures/pattern_fill.psd`; confirm the existing fixtures are byte-identical.
- [x] 6.3 Register the fixture in the codec `FIXTURES` table and add an oracle test in `crates/pictura-codec/tests/oracle.rs`: the `PtFl` key is present, `decode_patterns` returns the expected pattern id/size/pixels, and the whole `Document` round-trips through `write_psd`/`read_psd` with the `Patt` block preserved.
- [x] 6.4 Add a render test that loads `pattern_fill.psd` with `include_bytes!` and asserts `decode_adjustment` yields `Adjustment::PatternFill` (id, scale 100, link_with_layer true) and that `composite_rgba` produces the exact 2×2 tiling; where practical, compare the tiling against psd-tools' `draw_pattern_fill` output at scale 100 and self-skip when psd-tools is absent.
- [x] 6.5 Update `crates/pictura-codec/tests/fixtures/README.md`: the contents table row and a `pattern_fill()` snippet.

## 8. Hardening (independent-verify follow-up)

- [x] 8.1 Guard `patterns.rs` against a crafted pattern rectangle: `checked_mul` for `n * 4`, an 8-bit-only gate on each written channel's `depth`/`pixel_depth`, a per-channel rectangle that must match the pattern-level rectangle, and a bounded pixel cap; a malformed pattern is skipped, never panics or allocates without bound.
- [x] 8.2 Require exactly three written colour planes for RGB and one for grayscale (skip a pattern with a hole); take the trailing written alpha-region slot, matching psd-tools' `pixels[:, :, -1:]`.
- [x] 8.3 Route `read_channel_data` through `decode_channel_data` (now `is_psb`-aware) so the shared decoder's comment is accurate and the arms are not duplicated.
- [x] 8.4 Add `pattern_fill_16bit.psd` (authored with psd-tools `set_data(..., depth=16)`) and a codec oracle asserting the `PtFl` block is recognised but the 16-bit pattern is skipped and the document round-trips.
- [x] 8.5 Tests: crafted oversized pattern rect no-panic/no-pattern; `pixel_depth = 16` skipped; both trailing alpha slots picks the last; RGB with an absent colour plane skipped; `link_with_layer` true with a non-zero layer origin anchors to the layer rect; a non-zero `origin` shifts the tile.
- [x] 8.6 Update design D5/D9 and the spec deltas: non-8-bit and malformed patterns are skipped and render the placeholder; soften the "documented placeholder" wording (docs say placeholder-or-last-known and warn; we have no warning).

## 9. Hardening (second-verify follow-up)

- [x] 9.1 Bound `read::inflate` to `expected + 1` bytes through `Read::take`, so a crafted ZIP/Deflate channel (patterns and layer channels alike) cannot expand before it is checked; output shorter or longer than `expected` is `PsdError::Invalid`, matching psd-tools' length-mismatch error. Valid exact-length streams are unchanged.
- [x] 9.2 Raise `MAX_PATTERN_PIXELS` from `1 << 20` to `1 << 24` (4096x4096) so realistic patterns (1025x1025, 2000x2000) decode instead of rendering the placeholder; `checked_mul` and the rectangle-consistency guard stay.
- [x] 9.3 Tests: `read::tests::zip_bomb_is_rejected_with_bounded_allocation` (256 MiB expansion rejected, peak RSS ~16 MiB), `read::tests::zip_exact_is_ok_and_both_mismatches_error`, `patterns::tests::pattern_pixel_cap_boundary` (one px under decodes, one px over skipped); a wrong-class `Ptrn` reject case in the render decoder test; a note on the oversized-rect test.
- [x] 9.4 Document the cap and the bounded decode in comments; keep the spec's "SHALL NOT allocate without bound".

## 10. Hardening (third-verify follow-up)

- [x] 10.1 In `patterns.rs`, resolve the expected colour-plane count from the pattern's `image_mode` (RGB → 3, Grayscale → 1; other modes skipped) before reading channels, reject the pattern unless the declared `num_channels` equals it, and decode only `expected_color + 2` slots (≤5). Removes `MAX_PATTERN_CHANNELS`; the `inflate` take-cap, `checked_mul`, and rect-consistency guards stay.
- [x] 10.2 Regression test `patterns::tests::oversized_channel_count_is_rejected_before_decoding`: `num_channels = 64` for RGB over a full-cap rect with a ZIP bomb channel is rejected before any plane is decoded (peak process RSS ~16 MiB, down from the measured ~2 GiB), and a well-formed pattern still decodes.
- [x] 10.3 Reword `read.rs` `inflate`'s doc comment: over-long output is a typed `Invalid`, a deliberate divergence from psd-tools (which substitutes a black channel with a warning), without claiming parity.
- [x] 10.4 Reconcile the adjust coverage with the 19 `Adjustment` variants: add `SolidFill` to the alpha-preservation refusal set and `apply_refuses_solid_fill_without_mutating`; keep the oracle table at 18 classified rows (16 destructive + `GradientFill` + `PatternFill`) with a comment that `SolidFill` has no `apply`/row; update the spec delta's alpha/validation enumeration to include `SolidFill`.

## 7. Gates

- [ ] 7.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [ ] 7.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [ ] 7.3 `openspec validate pattern-fill-layer --strict` and `openspec validate --all --strict`.
- [ ] 7.4 Commit with the new golden fixture and state the fixture addition and the `PtFl` pattern-location correction in the commit message. No `docs/` change is expected; if the roadmap's P3 entry or `STATE.md` is updated, commit it separately with `TASK-ALLOWS-DOCS`.
