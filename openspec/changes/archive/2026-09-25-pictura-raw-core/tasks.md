# Tasks: pictura-raw-core

## 1. Settings alias

- [x] 1.1 Add `pub type PicturaRawSettings = CrsSettings` beside `CrsSettings` and re-export it from `pictura-core`.

## 2. Codec

- [x] 2.1 Add `crates/pictura-codec/src/pictura_raw.rs` with the grounded `Fltr` key map, `decode_pictura_raw_settings`, and `encode_pictura_raw_fltr`; register and re-export from `lib.rs`.
- [x] 2.2 Add `attach_pictura_raw_filter`: edit an existing filter, insert into a preserved `SoLd`/`SoLE`, or record on a converted object.
- [x] 2.3 Make `smart_writer::author_sold_block` emit `filterFX` from `SmartObject.smart_filters`.
- [x] 2.4 Unit tests: encode/decode round-trip, tolerant decode, attach+write+read, insert-into-`SoLd`, error without a smart object, and the fixture's 11-key decode.
- [x] 2.5 psd-tools oracle: a written PSD's authored camera-raw filter and `Ex12`/`Temp`/`Cl12` decode with psd-tools.

## 3. Renderer

- [x] 3.1 Add `crates/pictura-adjust/src/pictura_raw.rs` with `render_pictura_raw` and a private separable box blur; re-export from `lib.rs`.
- [x] 3.2 Unit tests: identity no-op, exposure monotonicity + alpha, temperature monotonicity, clarity local contrast, vibrance/determinism, and malformed/empty settings.

## 4. Engine op

- [x] 4.1 Add `crates/pictura-render/src/document_ops/pictura_raw.rs` with `apply_pictura_raw`; register and re-export.
- [x] 4.2 Tests: bake + settings round-trip on a converted smart object, and refusal without mutation for a non-smart layer/missing path.

## 5. Docs and OpenSpec

- [x] 5.1 Record the change and its ceilings in `docs/dev/STATE.md` and `docs/dev/psd-support-roadmap.md`.
- [x] 5.2 Add the `pictura-raw` capability spec, proposal, and design under `openspec/changes/pictura-raw-core/`.

## 6. Verification

- [x] 6.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 6.2 `cargo nextest run --workspace`; `cargo test --workspace --doc`.
- [x] 6.3 `openspec validate --all --strict`; `bash scripts/check-file-size.sh`.
