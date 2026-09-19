## 1. Engine operation

- [x] 1.1 Add `pub fn replace_smart_object_contents(doc: &mut Document, path: &str, filename: &str, bytes: &[u8]) -> bool` to `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs`. Resolve `path`; return `false` without mutating unless the layer is a non-group, non-adjustment layer whose `smart_object` is `Embedded` with a payload.
- [x] 1.2 Decode `bytes` with `pictura_codec::read_psd`; return `false` without mutating when they do not parse.
- [x] 1.3 On success set `smart_object.payload = Some(bytes.to_vec())`, `filename`, `filetype = *b"8BPB"`, `creator = *b"8BIM"`, and clear `uuid`.
- [x] 1.4 Clear the layer's pixel channels so the new source renders through the embedded-source path scaled into the existing `rect`.
- [x] 1.5 Remove `SoLd`/`SoLE`/`plLd`/`PlLd` from `layer.extra_blocks`; when the old uuid was non-empty call `pictura_codec::remove_linked_source(&doc.layer_section_extra, &old_uuid)` and replace the section only on `Some`. Return `true`.
- [x] 1.6 Register/re-export `replace_smart_object_contents` from `layer_ops/mod.rs`, `document_ops/mod.rs`, and `pictura-render/src/lib.rs`, matching `convert_to_smart_object`.

## 2. Engine tests

- [x] 2.1 In `crates/pictura-render/src/tests/smart_object.rs`, read `assets/test_with_smart_object01.psd` (self-skip when absent), replace its embedded source with a written solid-colour PSD, and assert the payload and filename changed and `uuid` is empty.
- [x] 2.2 Assert `extra_blocks` has no `SoLd`-family block, the old uuid is gone from `layer_section_extra`, and the pre-existing uuid was non-empty.
- [x] 2.3 Assert `rect`, name, blend, opacity, and mask are unchanged.
- [x] 2.4 Assert `write_psd`→`read_psd` resolves the NEW payload.
- [x] 2.5 Add refusal tests (non-smart layer, group, adjustment, malformed bytes, bad path) that assert `false` and a pre-call clone equality.

## 3. App command and bridge

- [x] 3.1 Add `inline constexpr char LayerSmartObjectReplaceContents[] = "layer.smartObject.replaceContents";` to `crates/pictura-app/cpp/commands.h`.
- [x] 3.2 In `crates/pictura-app/cpp/command_tree.cpp`, replace the disabled `{Layer, Smart Objects, Replace Contents…}` leaf with `registry.add(command_ids::LayerSmartObjectReplaceContents, {"Layer", "Smart Objects", "Replace Contents…"}, ...)`, `implemented = true`; leave the other Smart Objects placeholders disabled.
- [x] 3.3 Declare `layer_can_replace_smart_object_contents(&self, path: &QString) -> bool` (read-only) and `replace_smart_object_contents(self: Pin<&mut Self>, path: &QString, file_path: &QString) -> bool` on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`.
- [x] 3.4 Implement both in `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs`. The command method reads the file, derives the display name from its base name, calls `pictura_render::replace_smart_object_contents`; on success only calls `clear_link_sets()`, `recomposite()`, and `record("Replace Contents")`; a refusal records nothing.
- [x] 3.5 In `crates/pictura-app/cpp/frame_menus.cpp`, add a `QFileDialog::getOpenFileName` handler filtered to `Photoshop files (*.psd *.psb)` (refresh on success) and an enabled provider consulting the read-only predicate, mirroring the Convert/Place commands.

## 4. C++ self-test

- [x] 4.1 Extend `crates/pictura-app/cpp/selftest_layers_controls.cpp`: place or convert a smart object, replace its contents with a written solid-colour PSD, and assert the composite changes and the layer reports a non-empty embedded payload.
- [x] 4.2 Assert exactly one history state labelled by the replace command on success, and that a malformed file refuses with no history state and an unchanged document.
- [x] 4.3 Use the next free exit code (280, after the used 279) and `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`; call the check from `runSelfTest`. The check was later extracted (with 277-279) into `selftest_layers_smart_object.{cpp,h}`, added to `CMakeLists.txt`, as a pure move; both files stay under the file-size cap.

## 5. Gates

- [x] 5.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 5.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`; record counts.
- [x] 5.4 `openspec validate replace-smart-object-contents --strict` and `openspec validate --all --strict`.
