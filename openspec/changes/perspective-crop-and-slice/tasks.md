# Tasks: perspective-crop-and-slice

## 1. Engine

- [x] 1.1 `layer_ops/perspective_crop.rs` (`perspective_crop`, `perspective_crop_size`, `perspective_crop_refusal`) reusing `solve_homography`/`bilinear`; tests.
- [x] 1.2 `Document::slices` and `document_ops/slices.rs` (`add_slice`, `resolve_slices`, `Slice`); tests.

## 2. Bridge

- [x] 2.1 `cxxqt_object/crop_group.rs` bridge; consolidate `build.rs` bridge files.

## 3. Tools

- [x] 3.1 `tool_perspectivecrop.cpp`, `tool_slice.cpp`, `image_view_overlays.cpp`; `ToolHandler::onDocumentRefreshed`; Enter commits a Perspective Crop; catalog rows enabled.

## 4. Verification

- [x] 4.1 `crop_group` self-test (532); guard (98) probes Slice Select; `bash scripts/verify-fast.sh`, `cargo deny check`.
