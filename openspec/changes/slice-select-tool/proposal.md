# Proposal: slice-select-tool

## Why

The Slice tool (#4) can only add slices; a mistaken slice could be removed only
by Undo. The Slice Select tool (issue #5) selects, moves, resizes, and deletes
user slices; photorust implements it on the same slice model.

## What Changes

- `pictura_render::{set_slice, remove_slice}` and bridge
  `set_user_slice(commit)` / `remove_user_slice`.
- `SliceSelectToolHandler` in `tool_slice.cpp`: click to select (orange
  handles), drag inside to move, drag an edge/corner handle to resize, Delete to
  remove, Escape to deselect; resize cursors over the handles.
- The unimplemented-tool guard (98) now probes the Ruler.
- The `crop_group` self-test (532) covers Slice Select.

## Capabilities

### Modified Capabilities

- `tools/slice-tool`: adds editing user slices and the Slice Select tool.

## Impact

- `slices.rs`, `crop_group.rs`, `tool_slice.cpp`, `image_view.h`,
  `image_view_overlays.cpp`, `tools.cpp`, `tool_catalog.cpp`, `selftest.cpp`,
  `selftest_crop_group.{h,cpp}`. No new dependency.
