# Proposal: crop-tool-box

## Why

Review on PR #98 (Zawaro): Kooka's Crop tool was drag-a-region-then-Enter only,
with no options bar, while photorust (the port source) already has CS6's
redesigned crop: a default crop box on activation, a crop shield,
rule-of-thirds guides, eight resize handles with resize/move cursors, moving
and ratio-locked resizing, an aspect-ratio preset combo, Delete Cropped
Pixels, Cancel/Apply buttons, and Escape to reset.

## What Changes

- `pictura_render::delete_cropped_pixels`: after `crop_document`, trim each
  pixel layer's pixels and mask to the new canvas (layers with live type, smart
  objects, vector masks, or retained 16/32-bit samples keep theirs —
  `ponytail:`). Bridge `crop_to(x, y, w, h, delete_cropped)` records one
  "Crop" state; the existing `crop` (Image > Crop, control server) is unchanged.
- `tool_crop.cpp` rewritten: a canvas-sized box on activation (fitted to the
  ratio), eight handles, move inside, a new box outside, ratio-locked resizing;
  Enter / double-click inside / Apply commit; Escape / Cancel reset. A crop is
  "pending" only while the tool is active and the box differs from the canvas,
  so Image > Crop still crops to the selection.
- `ImageView::setCropBox` overlay (shield, thirds, frame, handles).
- Options bar Crop page: ratio presets (Unconstrained, 1:1, 4:5, 5:7, 2:3,
  16:9), Delete Cropped Pixels (on by default, per `docs/03-tools/crop-tool.md`),
  Cancel ✘ / Apply ✓.
- `crop_grip.h`: the eight-handle geometry shared with Slice Select.
- C++ self-test `crop_tool` (code 534); the ported-feature checks move to
  `selftest_ports.cpp` to keep `selftest_layers_controls.cpp` under its cap.

## Capabilities

### Modified Capabilities

- `tools/canvas-tools`: the Crop tool requirement gains the crop box, options,
  and Delete Cropped Pixels.

## Impact

- `crop.rs` (+ tests), `crop_group.rs`, `tool_crop.cpp`, new `crop_grip.h`,
  `tool_slice.cpp`, `image_view.h`, `image_view_overlays.cpp`, `tools.{h,cpp}`,
  `tool_context.h`, `tool_handler.h`, `options_bar.{h,cpp}`,
  `tool_catalog.cpp`, new `selftest_crop_tool.{h,cpp}` and
  `selftest_ports.{h,cpp}`, `selftest_layers_controls.cpp`, `CMakeLists.txt`.
- No new dependency. Follow-ups #100–#105 cover the remaining CS6 crop options
  (not in photorust).
