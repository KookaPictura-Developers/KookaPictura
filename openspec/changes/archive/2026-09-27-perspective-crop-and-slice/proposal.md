# Proposal: perspective-crop-and-slice

## Why

The Perspective Crop (issue #3) and Slice (issue #4) tools were catalogued but
disabled. photorust (perfecto25/photorust) ships both: a homography-based
perspective crop and a slice set with derived auto slices. This change ports
them onto Kooka's document model, tool framework, and canvas.

## What Changes

- `pictura_render::{perspective_crop, perspective_crop_size,
  perspective_crop_refusal}`: warp every pixel layer, layer mask, and extra
  channel so a user quad becomes the canvas, reusing Kooka's
  `solve_homography`/`bilinear`; refuses live type, smart objects, vector masks,
  and retained 16/32-bit samples rather than silently rasterizing or dropping
  them.
- `Document::slices` (user slices) and `pictura_render::{add_slice,
  resolve_slices, Slice}`: auto slices tile the rest of the canvas, all numbered
  in reading order.
- A `cxxqt_object/crop_group.rs` bridge (`perspective_crop_commit`,
  `perspective_crop_refusal_reason`, `slice_count`, `slice_at`,
  `add_user_slice`); `build.rs` now lists every bridge file once.
- `tool_perspectivecrop.cpp` and `tool_slice.cpp`, `ImageView` overlays
  (`image_view_overlays.cpp`), and a `ToolHandler::onDocumentRefreshed` hook so
  the slice overlay follows edits and undo.
- C++ self-test `crop_group` (code 532); the unimplemented-tool guard (code 98)
  now probes Slice Select.

## Capabilities

### New Capabilities

- `tools/perspective-crop`: the Perspective Crop engine and tool.
- `tools/slice-tool`: user and auto slices and the Slice tool.

## Impact

- `pictura-core` (`Document::slices`), `pictura-codec` (constructors),
  `pictura-render` (`layer_ops/perspective_crop.rs`, `document_ops/slices.rs`,
  `transform.rs` visibility), `pictura-app` bridge and C++ as above.
- No new dependency.
