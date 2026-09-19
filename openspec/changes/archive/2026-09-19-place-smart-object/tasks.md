## 1. Engine operation

- [x] 1.1 Add `pub fn place_smart_object(doc: &mut Document, filename: &str, bytes: &[u8]) -> Option<String>` to `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs`. Decode `bytes` with `pictura_codec::read_psd`; return `None` without mutating when it fails.
- [x] 1.2 Append a new topmost layer named `filename` with `rect = (0, 0, decoded.width, decoded.height)`, no pixel channels, `visible`, `BlendMode::Normal`, `opacity = 255`, and `smart_object = Some(SmartObject { kind: Embedded, payload: Some(bytes.to_vec()), filename: filename.to_string(), filetype: *b"8BPB", creator: *b"8BIM", ..Default::default() })`.
- [x] 1.3 Return `Some(path)` for the new layer using the existing path-string convention; register the function in `layer_ops/mod.rs` and re-export it from `document_ops/mod.rs` and `lib.rs` beside `convert_to_smart_object`/`rasterize_smart_object`.

## 2. Engine tests

- [x] 2.1 In `crates/pictura-render/src/tests/smart_object.rs`, add a place test: write a solid-colour PSD, place it, assert `Some(path)`, a new topmost layer named after the source, `rect` equal to the source size, no pixel channels, and an `Embedded` smart object with the payload and filename.
- [x] 2.2 Assert `composite_rgba` shows the source colour over the layer rect and clips content beyond the canvas.
- [x] 2.3 Assert `write_psd` then `read_psd` re-resolves the placed layer as an embedded smart object with an unchanged payload.
- [x] 2.4 Add a refusal test: malformed bytes return `None` and leave the document equal to a clone taken before the call.

## 3. App command and bridge

- [x] 3.1 Add `inline constexpr char FilePlace[] = "file.place";` to `crates/pictura-app/cpp/commands.h`.
- [x] 3.2 In `crates/pictura-app/cpp/command_tree.cpp`, replace the disabled `{File, Place…}` leaf with `registry.add(command_ids::FilePlace, {"File", "Place…"}, QStringLiteral("Place…"), QKeySequence(), true);`.
- [x] 3.3 Declare `fn place_smart_object(self: Pin<&mut Self>, path: &QString) -> QString;` on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`.
- [x] 3.4 Implement it in `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs`: read the file with `std::fs::read` (mirror `impl_core.rs::open`), derive the display name from the file's base name, and call the engine op on the current document. On success only call `clear_link_sets()`, `recomposite()`, and `record("Place")`, returning the new layer path; on refusal return an empty `QString` and record nothing.
- [x] 3.5 In `crates/pictura-app/cpp/frame_menus.cpp`, add the handler: open a `QFileDialog::getOpenFileName` filtered to `"Photoshop files (*.psd *.psb)"` (mirror `showOpenDialog` in `frame.cpp`), call the bridge on a non-empty path, and refresh on success. Add an enabled provider requiring an open document.

## 4. C++ self-test

- [x] 4.1 Extend `crates/pictura-app/cpp/selftest_layers_controls.cpp`: write a small PSD to a temp path, place it, and assert a new smart-object layer appears at the top and the composite changes.
- [x] 4.2 Assert a malformed file refuses: the bridge returns empty and no history state is recorded.
- [x] 4.3 Use the next free exit code 279 (277 and 278 are taken), with `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`, and keep the file within its `scripts/file-size-allowlist.txt` ceiling. No new `.cpp`/`.h`, so `CMakeLists.txt` is unchanged.

## 5. Gates

- [x] 5.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 5.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`.
- [x] 5.4 `openspec validate place-smart-object --strict` and `openspec validate --all --strict`.
