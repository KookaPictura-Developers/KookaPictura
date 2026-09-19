## 1. Engine accessor

- [x] 1.1 Add `pub fn smart_object_source_bytes(doc: &Document, path: &str) -> Option<Vec<u8>>` to `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs`. Resolve `path`; return `Some(payload.clone())` only when the layer is not a group, has no adjustment data, and its `smart_object` has a non-empty payload; otherwise return `None`. Do not mutate.
- [x] 1.2 Register/re-export `smart_object_source_bytes` from `layer_ops/mod.rs`, `document_ops/mod.rs`, and `pictura-render/src/lib.rs`, matching the other smart-object ops.

## 2. Engine tests

- [x] 2.1 In `crates/pictura-render/src/tests/smart_object.rs`, place a written solid-colour PSD with `place_smart_object` and assert `smart_object_source_bytes` returns the exact placed bytes.
- [x] 2.2 Assert `None` for a non-smart layer, a group, an adjustment layer, a smart object with an empty payload, and an unresolved path.
- [x] 2.3 Assert the document is unchanged by an accessor call (compare against a pre-call clone).

## 3. App command and bridge

- [x] 3.1 Add `inline constexpr char LayerSmartObjectExportContents[] = "layer.smartObject.exportContents";` to `crates/pictura-app/cpp/commands.h`.
- [x] 3.2 In `crates/pictura-app/cpp/command_tree.cpp`, replace the disabled `{Layer, Smart Objects, Export Contents…}` leaf with `registry.add(command_ids::LayerSmartObjectExportContents, {"Layer", "Smart Objects", "Export Contents…"}, ...)`, `implemented = true`; leave the other Smart Objects placeholders disabled.
- [x] 3.3 Declare `layer_can_export_smart_object_contents(&self, path: &QString) -> bool` (read-only) and `export_smart_object_contents(&self, path: &QString, dest: &QString) -> bool` on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`.
- [x] 3.4 Implement both in `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs`. The predicate consults `smart_object_source_bytes`; the export method reads the payload through the accessor and writes it with `std::fs::write`, returning true only on a successful write. Record no history state, clear no link sets, and do not recomposite.
- [x] 3.5 In `crates/pictura-app/cpp/frame_menus.cpp`, add a `QFileDialog::getSaveFileName` handler filtered to `Photoshop files (*.psd)` at a `*.psd` default, calling `export_smart_object_contents`, and an enabled provider consulting `layer_can_export_smart_object_contents`.

## 4. C++ self-test

- [x] 4.1 Add `runLayersExportSmartObjectChecks` to `crates/pictura-app/cpp/selftest_layers_smart_object.{cpp,h}` using exit code **282** (277-281 are used): place or convert a smart object, export its contents to a temp path, and assert the file exists and its bytes parse with the codec (or match a recorded hash).
- [x] 4.2 Assert a non-smart layer refuses and that the document's history count is unchanged by the export; clean up the temp file.
- [x] 4.3 Call the new check from `crates/pictura-app/cpp/selftest_layers_controls.cpp` after `runLayersOpenSmartObjectChecks`, and keep `selftest_layers_smart_object.cpp` under its file-size cap.

## 5. Gates

- [x] 5.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 5.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`; record counts.
- [x] 5.4 `openspec validate export-smart-object-contents --strict` and `openspec validate --all --strict`.
