# Tasks: properties-panel

## 1. Panel

- [x] 1.1 `PropertiesPanel` widget with `setView(PictureView*)` and `refresh()`.
- [x] 1.2 Empty state "No Properties" without a document or without an active adjustment layer.
- [x] 1.3 Name the active adjustment layer, found by matching `active_layer_path()` against `layer_row_path(i)`.

## 2. Wiring

- [x] 2.1 Replace the Properties `PlaceholderPanel` with `PropertiesPanel`, keeping `objectName`.
- [x] 2.2 Register the `.cpp`/`.h` pair in the root `CMakeLists.txt` `pictura_shell` lists.
- [x] 2.3 Refresh from `retargetDock()`.
- [x] 2.4 Add `tst_properties_panel` to `PICTURA_QT_TESTS`.

## 3. Verification

- [x] 3.1 `tst_properties_panel` covers the plain-document empty state and the named adjustment.
- [x] 3.2 `cmake --build build --parallel` and `ctest --test-dir build -R '^tst_'`.
