# Tasks: hdr-conversion

## 1. Engine op

- [x] 1.1 Add `Samples::to_u16` in `crates/pictura-core/src/samples.rs` (`u8` widens `v*257`, `u16` unchanged, `f32` is `clamp(v,0,1)*65535` rounded) with a unit test.
- [x] 1.2 Add `crates/pictura-render/src/document_ops/depth.rs` with `convert_depth_exposure_gamma(doc, out, params)` implementing the gate, color-only tone map, output quantization, store/depth/composite rebuild, and layer `source_channels` clear.
- [x] 1.3 Re-export through `document_ops/mod.rs` and `crates/pictura-render/src/lib.rs`.

## 2. Tests

- [x] 2.1 `crates/pictura-render/src/tests/native_depth.rs`: 32 -> 16 keeps a >1.0 tone-mapped sample (not pre-clamped), extras untouched, store typed u16, `retains_source_depth()`; 32 -> 8 clears the store; refusals leave the document byte-identical.
- [x] 2.2 `crates/pictura-codec/src/tests/depth.rs`: read `rgb32.psd`, convert to 16, `write_psd`, `read_psd`; assert header depth 16 and `source_depth == Some(Sixteen)`.

## 3. App command and dialog

- [x] 3.1 Add real command ids in `commands.h`; replace the two `leaf()` stubs in `command_tree.cpp`.
- [x] 3.2 Add `HdrConversionDialog` (`hdr_conversion_dialog.{h,cpp}`) with Exposure and Gamma fields; register both files in `CMakeLists.txt`.
- [x] 3.3 Add the `convert_depth` bridge (`cxxqt_object.rs` + `impl_transform/dispatch.rs`) and the handlers/enabled providers in `frame_menus.cpp` gated on `document_depth_bits() == 32`.

## 4. C++ self-test

- [x] 4.1 Append one `selftest_layers_adjustments.cpp` check (next free code 526): hand-build a 32-bit PSD, open it, run `convert_depth(16, 0, 1.0)`, assert `document_depth_bits() == 16`.

## 5. Docs and verification

- [x] 5.1 Update `docs/dev/STATE.md` and `docs/dev/psd-support-roadmap.md` with the shipped change and its ceilings.
- [x] 5.2 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -p pictura-core -p pictura-adjust -p pictura-render -p pictura-codec`, `cargo test --workspace --doc`, `openspec validate hdr-conversion --strict`, file-size gate.
- [x] 5.3 CMake build + `./build/pictura --headless --self-test`.
