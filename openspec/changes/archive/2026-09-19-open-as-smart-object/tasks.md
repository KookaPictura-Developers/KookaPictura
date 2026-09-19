## 1. Engine operation

- [x] 1.1 Add `pub fn open_as_smart_object(filename: &str, bytes: &[u8]) -> Option<Document>` to `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs`. Decode `bytes` with `pictura_codec::read_psd`; return `None` when it fails.
- [x] 1.2 Build `Document::new(src.width, src.height, src.mode, src.depth)`, copy the decoded source's merged composite into the new document's composite, then call the existing `place_smart_object(&mut doc, filename, bytes)` to append exactly one topmost embedded, channel-less, native-size smart-object layer.
- [x] 1.3 Return `Some(doc)`; register the function in `layer_ops/mod.rs` and re-export it from `document_ops/mod.rs` and `lib.rs` beside `place_smart_object`.

## 2. Engine tests

- [x] 2.1 In `crates/pictura-render/src/tests/smart_object.rs`, add an open test: write a solid-colour PSD, open it as a smart object, assert `Some(doc)` with the source dimensions and mode, exactly one layer, and an `Embedded` smart object whose payload equals the source bytes and whose name is the supplied display name.
- [x] 2.2 Assert `composite_rgba` shows the source colour.
- [x] 2.3 Add a refusal test: malformed bytes return `None`.

## 3. App command and bridge

- [x] 3.1 Add `inline constexpr char FileOpenAsSmartObject[] = "file.openAsSmartObject";` to `crates/pictura-app/cpp/commands.h`.
- [x] 3.2 In `crates/pictura-app/cpp/command_tree.cpp`, replace the disabled `{File, Open As Smart Object…}` leaf with `registry.add(command_ids::FileOpenAsSmartObject, {"File", "Open As Smart Object…"}, QStringLiteral("Open As Smart Object…"), QKeySequence(), true);`.
- [x] 3.3 Declare `fn open_as_smart_object(self: Pin<&mut Self>, path: &QString) -> bool;` on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`.
- [x] 3.4 Implement it in `crates/pictura-app/src/cxxqt_object/impl_core.rs`: read the file with `std::fs::read`, derive the display name from the base name, and build the document via `pictura_render::open_as_smart_object`. Return `false` before mutating on a failed read or decode; on success store the composited image and document, call `reset_edit_state()`, capture exactly one `"Open As Smart Object"` history state, set the view's path to NONE, and mark it clean.
- [x] 3.5 In `crates/pictura-app/cpp/frame_menus.cpp`, add the handler: open a `QFileDialog::getOpenFileName` filtered to `"Photoshop files (*.psd *.psb)"`, call the bridge on a non-empty path, and mirror `openPath` by constructing a `PictureView`, calling the bridge, and `addDocument(view, QString())` on success or deleting the view on failure. No enabled provider (always enabled).
- [x] 3.6 Declare `bool openAsSmartObjectPath(const QString& path);` in `crates/pictura-app/cpp/frame.h` beside `openPath` and implement it in `crates/pictura-app/cpp/frame.cpp`.

## 4. C++ self-test

- [x] 4.1 Extend the existing `crates/pictura-app/cpp/selftest_layers_smart_object.{h,cpp}` (no new file, so `CMakeLists.txt` is unchanged). Write a small solid-colour PSD to a temp path, open it as a smart object via the frame, and assert exactly one layer that reports a smart object, the composite is non-empty and changed from blank, and the tab is untitled (empty path).
- [x] 4.2 Assert a malformed file fails and adds no document.
- [x] 4.3 Clean up temp files; use the next free exit code 281 (277-280 are taken) with `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`, and keep the existing file under the size cap.

## 5. Gates

- [x] 5.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.2 `cargo nextest run --workspace` (738 passed / 8 skipped) and `cargo test --workspace --doc`.
- [x] 5.3 `bash scripts/verify-full.sh` (OK; TOTAL 993 passed / 9 skipped; self-test SUMMARY passed=217 failed=0 skipped=0); build with CMake and run `./build/pictura --headless --self-test`.
- [x] 5.4 `openspec validate open-as-smart-object --strict` and `openspec validate --all --strict`.
