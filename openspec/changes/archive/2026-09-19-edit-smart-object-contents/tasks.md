## 1. Bridge: eligibility and shared source apply

- [x] 1.1 In `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs`, factor the body shared by `replace_smart_object_contents` into a private helper `apply_smart_object_source(self, path, file_path, label) -> bool`: read the file, derive the display name from its base name, call `pictura_render::replace_smart_object_contents`, and on success `clear_link_sets()`, `recomposite()`, and `record(label)`. Make `replace_smart_object_contents` delegate with `"Replace Contents"` so its signature, behavior, and label are unchanged.
- [x] 1.2 Add `commit_smart_object_edit(self: Pin<&mut Self>, path: &QString, file_path: &QString) -> bool` delegating to the same helper with `"Edit Contents"`.
- [x] 1.3 Add the read-only predicate `layer_can_edit_smart_object_contents(&self, path: &QString) -> bool` returning true only when `pictura_render::smart_object_source_bytes(doc, path)` is `Some` AND `pictura_codec::read_psd(&bytes).is_ok()`. It must mutate nothing.
- [x] 1.4 Declare all three methods on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs` with the surrounding `#[qinvokable]` doc comments.

## 2. Frame: session, entry point, Save interception, cleanup

- [x] 2.1 In `crates/pictura-app/cpp/frame.h`, add the private `struct SmartObjectEditSession { PictureView* editor; PictureView* origin; QString layerPath; QString filename; QTemporaryDir* temp; }`, a `QList<SmartObjectEditSession> editSessions_;` member, and forward-declare `class QTemporaryDir;`.
- [x] 2.2 Declare `bool editSmartObjectContents(const QString& layerPath);` in the public document-operations block of `frame.h`, beside `openAsSmartObjectPath`.
- [x] 2.3 Implement it in `frame.cpp`: require `layer_can_edit_smart_object_contents` for `layerPath`, create a `QTemporaryDir` and a temp PSD path, `origin->export_smart_object_contents(layerPath, tempFile)`, construct a `PictureView`, `editor->open(tempFile)`, `addDocument(editor, QString())`, append the session, and return the result. Record no origin history state. Delete the view/temp and return false on any failure. Include `<QtCore/QTemporaryDir>` and `<QtCore/QDir>` as needed.
- [x] 2.4 Intercept Save at the top of `PicturaMainWindow::saveActive()`: if the active view is a session editor, call `editor->save(tempFile)`, then `origin->commit_smart_object_edit(layerPath, tempFile)`, `refresh()`, and return true; never open a dialog and never call `saveActiveAs`. Leave `saveActiveAs` unchanged.
- [x] 2.5 In `removeDocument` (the single cleanup point called by `closeDocument`), drop sessions whose `editor` is the removed view (delete the `QTemporaryDir`) and sessions whose `origin` is the removed view (leave the orphaned editor open, now untitled). Ensure the destructor cleans up any remaining sessions.

## 3. Command wiring

- [x] 3.1 Add `inline constexpr char LayerSmartObjectEditContents[] = "layer.smartObject.editContents";` to `crates/pictura-app/cpp/commands.h`.
- [x] 3.2 In `crates/pictura-app/cpp/command_tree.cpp:379`, replace the disabled `{Layer, Smart Objects, Edit Contents}` leaf with `registry.add(command_ids::LayerSmartObjectEditContents, {"Layer", "Smart Objects", "Edit Contents"}, QStringLiteral("Edit Contents"), QKeySequence(), true);`.
- [x] 3.3 In `crates/pictura-app/cpp/frame_menus.cpp`, add a `currentEditableSmartPath` lambda mirroring `currentReplaceableSmartPath` that returns `layersPanel_->currentPath()` only when `view->layer_can_edit_smart_object_contents(path)`; set the command handler to call `editSmartObjectContents(path)` and `refresh()`, and set the enabled provider to require a non-empty resolved path. No file dialog.

## 4. Tests

- [x] 4.1 In `crates/pictura-app/src/cxxqt_object/tests.rs`, add a Rust test for the pure eligibility predicate `can_edit_smart_object_contents`: an `Embedded` PSD/PSB payload is editable; a non-parsing payload, an empty/absent payload, a group, an adjustment layer, a non-`Embedded` kind, and an unresolved path are refused; and the predicate mutates nothing. The `commit_smart_object_edit` bridge cannot be driven from Rust (`PictureView` is a C++-constructed QObject; cxx-qt 0.10 exposes no Rust constructor), so it is verified end-to-end by C++ self-test 288 plus the engine `replace_smart_object_contents` tests it calls unchanged.
- [x] 4.2 Extend `crates/pictura-app/cpp/selftest_layers_smart_object.cpp` with a new check `lpr_edit_smart_object_contents` using the next free exit code **288** (287 is the current maximum). Create an origin with a converted smart object, call `frame.editSmartObjectContents(layerPath)`, and assert one new tab exists, is untitled, holds the decoded source as an ordinary document, and leaves the origin history unchanged.
- [x] 4.3 In the same check: modify the editor through an existing bridge call, set it active, call `frame.saveActive()`, and assert the origin records exactly one `"Edit Contents"` state and its layer renders the new source. Then open a second editor, modify it, close it without saving, and assert the origin is unchanged.
- [x] 4.4 Assert an ineligible target (non-smart layer / non-PSD payload) is refused and adds no tab. Clean up temp files. Call the new check from `selftest_layers_controls.cpp`/`runSelfTest` where the sibling smart-object checks are invoked.
- [x] 4.5 Add a second check `lpr_edit_smart_object_session` with the next free exit code **289** (288 is now the maximum): closing an editor deletes its session temporary file; closing the origin leaves the orphaned editor open as an untitled tab (`documentPath` empty, no crash) and drops the session temp. Declare and wire it beside the other smart-object checks.

## 5. Gates

- [x] 5.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 5.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`; record counts.
- [x] 5.4 `openspec validate edit-smart-object-contents --strict` and `openspec validate --all --strict`.
