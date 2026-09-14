## Why

M4 decoded and composited adjustment layers in `pictura-render`, but the Qt
app could not see or change a layer stack: the shell showed a single flattened
image with no way to toggle visibility, add an adjustment, or remove a layer.
M4-C closes that gap so layers and adjustments are usable end to end.

## What Changes

- Extend the Rust `PictureView` QObject with layer introspection:
  `layer_count`, `layer_name(i)`, `layer_kind(i)` (`pixel`/`group`/`adjustment`),
  `layer_visible(i)`.
- Add mutating commands that recomposite and emit the `changed` signal:
  `set_layer_visible(i, bool)`, `add_adjustment(kind)`, `remove_layer(i)`.
- Add small adjustment **encoders** in `pictura-render` next to the decoder so
  the app can build adjustment layers for the decoded subset (Invert `nvrt`,
  Posterize `post`, Threshold `thrs`, BrightnessContrast `brit`, HueSaturation
  `hue2`). `pictura-core` stays dependency-free.
- Add a C++ Layers dock: layer list with visibility checkboxes, an
  adjustment-type combo + Add button, a Remove button, and a re-render on every
  `changed` signal.
- Extend the headless `--self-test`: after loading a layered PSD, add an Invert
  adjustment, recomposite, assert pixels changed; toggle a layer's visibility
  and assert the output changed; exit 0.

Out of scope: editing adjustment parameters with widgets, undo/redo,
drag-reorder, and the full layer-thumbnail UI.

## Capabilities

### New Capabilities

- `adjustment-ui`: layer introspection, layer-mutating commands, in-memory
  adjustment encoders, the C++ Layers dock, and the headless self-test
  assertions that prove the recomposite path.

### Modified Capabilities

<!-- None: openspec/specs/ is empty; this change introduces a new capability. -->

## Impact

- `crates/pictura-app/src/cxxqt_object.rs` — new `#[qinvokable]` methods on
  `PictureView`, the `changed` signal wiring, and `adjustment_layer` helper
  tests.
- `crates/pictura-app/cpp/main.cpp` — Layers dock widgets, signal/slot wiring,
  and the extended `--self-test` branch.
- `crates/pictura-render/src/lib.rs` — encoder functions
  (`encode_invert`, `encode_posterize`, `encode_threshold`,
  `encode_brightness_contrast`, `encode_hue_saturation`) plus round-trip tests.
- No new dependencies. `pictura-core` is untouched and remains dependency-free.
