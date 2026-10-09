# Tasks: shape-layer-actions

## 1. Engine — attributes

- [x] 1.1 `shape_style.rs`: add `ShapeAttributes`, `copy_shape_attributes`, `paste_shape_attributes`; export from `layer_ops/mod.rs`.
- [x] 1.2 Unit tests: copy returns fill+stroke for a shape and `None` for a non-shape; paste refuses a non-shape; paste replaces fill and stroke and reports a change; an identical paste is a no-op.

## 2. Engine — Rasterize Shape

- [x] 2.1 `rasterize.rs`: add `rasterize_shape` (scratch-composite the layer, bake within its rect, drop `adjustment`/`vmsk`/`vogk`/`lfx2`); add shape layers to `rasterize_all_layers`; export.
- [x] 2.2 Unit tests: a shape rasterizes and its composite is unchanged; the adjustment/vector/live-shape/stroke data is dropped; a second run refuses; a non-shape refuses without mutation.

## 3. Engine — forced locks

- [x] 3.1 `shape_layer.rs`: add `has_forced_locks`; export.
- [x] 3.2 `properties.rs`: `set_lock_paths` cannot clear Transparency/Image on a forced-lock layer.
- [x] 3.3 Unit test: clearing a forced bit is refused (Position still clears); a non-forced layer clears normally.

## 4. Bridge

- [x] 4.1 `state.rs`: `shape_attributes` clipboard field (+ default).
- [x] 4.2 `shapes.rs`: `shape_copy_attributes`, `shape_paste_attributes`, `shape_rasterize`, `shape_has_copied_attributes` invokables; each records at most one state and recomposites.
- [x] 4.3 `impl_layers.rs`: `layer_row_lock` reports the forced bits.

## 5. C++ wiring

- [x] 5.1 `commands.h` / `command_tree.cpp`: add `LayerCopyShapeAttributes`, `LayerPasteShapeAttributes`; make `Rasterize > Shape` implemented.
- [x] 5.2 `frame_menus.cpp`: handlers + enabled providers for the three shape commands.
- [x] 5.3 `layers_panel_menu.{cpp,h}`: `Shape` mask and Copy/Paste/Rasterize shape rows; detect shape rows from `LayerRowShapeRole`.
- [x] 5.4 `layers_panel.cpp`: force Transparency/Image checks and disable those toggles for shape/type current rows.

## 6. Tests (Qt)

- [x] 6.1 `tst_shape_tools`: create two shapes, copy/paste attributes, assert refusal on a pixel layer, rasterize shape to pixels.
- [x] 6.2 `tst_layers_panel`: shape row menu texts; forced-lock toggles checked and disabled for a shape and a type layer.
- [x] 6.3 `tst_command_tree`: the three Layer-menu commands exist and are enabled per current layer.

## 7. Verify

- [x] 7.1 `cargo nextest run -p pictura-render -p pictura_app`
- [x] 7.2 `cargo clippy -p pictura-render -p pictura_app --all-targets -- -D warnings`
- [x] 7.3 `cargo fmt --all --check`
- [x] 7.4 `cmake --build build --parallel`
- [x] 7.5 `ctest --test-dir build -R '^tst_layers_panel$|^tst_command_tree$|^tst_shape' --output-on-failure`
- [x] 7.6 `bash scripts/verify-fast.sh`
- [x] 7.7 `openspec validate --all --strict`
- [x] 7.8 Archive the change with `openspec archive shape-layer-actions -y`.
