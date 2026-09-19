## Why

Smart objects can be created (Convert, Place), replaced, rasterized, and opened,
but their embedded source cannot be extracted. `Layer > Smart Objects > Export
Contents…` is a disabled placeholder in `command_tree.cpp`, and Adobe's action
is exactly this: write the embedded source to a file on disk so it can be
edited or reused outside the document. Today the payload is reachable only
inside the document model.

## What Changes

- Add a pure engine accessor
  `pictura-render::smart_object_source_bytes(doc: &Document, path: &str) ->
  Option<Vec<u8>>`. It resolves `path` and returns a clone of the embedded
  payload only when the target is a non-group, non-adjustment layer whose
  `smart_object` has a non-empty payload; every other target returns `None`.
  It is a pure read and never mutates the document.
- Add the bridge `PictureView::export_smart_object_contents(path, dest) -> bool`
  in `crates/pictura-app/src/cxxqt_object`: read the payload through the engine
  accessor and write it to `dest` with `std::fs::write`. Return `true` only on a
  successful write.
- Add the app command `Layer > Smart Objects > Export Contents…` (stable id
  `LayerSmartObjectExportContents`), enabled only when the current layer is a
  smart object with a payload. Its handler shows a save dialog defaulting to
  `*.psd`, calls the bridge, and reports success. Export only reads the
  document, so it records no history state.
- Add a Rust engine test and a C++ self-test.
- No new dependencies.

## Capabilities

### New Capabilities

<!-- None: the action extends the existing smart-object-layer-actions capability. -->

### Modified Capabilities

- `smart-object-layer-actions`: adds requirements for reading a layer's
  embedded payload, writing it byte-for-byte to a destination, and the
  read-only Export Contents command. The existing convert, place, replace,
  rasterize, and open-as requirements are unchanged.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` (extend)
  plus registration in `layer_ops/mod.rs`, `document_ops/mod.rs`, and `lib.rs`,
  and tests in `crates/pictura-render/src/tests/smart_object.rs`.
- `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs` bridge and
  `crates/pictura-app/src/cxxqt_object.rs` declarations.
- `crates/pictura-app/cpp/commands.h`, `crates/pictura-app/cpp/command_tree.cpp`,
  and `crates/pictura-app/cpp/frame_menus.cpp`.
- `crates/pictura-app/cpp/selftest_layers_smart_object.{cpp,h}` and its call
  site in `selftest_layers_controls.cpp` (no new `.cpp`/`.h`, so
  `CMakeLists.txt` is unchanged).
- No new dependencies.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- **Edit Contents** (the remaining flagship; it needs a cross-document editing
  session and save-path changes). Replace Contents is already shipped.
- The interactive transform session.
- Non-PSD/PSB conversion on export: the payload is written byte-for-byte,
  whatever its original format. The "layer-created object exports as PSB" rule
  from the CS6 contract is not implemented here; a converted-from-layers object
  exports the PSD bytes that `write_psd` stored.
- Linked (external) objects.
- New Smart Object via Copy, which stays a disabled placeholder.
