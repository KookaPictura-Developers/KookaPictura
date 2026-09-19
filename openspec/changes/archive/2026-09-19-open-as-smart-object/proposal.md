## Why

The smart-object app commands now cover Convert, Place, Rasterize, and Replace
Contents, but `File > Open As Smart Object…` is still a disabled leaf
(`crates/pictura-app/cpp/command_tree.cpp:41`). Photoshop opens a file as a new
untitled document whose sole layer is a smart object containing that file. The
engine already decodes a PSD/PSB and appends an embedded smart-object layer
(`pictura-render::place_smart_object`), so the open path is wiring, not new
machinery.

## What Changes

- Add an engine operation `pictura-render::open_as_smart_object(filename: &str,
  bytes: &[u8]) -> Option<Document>`. It decodes `bytes` with
  `pictura_codec::read_psd`; on parse failure it returns `None`. On success it
  creates a NEW document at the source's dimensions and color mode/depth, sets
  its merged composite from the decoded source's composite, appends one top
  layer via `place_smart_object(&mut doc, filename, bytes)` (an Embedded,
  channel-less, native-size smart-object layer), and returns `Some(doc)` with
  exactly one layer.
- Add the bridge `PictureView::open_as_smart_object(path) -> bool` in
  `crates/pictura-app/src/cxxqt_object`. It reads the file and builds the
  document through the engine op with the file's display name, stores the
  composite/image, resets edit state, captures exactly one `"Open As Smart
  Object"` history state, and leaves the view's path as NONE (a new untitled
  document, so Save cannot overwrite the source). It returns `false` without
  mutating on failure.
- Add the app command `File > Open As Smart Object…` with the stable id
  `FileOpenAsSmartObject = "file.openAsSmartObject"`. It opens a `*.psd *.psb`
  dialog and is enabled always (like File Open). On a chosen path it mirrors
  `PicturaMainWindow::openPath` (`crates/pictura-app/cpp/frame.cpp:316`): new
  `PictureView`, call the bridge, then `addDocument(view, QString())` so the tab
  is untitled.
- Add a Rust engine test in `pictura-render` and a C++ self-test covering open
  as a smart object, one-layer/untitled assertions, and malformed-source refusal.
- No new dependencies.

## Capabilities

### New Capabilities

<!-- None: the action extends the existing smart-object-layer-actions capability. -->

### Modified Capabilities

- `smart-object-layer-actions`: adds `File > Open As Smart Object…`, which opens
  a PSD/PSB source as a new untitled document holding exactly one embedded
  smart-object layer, plus the engine op, bridge, and undo-state contract. The
  existing convert, place, rasterize, and replace requirements are unchanged.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` (extend)
  plus registration in `layer_ops/mod.rs`, `document_ops/mod.rs`, and `lib.rs`.
- `crates/pictura-render/src/tests/smart_object.rs`: engine coverage.
- `crates/pictura-app/src/cxxqt_object/impl_core.rs` (or a smart-object impl)
  and the `PictureView` declaration in `crates/pictura-app/src/cxxqt_object.rs`.
- `crates/pictura-app/cpp/{commands.h,command_tree.cpp,frame_menus.cpp,frame.cpp,frame.h}`
  and a `selftest_*.cpp` check with the next free exit code (281).
- No new dependencies.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- **Edit Contents**, **Export Contents**, the interactive transform session,
  non-PSD/PSB source formats, and linked (external) objects. This change covers
  the PSD/PSB source path only.
