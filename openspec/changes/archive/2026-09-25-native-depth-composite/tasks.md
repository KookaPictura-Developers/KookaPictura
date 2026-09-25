# Tasks: native-depth-composite

## 1. Native adjustment compositing

- [x] 1.1 In `crates/pictura-render/src/composite.rs` `composite_adjustment`, when `doc.source_depth.is_some()`, build a planar RGB `Samples::F32` from `canvas.px`, call `pictura_adjust::apply_native(adjustment, &mut samples, w, h, 3)`, and on success write the three color planes back to `canvas.px` (alpha untouched) before the existing `blend_into` gate loop.
- [x] 1.2 On `AdjustError::Unsupported` fall through to the existing 8-bit path; on other `AdjustError` variants keep the current no-op `return`. For a document with no `source_depth`, take the existing 8-bit path unchanged.
- [x] 1.3 Do not change the fill early-returns or the 8-bit path.

## 2. Native composite output

- [x] 2.1 Add `pub fn composite_native(doc: &Document) -> Option<Samples>`: `None` when `doc.source_depth` is `None`, else assemble the canvas exactly as `composite_rgba` and emit `Samples::F32` (source depth 32) or `Samples::U16` (16) in planar RGBA with each value `round(v.clamp(0.0,1.0) * (65535 or 1.0))`.
- [x] 2.2 `crates/pictura-render/src/lib.rs`: export `composite_native`.
- [x] 2.3 Leave `composite_rgba` and `Canvas::into_pixel_buffer` unchanged.

## 3. Verification

- [x] 3.1 Render test: build a depth-16 document (source_depth = Sixteen) with a pixel layer plus an adjustment layer whose application produces a non-8-bit code; assert `composite_native` is `Some(U16)` and some sample is not `u8 * 257` where `u8` is the corresponding `composite_rgba` byte.
- [x] 3.2 Render test: `composite_native` is `None` for an 8-bit/constructed document and `Some(F32)` when `source_depth` is 32.
- [x] 3.3 The existing composite/image/golden tests and the `document_oracle` still pass unchanged.
- [x] 3.4 `cargo nextest run --workspace`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/check-file-size.sh`, `openspec validate native-depth-composite --strict`.
