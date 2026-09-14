# M4-C — App Layer/Adjustment UI

Goal: make layers and adjustments real in the app — a layer list, visibility
toggles, and adding an adjustment layer with a live recomposite.

The app is a C++ Qt shell (`crates/pictura-app/cpp/main.cpp`) over a Rust
`PictureView` QObject (cxx-qt). `pictura-render::composite_rgba` already composes
pixel layers and decoded adjustment layers.

## Scope

In:
- Extend the Rust `PictureView` QObject with:
  - layer introspection: `layer_count`, `layer_name(i)`, `layer_kind(i)`
    (pixel/group/adjustment), `layer_visible(i)`;
  - commands: `set_layer_visible(i, bool)`, `add_adjustment(kind)`,
    `remove_layer(i)`, then recomposite + emit a changed signal.
- Small **encoders** for the adjustment subset the renderer decodes, next to the
  decoder in `pictura-render` (`AdjustmentData` for Invert `nvrt`, Posterize
  `post`, Threshold `thrs`, BrightnessContrast `brit`, HueSaturation `hue2`), so
  the app can construct adjustment layers. `pictura-core` stays dependency-free.
- C++ dock UI: a layer list with visibility checkboxes, an adjustment-type combo
  + "Add" button, a remove button; re-render on change.
- `--self-test` (headless): after loading a layered PSD, add an Invert adjustment
  layer, recomposite, assert the pixels changed as expected; toggle a layer's
  visibility and assert the output changes; exit 0.

Out:
- Editing adjustment parameters with widgets, undo/redo, drag-reorder, the full
  layer thumbnail UI.

## Acceptance

- `cargo test --workspace` green; new Rust tests for the encoders and the
  self-test path.
- `cmake --build build` succeeds; `xvfb-run ./build/pictura --self-test
  crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0 and reports the
  adjustment applied.
- `scripts/guard.sh` green.
