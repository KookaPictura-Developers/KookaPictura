# Proposal: click-to-deselect

## Why

With a selection active, clicking outside it with a selection tool did nothing:
the selection went away only when another selection replaced it. In CS6 and
photorust a click without a drag clears the selection (issue #96).

## What Changes

- A Rectangular / Elliptical Marquee click with no area, a Lasso release with
  fewer than three points, or a Polygonal Lasso closed with fewer than three
  vertices runs Deselect when the gesture's mode is New and a selection exists.
- Add / Subtract / Intersect leave the selection alone.
- C++ self-test `click_deselect` (code 531).

## Capabilities

### Modified Capabilities

- `tools/shape-selection-tools`: adds the click-to-deselect requirement.

## Impact

- `crates/pictura-app/cpp/tool_selection.cpp`, new
  `selftest_click_deselect.{h,cpp}`, `selftest_layers_controls.cpp`,
  `CMakeLists.txt`. No new dependency.
