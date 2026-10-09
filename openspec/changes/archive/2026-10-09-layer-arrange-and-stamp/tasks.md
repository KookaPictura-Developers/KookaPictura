# Tasks

## 1. Engine: arrange, reverse, stamp

- [x] 1.1 Add `Arrange` enum, `arrange_path`, and `can_arrange_path` to `crates/pictura-render/src/document_ops/layer_ops/properties.rs`; export them from `layer_ops/mod.rs` and the crate root. Verify: `cargo nextest run -p pictura-render arrange reverse`.
- [x] 1.2 Add `reverse_paths` to `properties.rs` (contiguous single-container run; refuse Background/locked; reverse in place). Verify: unit tests in `layer_ops/tests.rs`.
- [x] 1.3 Add `StampScope` and `stamp_scope` to `layer_ops/merge.rs`, factor the shared visible-node gather out of `merge_visible`, and export from `mod.rs`/crate root. Verify: unit tests in `layer_ops/merge_tests.rs`.
- [x] 1.4 Add engine unit tests: arrange Front/Forward/Backward/Back within a group (not reparenting), boundary refusal, Background/locked refusal; reverse contiguous run and refusals; stamp visible/selected adds one layer while source count is unchanged and pixels match the composite. Verify: `cargo nextest run -p pictura-render`.

## 2. Bridge

- [x] 2.1 Declare and implement `arrange_layer(path, op)`, `can_arrange_layer(path, arrange)`, and `reverse_layers(paths)` on `PictureView` (each mutation `recomposite()` + one `record(...)`). Verify: `cargo nextest run -p pictura_app`.
- [x] 2.2 Declare and implement `merge_down(path)`, `stamp_visible(path)`, and `stamp_selected(paths, active)` reusing the merge scope and stamp engine op; one undo state each; empty/0 on refusal. Verify: `cargo nextest run -p pictura_app`.
- [x] 2.3 Add bridge tests covering arrange/reverse/merge-down/stamp (including stamp preserving the source layer count). Verify: `cargo nextest run -p pictura_app`.

## 3. C++ Layer menu wiring

- [x] 3.1 Add frozen ids to `cpp/commands.h`: `LayerArrangeFront`, `LayerArrangeForward`, `LayerArrangeBackward`, `LayerArrangeBack`, `LayerArrangeReverse`, `LayerMergeDown`, `LayerDeleteLayer`, `LayerStampVisible`, `LayerStampSelected`.
- [x] 3.2 Replace the greyed `Layer > Arrange` leaves in `cpp/command_tree.cpp` with `registry.add(command_ids::...)` rows, add `Arrange > Reverse`, add a distinct `Merge Down` row, convert `Delete Layer` to a frozen id, and add `Stamp Visible` / `Stamp Selected` rows with their shortcuts.
- [x] 3.3 Add handlers and enabled-providers in `cpp/frame_menus.cpp` for all nine ids, using the panel selection/current path; each handler refreshes on success. Verify: `cmake --build build --parallel`.

## 4. Qt Test + verification

- [x] 4.1 Add a Qt Test case in `crates/pictura-app/cpp/tests/tst_command_tree.cpp` (register in `CMakeLists.txt` if needed): the Arrange/Reverse/Merge Down/Delete Layer/Stamp leaves exist; enable where applicable; Stamp Visible adds exactly one layer while the source count is preserved; a hidden active layer disables Stamp Visible. Verify: `ctest --test-dir build -R '^tst_command_tree$' --output-on-failure`.
- [x] 4.2 Run the full gate set: `cargo nextest run -p pictura-render -p pictura_app`; `cargo clippy -p pictura-render -p pictura_app --all-targets -- -D warnings`; `cargo fmt --all --check`; `cmake --build build --parallel`; `ctest --test-dir build -R '^tst_command_tree$|^tst_layers_panel$' --output-on-failure`; `bash scripts/verify-fast.sh`; `openspec validate layer-arrange-and-stamp --strict`.
- [x] 4.3 Archive the change with `openspec archive layer-arrange-and-stamp -y` and re-validate all specs.
