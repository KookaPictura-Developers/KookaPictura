# Proposal: perspective-crop-cursor

## Why

The Perspective Crop pointer was the tool's trapezoid icon. CS6 and photorust
show a precise crosshair while placing the quad, and a move cursor over a corner
handle (reported on PR #98).

## What Changes

- The `tool.perspectivecrop` cursor asset becomes the precise crosshair the
  marquee uses; the toolbox icon is unchanged.
- Hovering a corner handle shows the move cursor.
- The `crop_group` self-test (532) asserts both.

## Capabilities

### Modified Capabilities

- `tools/perspective-crop`: adds the pointer requirement.

## Impact

- `assets/cursors/tool.perspectivecrop.svg`, `tool_perspectivecrop.cpp`,
  `selftest_crop_group.{h,cpp}`. No new dependency.
