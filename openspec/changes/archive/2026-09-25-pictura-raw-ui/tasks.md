# Tasks: pictura-raw-ui

## 1. Bridge

- [x] 1.1 Declare `layer_pictura_raw_settings` and `apply_pictura_raw_filter` in `cxxqt_object.rs`; add `impl_pictura_raw.rs`.
- [x] 1.2 `layer_pictura_raw_settings`: read the camera-raw `Fltr` of the resolved layer and encode the 11 values, or empty.
- [x] 1.3 `apply_pictura_raw_filter`: convert a plain raster target first, apply Pictura Raw, recomposite, and record one `"Pictura Raw"` state; false without recording on refusal.

## 2. Dialog

- [x] 2.1 Add `cpp/pictura_raw_dialog.{h,cpp}` with the 11 labelled controls, documented ranges, and OK/Cancel.
- [x] 2.2 Add both files to `CMakeLists.txt` and include the header from `frame_includes.h`.

## 3. Command wiring

- [x] 3.1 Add `command_ids::FilterPicturaRaw`.
- [x] 3.2 Replace the `Filter > Pictura Raw…` leaf with a real command entry.
- [x] 3.3 Register the handler (prefill from the bridge, apply on OK) and the enablement provider.

## 4. Tests

- [x] 4.1 Add `selftest_pictura_raw.{h,cpp}` (code 528): raster→smart object, pixels changed, one `"Pictura Raw"` state, settings attached; a second apply records exactly one more state.
- [x] 4.2 Register the new check file in `CMakeLists.txt` and call it from `runLayersControlsChecks`.

## 5. Docs and OpenSpec

- [x] 5.1 Update `docs/dev/STATE.md` and `docs/dev/psd-support-roadmap.md`.
- [x] 5.2 Add the `pictura-raw` capability delta, proposal, and design under `openspec/changes/pictura-raw-ui/`.

## 6. Verification

- [x] 6.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 6.2 `cargo nextest run --workspace`; `cargo test --workspace --doc`.
- [x] 6.3 `openspec validate --all --strict`; `bash scripts/check-file-size.sh`.
- [x] 6.4 CMake build; `./build/pictura --headless --self-test`.
