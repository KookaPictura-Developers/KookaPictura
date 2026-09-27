# Tasks: crop-tool-box

## 1. Engine and bridge

- [x] 1.1 `delete_cropped_pixels` with tests; `crop_to` bridge.

## 2. Tool

- [x] 2.1 `crop_grip.h` (shared with Slice Select); rewrite `tool_crop.cpp`; `ImageView::setCropBox`; crop options on the controller.
- [x] 2.2 Options bar Crop page (ratio, Delete Cropped Pixels, Cancel, Apply); hint text.

## 3. Verification

- [x] 3.1 `crop_tool` self-test (534); move the ported checks to `selftest_ports.cpp`; `bash scripts/verify-fast.sh`.
