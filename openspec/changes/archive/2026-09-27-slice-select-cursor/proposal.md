# Proposal: slice-select-cursor

## Why

The Slice Select pointer was the tool's icon. CS6 and photorust show a precise
crosshair away from the selected slice's handles (reported on PR #98).

## What Changes

- The `tool.sliceselect` cursor asset becomes the precise crosshair the
  marquee uses; the toolbox icon is unchanged. Move/resize cursors over the
  selected slice are unchanged.
- The `crop_group` self-test (532) compares both crop-group crosshairs with the
  marquee cursor pixmap.

## Capabilities

### Modified Capabilities

- `tools/slice-tool`: adds the Slice Select pointer requirement.

## Impact

- `assets/cursors/tool.sliceselect.svg`, `selftest_crop_group.{h,cpp}`. No new
  dependency.
