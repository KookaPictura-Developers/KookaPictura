## Why

The Layers panel works, but four rough edges remain after the controls/chrome/row
passes: each row paints a native checkbox next to the custom eye icon, the `%`
sign sits outside the value box, a locked layer has no lock indicator on its row,
and dragging Opacity/Fill writes one undo state per tick instead of one per drag.

## What Changes

- Row visibility is a single eye icon; the native checkbox indicator is removed.
- The Opacity/Fill `%` sign is rendered inside the value box, not beside it.
- A locked layer draws a lock badge on the right side of its row.
- Dragging Opacity/Fill (label scrub, field scrub, popup slider, or text entry)
  shows a live preview and records exactly one undo state when the edit finishes.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `layers-panel`: rows show only an eye toggle (no checkbox); locked rows show a
  lock badge; the `%` is inside the Opacity/Fill box; the percent controls live-
  preview and add a single undo state per edit.

## Impact

- Rust bridge: new `preview_layers_opacity`/`commit_layers_opacity` and
  `preview_layers_fill`/`commit_layers_fill` invokables plus two preview flags in
  `crates/pictura-app/src/cxxqt_object{,/impl_layers}.rs`.
- C++ UI: `crates/pictura-app/cpp/panels/percent_field.{h,cpp}`,
  `layers_panel{.cpp,_internal.h,_test.cpp}`, and
  `crates/pictura-app/cpp/selftest_layers_controls.{h,cpp}`.
- No new dependency; no document-format change.
