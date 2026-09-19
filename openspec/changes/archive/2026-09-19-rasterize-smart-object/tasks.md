## 1. Codec linked-source removal

- [x] 1.1 Add `pub fn remove_linked_source(layer_section_extra: &[u8], uuid: &str) -> Option<Vec<u8>>` to `crates/pictura-codec/src/smart_object.rs` (or a new `linked.rs` if the file's ceiling requires it). Walk the `8BIM` tagged blocks with the same length/even-padding rules as `collect_linked_records`.
- [x] 1.2 For each `lnkD`/`lnk2`/`lnk3`/`lnkE` block, parse the `u64`-length-prefixed record list, drop the record(s) whose Pascal uuid equals `uuid`, and rebuild the block via `write_tag` (same key, recomputed length, even padding). Copy every non-linked block byte-for-byte.
- [x] 1.3 Return `Some(new_bytes)` when at least one record was removed, `None` when nothing matched; a malformed block is copied verbatim and never panics.
- [x] 1.4 Re-export `remove_linked_source` from `crates/pictura-codec/src/lib.rs`.

## 2. Engine operation

- [x] 2.1 Add `pub fn rasterize_smart_object(doc: &mut Document, path: &str) -> bool` to `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs`. Resolve `path` with `resolve_path_mut`; return `false` without mutating unless the layer is a non-group, non-adjustment layer with `smart_object.is_some()`.
- [x] 2.2 Materialize the content. If the layer has a color channel (`id == 0`), leave its pixel channels unchanged. Otherwise decode the embedded payload through the existing embedded-source render path, scale the decoded source into the layer `rect`, and write channels `0..mode.color_channels()` plus a `-1` alpha channel. Return `false` without mutating when the payload is empty or cannot be decoded.
- [x] 2.3 Clear `layer.smart_object = None`.
- [x] 2.4 Remove the layer's preserved config block: retain `extra_blocks` entries whose key is not `SoLd`/`SoLE`/`plLd`/`PlLd`.
- [x] 2.5 Remove the matching document-level record: call `pictura_codec::remove_linked_source(&doc.layer_section_extra, &uuid)` and replace `doc.layer_section_extra` only when it returns `Some`. Return `true`.
- [x] 2.6 Register/re-export `rasterize_smart_object` from `layer_ops/mod.rs`, `document_ops/mod.rs`, and `pictura-render/src/lib.rs`, matching `convert_to_smart_object`.

## 3. Engine tests

- [x] 3.1 In `crates/pictura-render/src/tests/smart_object.rs`, add a test that converts a raster layer, rasterizes it, and asserts: channels are byte-for-byte unchanged, `smart_object` is `None`, and `SoLd` is gone from `extra_blocks`.
- [x] 3.2 Add a source-only test: a channel-less embedded smart-object layer rasterizes from the payload, writing color channels and `-1` alpha scaled into the layer `rect`.
- [x] 3.3 Add refusal tests: a non-smart layer, a group, an adjustment layer, an unresolved path, and a channel-less layer with an empty/undecodable payload each return `false` and leave a pre-call clone equal.
- [x] 3.4 Add a round-trip test: after rasterizing a layer that carried a preserved `SoLd` plus a document-level `lnk2` record, `write_psd` then `read_psd` resolves no smart object and carries no orphan `lnk*` record.

## 4. Codec unit test

- [x] 4.1 In `smart_object.rs` / `linked.rs`, build a section with a `lnk2` holding two records plus an unrelated tagged block. Remove one uuid with `remove_linked_source` and assert `Some`; the survivor and the unrelated block are byte-preserved and the removed uuid is absent.
- [x] 4.2 Assert a non-matching uuid returns `None`, and that a malformed block is left verbatim without panicking.

## 5. App command and bridge

- [x] 5.1 Add `inline constexpr char LayerRasterizeSmartObject[] = "layer.rasterize.smartObject";` to `crates/pictura-app/cpp/commands.h`, next to the existing Rasterize ids.
- [x] 5.2 In `crates/pictura-app/cpp/command_tree.cpp`, replace the disabled `{Layer, Rasterize, Smart Object}` leaf with `registry.add(command_ids::LayerRasterizeSmartObject, {"Layer", "Rasterize", "Smart Object"}, ...)`, `implemented = true`; leave the other Rasterize placeholders disabled.
- [x] 5.3 Declare `layer_can_rasterize_smart_object(&self, path: &QString) -> bool` (read-only) and `rasterize_smart_object(self: Pin<&mut Self>, path: &QString) -> bool` on `PictureView` in `crates/pictura-app/src/cxxqt_object.rs`.
- [x] 5.4 Implement both in `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs`. The command method calls `pictura_render::rasterize_smart_object`; on success only calls `clear_link_sets()`, `recomposite()`, and `record("Rasterize Smart Object")`; a refusal records nothing.
- [x] 5.5 In `crates/pictura-app/cpp/frame_menus.cpp`, add the handler (refresh on success) and the enabled provider consulting the read-only predicate, mirroring the Convert command.

## 6. C++ self-test

- [x] 6.1 Extend `crates/pictura-app/cpp/selftest_layers_controls.cpp`: create a document with a raster layer, convert it, rasterize it, and assert the layer no longer reports a smart object.
- [x] 6.2 Assert the composite is unchanged versus before rasterizing, and that save→load leaves no smart object.
- [x] 6.3 Assert refusal on a non-smart layer with no history state recorded.
- [x] 6.4 Use the next free exit code (278, after the used 277) and `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`; call the check from `runSelfTest`. No new `.cpp`/`.h`, so `CMakeLists.txt` is unchanged; keep the file within its `scripts/file-size-allowlist.txt` ceiling.

## 7. Gates

- [x] 7.1 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 7.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 7.3 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`; record counts (verify-full OK; `cargo nextest --workspace` 727 passed / 8 skipped; self-test SUMMARY passed=214 failed=0 skipped=0).
- [x] 7.4 `openspec validate rasterize-smart-object --strict` and `openspec validate --all --strict`.
