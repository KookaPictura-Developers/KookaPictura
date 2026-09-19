## 1. Codec probe and typed error

- [x] 1.1 Add an image-header probe module to `crates/pictura-codec/src/` (sniff PNG `IHDR`, JPEG JFIF/SOF, GIF, BMP, TIFF `II`/`MM`, WebP `RIFF`), exposing `ImageProbe { format, width, height, bit_depth }`, `ImageBudget { max_dimension, max_alloc_bytes }` with documented defaults, and a typed `ImportError` carrying the source description, declared dimensions, bytes read, and which limit was exceeded.
- [x] 1.2 Implement `pub fn probe_image(bytes: &[u8], budget: ImageBudget) -> Result<ImageProbe, ImportError>`: pure, Qt-free, no decode, no allocation from declared size; refuse over-dimension, over-allocation, unknown container, and truncated headers. Re-export from `lib.rs`.
- [x] 1.3 Unit tests in `crates/pictura-codec/src/tests.rs` (or a sibling): each recognized header returns the declared metadata; over-dimension and over-allocation refuse naming the limit; unknown and truncated inputs error without panicking.

## 2. Engine construction from RGBA8888

- [x] 2.1 Add `pub fn Document::from_rgba(name: &str, width: u32, height: u32, rgba: &[u8]) -> Document` to `crates/pictura-core/src/lib.rs`: RGB/8-bit, composite seeded as a 4-plane RGBA `PixelBuffer`, exactly one pixel layer named `name` with `rect = (0, 0, w, h)` and planar channels `0,1,2,-1`, no smart object or adjustment, no panic on a short buffer.
- [x] 2.2 Add `pub fn add_raster_layer_from_rgba(doc: &mut Document, name: &str, width: u32, height: u32, rgba: &[u8]) -> String` to `crates/pictura-render/src/document_ops/layer_ops/create.rs` (mirroring `add_solid_fill`/`place_smart_object`): topmost append, `rect = (0, 0, w, h)`, planar channels, path via `format_segments`; empty string for a zero dimension. Register in `layer_ops/mod.rs` and re-export from `document_ops/mod.rs` and `lib.rs`.
- [x] 2.3 Unit tests in `pictura-core` for `from_rgba` (size/mode/depth, one layer, planar fidelity including alpha, short-buffer no panic).
- [x] 2.4 Unit tests in `pictura-render` for `add_raster_layer_from_rgba` (topmost, native-size, planar pixels, zero-dimension refusal) and that `convert_to_smart_object` succeeds on the returned path.

## 3. App decode edge and bridges

- [x] 3.1 Declare the C++ decode helper in the `#[cxx::bridge]` of `crates/pictura-app/src/cxxqt_object.rs`, e.g. `fn decode_image_rgba(data: &[u8], width: &mut i32, height: &mut i32) -> Vec<u8>;`.
- [x] 3.2 Implement it in a small new `crates/pictura-app/cpp/` translation unit (`QImage::fromData(...).convertToFormat(QImage::Format_RGBA8888)`; return the packed buffer) and register the `.cpp`/`.h` in `CMakeLists.txt` (no globbing).
- [x] 3.3 Add `pub fn open_image(self: Pin<&mut Self>, path: &QString) -> bool` in `crates/pictura-app/src/cxxqt_object/impl_core.rs`: read bytes → `probe_image` → `decode_image_rgba` → check the actual decoded allocation → `Document::from_rgba` → store composite/image → reset edit state → one `"Open"` snapshot → path `None` → unmodified; `false` without mutating on any refusal.
- [x] 3.4 Add `pub fn place_image(self: Pin<&mut Self>, path: &QString) -> QString` in `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs`: read bytes → probe → decode → actual-allocation check → `add_raster_layer_from_rgba` → `convert_to_smart_object` → `clear_link_sets` + `recomposite` + `record("Place")` → return the path; empty without recording on refusal.
- [x] 3.5 Declare both methods on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`.

## 4. Dialogs and native-PSD routing

- [x] 4.1 `crates/pictura-app/cpp/frame.cpp`: `showOpenDialog` offers `Images (…)` beside `Photoshop files (*.psd *.psb)`; a chosen `*.psd`/`*.psb` uses `openPath`, every other suffix uses a new `openImagePath` that constructs a `PictureView`, calls `open_image`, and on success `addDocument(view, QString())` (untitled), deleting the view on failure.
- [x] 4.2 `crates/pictura-app/cpp/frame.h`/`frame.cpp`: add the `openImagePath(const QString&)` declaration and definition beside `openPath`.
- [x] 4.3 `crates/pictura-app/cpp/frame_menus.cpp`: the `File > Place…` handler offers `Images (…)` and routes `*.psd`/`*.psb` to `place_smart_object` and other suffixes to `place_image`, refreshing on a non-empty result.
- [x] 4.4 Confirm PSD/PSB is never handed to the Qt decode edge (routing branch only).

## 5. C++ self-test

- [x] 5.1 Extend an existing suite in `crates/pictura-app/cpp/selftest_*.cpp` (no new file, so `CMakeLists.txt` app sources are otherwise unchanged): save a small solid-color PNG to a temp path, `open_image` it, and assert a one-layer document of the expected size and an `"Open"` history state.
- [x] 5.2 Place the same PNG into an open document and assert a new topmost smart-object layer and exactly one `"Place"` history state; assert an unrecognized file and an over-budget file refuse with no history state; clean up temp files.
- [x] 5.3 Use the next free exit code **290** (the current maximum in `crates/pictura-app/cpp/selftest*.cpp` is 289) with `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`, and keep the file within its `scripts/file-size-allowlist.txt` ceiling.

## 6. Gates

- [x] 6.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 6.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 6.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`.
- [x] 6.4 `openspec validate image-import --strict` and `openspec validate --all --strict`.
