## 1. Engine operation

- [x] 1.1 Add `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` with `pub fn convert_to_smart_object(doc: &mut Document, path: &str) -> bool`. Resolve `path` with `resolve_path_mut`; return `false` without mutating unless the layer is a non-group raster pixel layer (`adjustment.is_none()`, `!background`, `smart_object.is_none()`, `rect.width() > 0 && rect.height() > 0`).
- [x] 1.2 Build the embedded source: `Document::new(rect.width(), rect.height(), doc.mode, doc.depth)` containing a copy of the target layer translated to `(0, 0)` with its channels, and a merged composite equal to the layer's raster over that rect. Serialize it with `pictura_codec::write_psd`; return `false` without mutating if serialization fails.
- [x] 1.3 Attach `SmartObject { kind: SmartObjectKind::Embedded, payload: Some(bytes), filename: format!("{}.psd", layer.name), filetype: *b"8BPB", creator: *b"8BIM", ..Default::default() }`. Keep the layer's existing pixel channels (the raster proxy). Return `true`.
- [x] 1.4 Register the module in `layer_ops/mod.rs` and re-export `convert_to_smart_object` from `document_ops/mod.rs` and `pictura-render/src/lib.rs`, matching `rasterize_fill_content`.

## 2. Engine tests

- [x] 2.1 In `crates/pictura-render/src/tests/smart_object.rs`, add an op-level test: convert a raster layer, assert the returned bool, the `Embedded` kind, the non-empty payload, and `filename`/`filetype`/`creator`.
- [x] 2.2 Assert the pixel channels are byte-for-byte unchanged and `composite_rgba` before/after conversion is equal.
- [x] 2.3 Assert the payload parses with `pictura_codec::read_psd`, matches the layer rect, resolves as `Embedded`, and that the source document's merged composite equals the layer raster.
- [x] 2.4 Add refusal tests: a group, an adjustment layer, the Background, an already-smart layer, a zero-size `rect`, and a path that does not resolve — each returns `false` and leaves the document equal to a clone taken before the call.

## 3. App command and bridge

- [x] 3.1 Add `inline constexpr char LayerSmartObjectConvertTo[] = "layer.smartObject.convertTo";` to `crates/pictura-app/cpp/commands.h`.
- [x] 3.2 In `crates/pictura-app/cpp/command_tree.cpp`, replace the disabled Smart Objects `"Convert to Smart Object"` leaf with `registry.add(command_ids::LayerSmartObjectConvertTo, {"Layer", "Smart Objects", "Convert to Smart Object"}, QStringLiteral("Convert to Smart Object"), QKeySequence(), true);`. Leave the other Smart Objects leaves as placeholders.
- [x] 3.3 Declare the bridge methods on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`: `can_convert_to_smart_object(&self, path: &QString) -> bool` (read-only) and `convert_to_smart_object(self: Pin<&mut Self>, path: &QString) -> bool`.
- [x] 3.4 Implement both in a new `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs` (declared in `impl_layers` module list). The command method calls `pictura_render::convert_to_smart_object`; on success only it calls `clear_link_sets()`, `recomposite()`, and `record("Convert to Smart Object")`; a refusal records nothing.
- [x] 3.5 In `crates/pictura-app/cpp/frame_menus.cpp`, add the handler (refresh on success) and the enabled provider consulting `can_convert_to_smart_object(currentPath)`, mirroring the Rasterize/Background commands.

## 4. C++ self-test

- [x] 4.1 Extend the existing `crates/pictura-app/cpp/selftest_layers_controls.cpp` with the check: create a document with a raster layer, convert it, and assert the layer reports a smart object.
- [x] 4.2 Assert the composite is unchanged versus before conversion (proxy intact), then assert save→load preserves the object: resolved kind `Embedded` with a non-empty payload.
- [x] 4.3 Assert refusal for a group and for a Background layer, with no history state recorded.
- [x] 4.4 Use the next free exit code (277, after `runToolsSelectionChecks`' 276) and `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`; call the new `run*Checks` from `runSelfTest`.
- [x] 4.5 No new `.cpp`/`.h` file was added (the check extends the existing suite), so `CMakeLists.txt` is unchanged; keep the file within its `scripts/file-size-allowlist.txt` ceiling.

## 5. Gates

- [x] 5.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 5.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 5.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`; record counts (verify-full OK; `cargo nextest --workspace` 719 passed / 8 skipped; self-test SUMMARY passed=213 failed=0 skipped=0).
- [x] 5.4 `openspec validate convert-to-smart-object --strict` and `openspec validate --all --strict`.
