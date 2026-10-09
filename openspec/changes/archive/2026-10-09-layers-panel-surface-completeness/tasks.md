# Tasks

## 1. OpenSpec

- [x] 1.1 Validate the change: `openspec validate layers-panel-surface-completeness --strict`.

## 2. Bridge: smart-object role, panel options, group lock

- [x] 2.1 Add `src/cxxqt_object/layers_surface.rs` (a secondary cxx-qt bridge with
  its own generated header, registered in `build.rs`) exposing
  `layer_row_is_smart_object(view, i)`, `set_layers_panel_options(view, add_copy,
  use_default_masks)`, and `lock_group_layers_run(view, group_path)`. Verify:
  `cargo nextest run -p pictura_app`.
- [x] 2.2 Add `add_copy` and `use_default_masks` (default `true`) to
  `PictureViewRust`; `duplicate_layer`/`duplicate_layers` strip the engine's
  `" copy"` suffix when `add_copy` is off; `add_adjustment` honours
  `use_default_masks`; `add_solid_fill`/`add_gradient_fill` assign a
  `selection_to_mask` when the flag is on. Rust bridge tests.
- [x] 2.3 Engine `lock_group_layers(doc, path)` (descendant batch) with unit
  tests; the bridge free function delegates to it. Verify:
  `cargo nextest run -p pictura-render lock_group`.

## 3. Panel: chip, badge, options, Tab navigation

- [x] 3.1 `LayerRow` gains `smartObject`; add `SmartObjectRole`; `refresh()`
  reads `layer_row_is_smart_object`; the delegate paints a color-label chip and
  the `layers.kindSmartObject` lower-right badge. Verify:
  `ctest --test-dir build -R '^tst_layers_panel$'`.
- [x] 3.2 Add `layersAddCopyOnDuplicate` / `layersUseDefaultMasksOnFill` to
  `SessionState` (`session.{h,cpp}`); add the two checkboxes to `openPanelOptions`
  and `setOptionsForTest` test hook; push both to the view via
  `set_layers_panel_options` in `setView` and after the dialog.
- [x] 3.3 `LayerRowDelegate::eventFilter` handles `Tab`/`Backtab` on its editor
  (commit, then `closeEditor(EditNextItem|EditPreviousItem)` when a neighbour
  exists, `NoHint` at the ends so there is no wrap). Qt Test cases.
- [x] 3.4 Add Qt Test cases to `tst_layers_panel.cpp`: label chip rect/color, the
  smart-object badge role, the two panel options (persist + duplicate name +
  default mask), and Tab/Shift+Tab rename navigation.

## 4. Lock commands

- [x] 4.1 Add the five frozen ids to `commands.h`; add the `Layer > Lock Layers`
  submenu and convert `Lock All Layers In Group…` in `command_tree.cpp`.
- [x] 4.2 Wire handlers/enabled-providers in `frame_menus_layer_ops.cpp` over
  the panel selection and current group path. Qt Test case in
  `tst_command_tree.cpp`.

## 5. Verify and archive

- [x] 5.1 `cargo nextest run -p pictura-render -p pictura_app`;
  `cargo clippy -p pictura-render -p pictura_app --all-targets -- -D warnings`;
  `cargo fmt --all --check`.
- [x] 5.2 `cmake --build build --parallel`;
  `ctest --test-dir build -R '^tst_layers_panel$|^tst_command_tree$|^tst_properties_panel$' --output-on-failure`.
- [x] 5.3 `bash scripts/verify-fast.sh`; `openspec validate --all --strict`.
- [x] 5.4 Archive with `openspec archive layers-panel-surface-completeness -y`
  and re-validate all specs.
