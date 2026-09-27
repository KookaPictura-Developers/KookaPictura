# Tasks: edit-clipboard

## 1. Engine

- [x] 1.1 Add `layer_ops/clipboard.rs`: `Clip`, `PasteMode`, `copy_layer`, `copy_merged`, `clear_layer`, `paste_clip`, `coverage_bounds`; make `insert_node` `pub(super)`; re-export through `layer_ops/mod.rs`, `document_ops/mod.rs`, and `lib.rs`.
- [x] 1.2 Engine tests (`tests_clipboard.rs`): bounding-box copy, whole-layer copy clipped to the canvas, refusals, grayscale replication, Copy Merged planes, coverage-scaled clear and no-op, locks, Background clear, plain / Into / Outside paste, cut-then-paste round trip, malformed clip, `coverage_bounds`.

## 2. Bridge

- [x] 2.1 Add the `cxxqt_object/clipboard.rs` bridge (`PasteKind`, `clipboard_copy/cut/clear/paste/has_contents/purge`) with a shared process-wide clipboard; register it in `build.rs` and `cxxqt_object.rs` without growing the ceiling file.

## 3. App commands

- [x] 3.1 Add `command_ids::EditCut/EditCopy/EditCopyMerged/EditPaste/EditPasteInto/EditPasteOutside/EditClear/EditPurgeClipboard` and promote the leaves in `command_tree.cpp`.
- [x] 3.2 Add `frame_menus_edit.cpp` (`registerEditHandlers`) with handlers and enabled providers; register it in `CMakeLists.txt`.

## 4. C++ self-test

- [x] 4.1 Append `edit_clipboard` (code 529) in `selftest_clipboard.{h,cpp}`: Copy records nothing; Cut, Paste Into, Clear, and Paste record one labelled state each; Paste Into restores the cut pixels with a mask and deselects; Purge empties the clipboard and disables Paste.

## 5. Verification

- [x] 5.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, workspace tests, `openspec validate --all --strict`, file-size check.
- [x] 5.2 CMake build + `./build/pictura --headless --self-test`.
