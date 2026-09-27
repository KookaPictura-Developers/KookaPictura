# Tasks: magnetic-lasso

## 1. Engine

- [x] 1.1 Add `pictura-select/src/magnetic.rs` (`EdgeMap::from_buffer`, `trace`) and export `EdgeMap`.
- [x] 1.2 Unit tests: flat image, snapping on and toward an edge, narrow width, Contrast ignoring a weak edge, RGB luma, off-canvas fallback, connectivity/endpoints, tiny/short buffers.

## 2. Bridge

- [x] 2.1 Add `edge_map` to `PictureViewRust` and the `cxxqt_object/magnetic.rs` bridge (`magnetic_begin/trace/end`, 32-bit refusal); register it in `build.rs` and `cxxqt_object.rs` without growing the ceiling file.

## 3. Tool

- [x] 3.1 Add `tool_magneticlasso.cpp` and register it; add `removeLassoPoint`/`lassoInProgress` to `ToolHandler` and the Width/Contrast/Frequency options to `ToolContext`/`ToolController`.
- [x] 3.2 Enable the catalog row, hints, and selection routing; Enter/Delete in `frame.cpp`; `[`/`]` in `frame_build.cpp` and `applyBrushShortcut`.
- [x] 3.3 Options-bar Width/Contrast/Frequency fields and a disabled Stylus Pressure box; size the bar to the active page.

## 4. Self-test

- [x] 4.1 Append `magnetic_lasso` (code 530) and point the unimplemented-tool guard (code 98) at Perspective Crop.

## 5. Verification

- [x] 5.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/verify-fast.sh`, `cargo deny check`, CMake build + headless self-test.
