## 1. M12-A1 — `document-resize` and tests

- [ ] 1.1 Add `pictura-ops` as a dependency of `crates/pictura-render` and add `crates/pictura-render/src/document.rs` with the `resize_document(doc: &mut Document, width: u32, height: u32, resample: Resample) -> Result<(), OpsError>` signature, re-exported from `src/lib.rs`
- [ ] 1.2 Implement the recursive layer walk: resample every pixel layer's color and `-1` alpha channels through `pictura_ops::resize`, resample a decoded `LayerMask`'s `data` when present, recurse through group `children`, and resample a group's own mask; an adjustment layer has no color channels and only its mask is resampled
- [ ] 1.3 Scale every layer and mask `rect` by `width / old_width` and `height / old_height` with round-to-nearest edges, preserving all layer metadata (`name`, `blend`, `opacity`, `clipping`, `visible`, `adjustment`)
- [ ] 1.4 Resample every `doc.channels` entry through `pictura_ops::resize` to the new document size, preserving each channel `id`
- [ ] 1.5 Set `doc.width`/`doc.height` and recompute `doc.composite = composite_rgba(doc)` as the final step
- [ ] 1.6 Validate before mutating: `width`/`height` below 1 and any pixel-layer channel or mask whose `data.len()` disagrees with its `rect` return `OpsError::InvalidParams` with `doc` bit-identical, and never panic on 1×1, empty, or no-layer documents
- [ ] 1.7 Unit-test the document dimensions, the untouched-on-error contract, the malformed-channel rejection, and panic-freedom on tiny/empty documents
- [ ] 1.8 Unit-test the recursion: a pixel layer's channels and rect scale, an adjustment layer keeps empty channels and resamples only its mask, a nested group child is resampled, and all layer metadata is preserved
- [ ] 1.9 Unit-test document-level channel resampling and that `doc.composite == composite_rgba(&doc)` after a successful resize

## 2. M12-A2 — `document-canvas` and tests

- [ ] 2.1 Implement `resize_canvas_document(doc: &mut Document, width: u32, height: u32, anchor: Anchor, background: [u8; 4]) -> Result<(), OpsError>` in `src/document.rs`, re-exported from `src/lib.rs`
- [ ] 2.2 Compute the `dx`/`dy` offset from the size deltas and `anchor` with the same nine-anchor math as `pictura_ops::resize_canvas`, and translate every layer and decoded mask `rect` by `(dx, dy)` while recursing through group `children`; never resample or reorder layer/mask channel data
- [ ] 2.3 Re-extend or crop every `doc.channels` entry through `pictura_ops::resize_canvas` with the same `anchor` and a zero fill, preserving each `id`
- [ ] 2.4 Set `doc.width`/`doc.height` and recompute `doc.composite = composite_rgba(doc)`; keep the added canvas area transparent regardless of `background`
- [ ] 2.5 Validate before mutating: `width`/`height` below 1 and any malformed pixel-layer channel or mask return `OpsError::InvalidParams` with `doc` bit-identical, and never panic on 1×1 or empty documents
- [ ] 2.6 Unit-test the anchor offsets and rect translation for the center, top-left, and bottom-right anchors, including a nested masked layer inside a group
- [ ] 2.7 Unit-test document-channel grow/crop with zero-filled added samples, that a non-zero `background` leaves the added composite transparent, and that `doc.composite == composite_rgba(&doc)` after a canvas resize

## 3. M12-A3 — `document-orientation` and tests

- [ ] 3.1 Implement `rotate_document(doc: &mut Document, quarter_turns: u32) -> Result<(), OpsError>` (1 = CW, 2 = 180, 3 = CCW) and `flip_document(doc: &mut Document, horizontal: bool) -> Result<(), OpsError>` in `src/document.rs`, re-exported from `src/lib.rs`
- [ ] 3.2 Apply the matching M10 exact index remap (`rotate90_cw`, `rotate90_ccw`, `rotate180`, `flip_horizontal`, `flip_vertical`) to every layer channel, every decoded mask `data`, and every `doc.channels` entry, recursing through group `children`; an adjustment layer contributes no color channels and only its mask is remapped
- [ ] 3.3 Remap every layer and mask `rect` with the half-open rectangle matching the pixel remap, swap `doc.width`/`doc.height` for 90°/270° turns, and preserve them for 180° and flips
- [ ] 3.4 Recompute `doc.composite = composite_rgba(doc)` after each op, and reject `quarter_turns` outside 1..=3 with `OpsError::InvalidParams` before mutating the document
- [ ] 3.5 Unit-test the entry-point validation: quarter turns 1/2/3 succeed, 0 and ≥4 return `InvalidParams` with the document untouched, and 1×1/empty documents do not panic
- [ ] 3.6 Unit-test the exact remaps: 90° CW/CCW swap dimensions and remap samples, nested layers and masks are remapped, and document-level channels are remapped
- [ ] 3.7 Unit-test the identities and the recomputed composite: four quarter turns, two 180° turns, CW then CCW, and a doubled flip are bit-identical, and `doc.composite == composite_rgba(&doc)` after every op

## 4. M12-B — psd-tools structural oracle and composite verification

- [ ] 4.1 Add `pictura-codec` (and keep `pictura-testkit`) as dev-dependencies of `crates/pictura-render` and add `crates/pictura-render/tests/document_oracle.rs`
- [ ] 4.2 Build a layered fixture (pixel layers, a nested group, a decoded mask, an adjustment layer, and a document-level channel) and, for each document op, write the result with `pictura_codec::write_psd`, re-read it with `pictura_codec::read_psd`, and assert the structural round-trip (dimensions, layer rects, channel lengths, channel ids)
- [ ] 4.3 Open each written PSD with the independent `psd-tools` library and assert the reported document dimensions and layer bounds/sizes match the transformed document
- [ ] 4.4 Assert `doc.composite == composite_rgba(&doc)` after every document op in the oracle
- [ ] 4.5 Skip the psd-tools differential with a message when `psd-tools` is not importable, and add no `#[ignore]`; record the oracle mapping and the psd-tools version in the test header or `crates/pictura-render/tests/README.md`

## 5. M12-C — OpenSpec change, reconcile, and verify

- [ ] 5.1 Validate `openspec validate m12-document-ops --strict` and `openspec validate --all --strict` green
- [ ] 5.2 Run `cargo test --workspace` green with the per-op unit tests, the recursion tests, and the psd-tools oracle
- [ ] 5.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 5.4 Run `scripts/guard.sh` green (no `.8bf`, no artboards, no unmarked `docs/` edits)
- [ ] 5.5 Reconcile the shipped `pictura-render` document signatures with `docs/04-image-ops/image-size.md` (`IMG-001`), `canvas-size.md` (`IMG-002`), and `image-rotation-and-flip.md` (`IMG-003`) and record any divergence (there is no `docs/dev/m12-document-ops.md` to reconcile)
