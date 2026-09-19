## 1. Dependency and constants

- [x] 1.1 Add `flate2 = "1"` to `crates/pictura-codec/Cargo.toml` (pure-Rust miniz_oxide backend; needed because deflate has no std decoder).
- [x] 1.2 Add `COMPRESSION_ZIP = 2` and `COMPRESSION_ZIP_PREDICTION = 3` to `common.rs`.

## 2. ZIP decode

- [x] 2.1 Add `fn inflate(payload, expected) -> Result<Vec<u8>, PsdError>`: `ZlibDecoder` first, raw `DeflateDecoder` fallback; short output is `Invalid`, long output is truncated; decoder error is `Unsupported`.
- [x] 2.2 Add `fn undo_prediction(data: &mut [u8], row_len: usize)`: byte-wise running-sum inverse per `row_len` chunk.
- [x] 2.3 Composite: `read_psd` matches `0/1/2/3`; for 2/3 inflate the remaining bytes to `channels*width*height` and undo the prediction (row length `width`) for code 3.
- [x] 2.4 Layer channels: `read_channel_data` inflates the `declared_len - 2` payload to `width*height`, undoing the prediction for code 3.
- [x] 2.5 Update the `pictura-codec` crate doc scope note in `lib.rs`.

## 3. Robust open

- [x] 3.1 `read_layer_record`: `BlendMode::from_psd_key(key).unwrap_or(BlendMode::Normal)`; keep the `8BIM` signature error.
- [x] 3.2 Absent composite: before the image-data compression word, when the reader is exhausted and the layer section produced layers, return a document with a zeroed `PixelBuffer` and no document channels; otherwise error as before.

## 4. Tests

- [x] 4.1 Rust unit tests building synthetic PSDs (compress with `flate2`): ZIP composite, ZIP-with-prediction composite, ZIP layer channel, ZIP-with-prediction layer channel — each asserts the decoded bytes.
- [x] 4.2 A prediction test that fails if the inverse is applied per plane instead of per row (and vice versa).
- [x] 4.3 Unknown blend key: a record with a `zzzz` key reads as `BlendMode::Normal` without error.
- [x] 4.4 Absent composite: a layered PSD truncated after the layer section parses with a zero composite; a header with no trailing data still errors.
- [x] 4.5 A malformed deflate stream returns `Unsupported`/`Invalid`, never panics.

## 5. Oracle and verification

- [x] 5.1 In `tests/oracle.rs`, when `psd-tools` is available, build a PSD by hand whose layer channel and composite use ZIP/ZIP-with-prediction, then assert `read_psd` recovers the same bytes `psd-tools` decodes from the same file (psd-tools' writer only emits raw/RLE, so it is the decoder oracle).
- [x] 5.2 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -p pictura-codec`.
- [x] 5.3 `bash scripts/verify-full.sh` and a headless self-test run; record the counts.
- [x] 5.4 Commit the change (roadmap note + `STATE.md` carry `TASK-ALLOWS-DOCS`).
