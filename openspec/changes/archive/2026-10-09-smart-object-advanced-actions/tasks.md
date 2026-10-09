# Tasks

## 1. Engine operations

- [x] 1.1 Add `can_reset_smart_object_transform` / `reset_smart_object_transform` to `layer_ops/smart_object.rs` (decode payload for native size, set `rect = (0,0,w,h)`, clear the proxy, keep the object) and re-export them from `layer_ops/mod.rs`; verify with `cargo nextest run -p pictura-render reset`
- [x] 1.2 Add `can_convert_smart_object_to_layers` / `convert_smart_object_to_layers` (map the source frame into the object rect, resample pixel channels, recurse groups, splice at the object slot, drop the linked record, single raster layer for an empty source) and re-export; verify with `cargo nextest run -p pictura-render to_layers`
- [x] 1.3 Add `can_new_smart_object_via_copy` / `new_smart_object_via_copy` (deep-clone, rename `"<name> copy"`, clear the copy's preserved config descriptor/uuid/blocks, insert above, return path) and re-export; verify with `cargo nextest run -p pictura-render via_copy`
- [x] 1.4 Engine unit tests in `crates/pictura-render/src/tests/smart_object.rs`: reset yields native placement at the origin and keeps the object; convert yields the source stack at the object's position (and a scaled object scales its stack); a replaced copy leaves the original's payload unchanged; verify with `cargo nextest run -p pictura-render -p pictura_app`

## 2. App bridge

- [x] 2.1 Add the reset/convert-to-layers/new-via-copy predicates and invokables as free functions in the new nested bridge module `cxxqt_object/impl_layers/smart_object_actions.rs` (record `"Reset Transform"` / `"Convert to Layers"` / `"New Smart Object via Copy"`; via-copy returns the copy path), register it in `impl_layers.rs` + `build.rs`, keeping `cxxqt_object.rs` unchanged at 1227; verify with `cargo clippy -p pictura_app --all-targets -- -D warnings`

## 3. Menu wiring

- [x] 3.1 Add `LayerSmartObjectResetTransform`, `LayerSmartObjectConvertToLayers`, and `LayerSmartObjectNewViaCopy` to `commands.h`; register/enable Reset Transform and Convert to Layers in `command_tree.cpp` and turn the `New Smart Object via Copy` leaf into a real command; verify with `cargo nextest run -p pictura_app` and the `tst_command_tree` suite
- [x] 3.2 Wire handlers and enabled-providers for the three commands in `frame_menus.cpp`, reusing the smart-object eligibility predicates; verify with `cmake --build build --parallel` and `ctest --test-dir build -R '^tst_command_tree$' --output-on-failure`

## 4. Layers panel row menu

- [x] 4.1 Add a `SmartObject` kind mask and the `Reset Transform` / `Convert to Layers` / `New Smart Object via Copy` rows to `layers_panel_menu.cpp`, thread a `smart` flag from `SmartObjectRole` through `populateRowMenu`, and dispatch the three rows to the smart-object bridge; verify with `ctest --test-dir build -R '^tst_layers_panel$' --output-on-failure`
- [x] 4.2 Qt Test: add `tst_smart_object_actions.cpp` (registered in `crates/pictura-app/cpp/tests/CMakeLists.txt`) covering the smart-object row menu (`smartRowMenu*ForTest`) and the three commands' one-undo-state behavior; verify with `ctest --test-dir build -R '^tst_command_tree$|^tst_layers_panel$|^tst_smart' --output-on-failure`

## 5. Verification

- [x] 5.1 Run `bash scripts/verify-fast.sh` green (fmt, clippy, test-report, file-size, guard, openspec) and `openspec validate --all --strict`; archive the change with `openspec archive smart-object-advanced-actions -y`
