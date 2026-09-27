# Tasks: annotation-tools

## 1. Engine

- [x] 1.1 `pictura_core::annotations` (`Annotations`, `MarkerKind`, `Ruler`, `Measurement`) and `Document::annotations`; tests.

## 2. Bridge

- [x] 2.1 `cxxqt_object/annotations.rs` bridge; `PictureViewRust::ruler`.

## 3. Tools

- [x] 3.1 `tool_annotations.cpp`, `tool_ruler.cpp`, `image_view_annotations.cpp`; catalog rows enabled; options-bar rows.
- [x] 3.2 Info panel sampler readouts; Notes panel.

## 4. Verification

- [x] 4.1 `annotation_tools` self-test (535); guard (98) probes Count; `bash scripts/verify-fast.sh`.
