# Proposal: edit-clipboard

## Why

Edit ▸ Cut, Copy, Copy Merged, Paste, Paste Special ▸ Paste Into / Paste
Outside, and Clear were disabled `leaf()` stubs, and there was no pixel
clipboard in the bridge (issue #88). photorust (perfecto25/photorust) ships a
working clipboard over its own document model; this change ports that behavior
onto Kooka's layer tree, selection, locks, and history.

## What Changes

- Add `pictura_render::{Clip, PasteMode, copy_layer, copy_merged, clear_layer,
  paste_clip, coverage_bounds}`: a clipboard that stores only the selection's
  bounding box (straight RGBA plus the selection coverage), copies from a layer
  or the visible composite, clears by coverage with lock checks, and pastes as a
  new raster layer, optionally masked by the selection (Into) or its inverse
  (Outside).
- Add a second cxx-qt bridge, `src/cxxqt_object/clipboard.rs`, exposing
  `clipboard_copy/cut/clear/paste/has_contents/purge` as free functions over a
  `PictureView`, with one process-wide clipboard shared by every open document.
  The main `PictureView` declaration file is at its size ceiling, so it does not
  grow.
- Promote the eight Edit leaves (including `Purge ▸ Clipboard`) to `command_ids`
  commands with handlers and enabled providers in a new `frame_menus_edit.cpp`.
- One C++ self-test check (`edit_clipboard`, code 529).
- Out of scope: the OS clipboard (Export Clipboard), Paste in Place, vector
  masks for Paste Into, and the background swatch for clearing a Background.

## Capabilities

### New Capabilities

- `document/edit-clipboard`: the Edit clipboard commands and their engine.

## Impact

- New `crates/pictura-render/src/document_ops/layer_ops/clipboard.rs` (+
  `tests_clipboard.rs`) and re-exports; `insert_node` becomes `pub(super)`.
- App: `build.rs`, `cxxqt_object.rs` (`mod clipboard;`),
  `cxxqt_object/clipboard.rs`, `commands.h`, `command_tree.cpp`, `frame.h`,
  `frame_menus.cpp`, new `frame_menus_edit.cpp`, new
  `selftest_clipboard.{h,cpp}`, `selftest_layers_controls.cpp`,
  `CMakeLists.txt`.
- No new dependency.
