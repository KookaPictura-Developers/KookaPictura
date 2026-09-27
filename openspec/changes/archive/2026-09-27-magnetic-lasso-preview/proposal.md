# Proposal: magnetic-lasso-preview

## Why

The Magnetic Lasso shipped (`magnetic-lasso`) drew its live outline as an open
solid polyline with no origin marker. photorust, like CS6, draws the outline
closed — the traced path, the live wire, and a straight connector from the
cursor back to the origin — and marks the origin with a small hollow square, so
the user can see where the outline will close (reported on PR #95).

## What Changes

- `ImageView::setSelectionPreviewOrigin`: a 5-screen-px hollow square at the
  origin of a click-driven lasso, cleared with the preview.
- The Magnetic Lasso preview becomes a closed, dashed outline ending at the
  cursor, with the origin marker.
- The `magnetic_lasso` self-test (code 530) asserts the closed outline and the
  marker, and that cancelling clears the marker.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `tools/magnetic-lasso`: adds the live outline preview requirement.

## Impact

- `image_view.{h,cpp}`, `tool_magneticlasso.cpp`, `selftest_magnetic_lasso.{h,cpp}`.
- No new dependency.
