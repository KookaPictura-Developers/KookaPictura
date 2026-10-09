## 1. Shared adjustment table

- [x] 1.1 Add `crates/pictura-app/cpp/layer_adjustments.h` with `struct LayerAdjustment { const char* kind; const char* leaf; bool ellipsis; }` and `inline constexpr LayerAdjustment kLayerAdjustments[]` holding the sixteen CS6 kinds in `frame_menus_adjust.cpp`'s current order.
- [x] 1.2 Register the header in `CMakeLists.txt`.
- [x] 1.3 In `frame_menus_adjust.cpp`, delete the local `LayerAdjustment` struct and `kLayerAdjustments` table and include the header instead; keep `wireLayerAdjustments` behavior unchanged.

## 2. Strip New Fill / Adjustment menu

- [x] 2.1 In `layers_panel.cpp`, build the menu from the shared table: the fill group (Solid Color…, Gradient…, disabled Pattern…), a separator, then all sixteen kinds labelled `leaf` with `…` when `ellipsis`, each calling `view_->add_adjustment(kind)`.
- [x] 2.2 Add the disabled Pattern… entry with a `— not implemented yet` tooltip.

## 3. Layer Content Options

- [x] 3.1 Add `command_ids::LayerContentOptions` to `commands.h`.
- [x] 3.2 In `command_tree.cpp`, replace the `leaf(...)` with a frozen-id `registry.add(..., true)` so the command is implemented.
- [x] 3.3 In `frame_menus.cpp`, add an enabled provider (current path is a row with an adjustment block) and a handler that shows the Properties panel and refreshes it. Do nothing when the provider is false.

## 4. Tests

- [x] 4.1 Extend `tst_layers_panel.cpp`: the strip menu contains all sixteen adjustment kinds plus Gradient… and a disabled Pattern…; choosing a kind adds a layer.
- [x] 4.2 Extend `tst_command_tree.cpp`: `Layer Content Options…` is disabled with a pixel current layer and, with an adjustment current, invoking it makes the Properties panel visible and reflect that layer.

## 5. Gates

- [x] 5.1 `cmake --build build --parallel`.
- [x] 5.2 `ctest --test-dir build -R '^tst_command_tree$|^tst_layers_panel$|^tst_properties_panel$|^tst_image_adjustments$' --output-on-failure`.
- [x] 5.3 `bash scripts/verify-fast.sh`.
- [x] 5.4 `openspec validate adjustment-fill-complete --strict`; then archive the change.
- [x] 5.5 `cargo nextest run -p pictura_app` (only if Rust changed; expected unchanged).
