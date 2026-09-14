## 1. Adjustment encoders in pictura-render

- [x] 1.1 Add `encode_invert`, `encode_posterize`, `encode_threshold`, `encode_brightness_contrast`, and `encode_hue_saturation` beside the decoder in `crates/pictura-render/src/lib.rs`, emitting the raw `AdjustmentData` keys `nvrt`, `post`, `thrs`, `brit`, `hue2`
- [x] 1.2 Clamp out-of-range parameters to the decoder's accepted ranges
- [x] 1.3 Add `encode_decode_round_trips` asserting each encoder decodes back to the intended `Adjustment` variant and clamped values
- [x] 1.4 Confirm `pictura-core` gains no dependency and its manifest is unchanged

## 2. PictureView introspection and commands

- [x] 2.1 Add `#[qinvokable]` introspection: `layer_count`, `layer_name(i)`, `layer_kind(i)` (`pixel`/`group`/`adjustment`), `layer_visible(i)` with safe defaults for missing document/out-of-range
- [x] 2.2 Add `set_layer_visible(i, visible)` that mutates the layer, recomposites, and emits `changed`; no-op on bad index
- [x] 2.3 Add `add_adjustment(kind)` using the encoders, building the layer (with a selection-derived mask when a selection is active), appending, recompositing, and emitting `changed`; return `false` for unknown kinds
- [x] 2.4 Add `remove_layer(i)` that removes, recomposites, and emits `changed`; no-op on bad index
- [x] 2.5 Gate every command through a shared `recomposite()` refresh so the cached `QImage` and the `changed` signal stay in sync

## 3. C++ Layers dock

- [x] 3.1 Add a `QDockWidget` "Layers" to `crates/pictura-app/cpp/main.cpp` with a checkable `QListWidget`, an adjustment-type `QComboBox`, an Add button, and a Remove button
- [x] 3.2 Populate one row per layer using `layer_name`/`layer_kind`/`layer_visible`
- [x] 3.3 Wire checkbox `itemChanged` to `set_layer_visible`, Add to `add_adjustment(currentData)`, and Remove to `remove_layer(currentRow)`
- [x] 3.4 Connect `PictureView::changed` to a `refresh` that rebuilds the list (under `QSignalBlocker`) and calls `window->setImage(view.image())`

## 4. Headless self-test

- [x] 4.1 Extend `--self-test` to load a layered PSD, add an Invert adjustment, and assert the last layer is an adjustment and the composite changed as expected
- [x] 4.2 Toggle a layer's visibility and assert the output differs, with distinct non-zero exit codes on each failure

## 5. Verification

- [x] 5.1 `cargo test --workspace` green, including the encoder round-trip and `cxxqt_object` adjustment/visibility tests
- [x] 5.2 `cmake --build build` succeeds and `xvfb-run ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0
- [x] 5.3 `scripts/guard.sh` green
