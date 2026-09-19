## 1. Container refactor

- [x] 1.1 In `crates/pictura-codec/src/write.rs`, split `write_psd` into a thin dispatcher and `fn write_container(doc: &Document, psb: bool) -> Result<Vec<u8>, PsdError>`; move the existing body into `write_container` unchanged except for the version and width branches.
- [x] 1.2 In `write_container`, write `VERSION_PSD` when `psb` is false and `VERSION_PSB` (`2`) when true; keep the header height/width as `u32`.
- [x] 1.3 Select the dimension limit with `if psb { MAX_DIM_PSB } else { MAX_DIM_PSD }` (mirroring `read.rs:31`) and return `PsdError::Unsupported` above it.
- [x] 1.4 Write the layer-and-mask section length and the layer-info length as `u64` under PSB and `u32` under PSD; keep the global layer-mask info length `u32` in both.
- [x] 1.5 Add `pub fn write_psb(doc: &Document) -> Result<Vec<u8>, PsdError>` delegating to `write_container(doc, true)`, and add `pub fn write_psd` that auto-selects `psb = doc.width > MAX_DIM_PSD || doc.height > MAX_DIM_PSD`.

## 2. Width threading

- [x] 2.1 Thread `psb: bool` through `write_layer_info`, `write_record`, and `rle_channel`.
- [x] 2.2 Change `OutChannel::declared_len` to `u64` and have `write_record` emit it as `u64` under PSB and `u32` under PSD; `Verbatim` bytes stay byte-for-byte.
- [x] 2.3 Add a `psb` parameter to `encode_scanlines`; emit `u32` count entries under PSB and `u16` under PSD, and guard each packed row against `u32::MAX` under PSB and `u16::MAX` under PSD. Update the existing `ponytail:` comment to cover both versions.
- [x] 2.4 Export `write_psb` from `crates/pictura-codec/src/lib.rs` alongside `write_psd`.

## 3. Tests

- [x] 3.1 In `crates/pictura-codec/src/tests/psb.rs`, add `write_psb` round-trip for a small document: header version word is `2`, `read_psd` equals the input, and section/info/channel lengths are 8-byte fields.
- [x] 3.2 Add an auto-selection test: `write_psd` of a 30 001×1 document emits version word `2` and round-trips equal.
- [x] 3.3 Add a preserved-verbatim test: a layer with a `Layer.raw_channels` stream round-trips byte-for-byte under PSB and only its declared length widens to `u64`.
- [x] 3.4 Add an over-limit error test: a 300 001-px dimension returns `PsdError::Unsupported`.
- [x] 3.5 Confirm the in-limit version-1 output is unchanged: `default_document_matches_rle_golden` and the existing randomized/layer/mask round-trips pass with no fixture refresh.

## 4. Oracle

- [x] 4.1 In `crates/pictura-codec/tests/oracle.rs`, add a self-skipping psd-tools test that writes a PSB with `write_psb`, opens it with `PSDImage.open`, and asserts the decoded composite pixels equal the input (pattern per `psd_tools_reads_rle_composite_and_layer`, `oracle.rs:383-471`).

## 5. App verification

- [x] 5.1 Confirm (no code change required) that `PictureView::save` (`crates/pictura-app/src/cxxqt_object/impl_core.rs:196`) routes through `write_psd`, so a >30 000 document now saves as a PSB, and that both Save As dialogs already offer `*.psd *.psb`. Record the `.psd` default-suffix and the 30 000 import-budget cap in `probe.rs` as follow-ups, not part of this change.

## 6. Gates

- [x] 6.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`.
- [ ] 6.2 `bash scripts/verify-full.sh` and `./build/pictura --headless --self-test`; record counts.
- [x] 6.3 `openspec validate psb-write --strict` and `openspec validate --all --strict`.

## 7. Docs and state

- [ ] 7.1 Update `docs/dev/psd-support-roadmap.md`: move G9/P5 to shipped (PSB write; dimensions to 300 000).
- [ ] 7.2 Update `docs/dev/STATE.md` with the shipped change and the `write_psb` entry point.
- [ ] 7.3 Commit the docs change separately with `TASK-ALLOWS-DOCS`.

## 8. PSB tagged-block big-key correction

- [x] 8.1 Add `pub is_psb: bool` to `Document` (false in `Document::new`), set on both `read_psd` returns; treat it as derived-`PartialEq` and set it explicitly in written-then-read PSB tests.
- [x] 8.2 Add `common::is_psb_big_key` backed by the exact psd-tools `TaggedBlock._BIG_KEYS` constant.
- [x] 8.3 Make `write_tag(out, key, data, psb)` emit an 8-byte length for PSB big keys; thread `psb` through `write_extra`/`write_record`/`write_layer_info`.
- [x] 8.4 Select the PSB container when `doc.is_psb` or a dimension exceeds 30 000 (removing the small-PSB-re-saves-as-PSD ceiling); `write_psb` still forces PSB.
- [x] 8.5 Frame the authored `lnk2` with `author_lnk2_bytes(..., psb)` and `write_tag(..., psb)`.
- [x] 8.6 Read the tagged-block length as `u64` for big keys in `read_layer_record`, and pass `is_psb` into `resolve_smart_objects`/`collect_linked_records`/`remove_linked_source` (mirrored read).
- [x] 8.7 Pass `doc.is_psb` from the `pictura-render` `remove_linked_source` callers; update `smart_object.rs` unit tests to `false` for PSD cases.
- [x] 8.8 Revise the `psd-codec` spec (tagged-block big-key rule, version rule, two new scenarios) and `design.md` (D6, field-width table, `Document.is_psb`, risks).
- [x] 8.9 Add unit tests (preserved big-key `u64` length; PSB source re-saves as PSB) and the oracle CRITICAL repro (psd-tools parses an authored-smart-object PSB); keep the golden and PSD tests unchanged.

## 9. PSB tagged-block framing correction

- [x] 9.1 `write_tag` declares an even length with the pad byte inside it (never an external pad); reader's odd-length skip fallback kept for legacy files.
- [x] 9.2 Add `reframe_document_extra(bytes, src_psb, dst_psb)` and call it in `write_container` so a preserved document-level big-key block is re-framed to the target container's width; same-container writes return bytes unchanged.
- [x] 9.3 Spec: drop `lnkD` from the big-key list; state the even/pad-inside rule and the re-framing rule; add odd-block and PSD→PSB scenarios.
- [x] 9.4 Design: update D2/D3/D5/D6 and the field-width table, add the re-framing helper and pad-inside rule, fix stale `read.rs`/`write.rs` anchors.
- [x] 9.5 Tests: update the two odd-payload round-trip tests; add direct RLE-count-width, determinism, non-big-key `u32`, `write_tag` pad-inside, and PSD→PSB reframe unit tests; add odd-block-then-`SoLd` and PSD→PSB psd-tools oracle tests.

## 10. Per-layer vs document-level tagged-block framing

- [x] 10.1 Split the writers: `write_tag` (per-layer, even length with pad inside) and `write_tag_document` (document-level, exact length, external pad to 4).
- [x] 10.2 Use `write_tag_document` at document-level sites: `author_lnk2_bytes` and `remove_linked_source` re-emission; leave `write_extra` on `write_tag`.
- [x] 10.3 `reframe_document_extra` reads exact length + skips external 4-pad (source width) and re-emits with `write_tag_document`; unchanged when `src_psb == dst_psb`.
- [x] 10.4 `collect_linked_records` / `remove_linked_source` skip external 4-pad (not the per-layer odd skip) for document-level blocks.
- [x] 10.5 Write `iOpa` as 4 bytes `[fill, 0, 0, 0]`; reader already uses `data[0]`.
- [x] 10.6 Proposal: rewrite the stale small-PSB bullet to match D6 (container preserved via `Document.is_psb`); add the two framing rules.
- [x] 10.7 Tests: `write_tag_document` external-pad unit test, `iOpa` 4-byte unit test, and a self-skipping psd-tools doc-level non-4-multiple-block regression; extend the per-layer odd-block oracle to a non-4-multiple block.
- [x] 10.8 Spec/design: distinguish per-layer and document-level framing rules in the `PSB write` requirement and D2/D5/D6 plus the field-width table; add the document-level scenario.
