## Why

Smart objects can be created (Convert, Place) and dissolved (Rasterize), but
their source cannot be swapped. `Layer > Smart Objects > Replace Contents…` is a
disabled placeholder in `command_tree.cpp`, and Adobe's action is exactly this:
swap the embedded source while keeping the layer's transform. Without dropping
the preserved `SoLd`/`lnk*` bytes a save would re-emit the old source.

## What Changes

- Add an engine operation
  `pictura-render::replace_smart_object_contents(doc, path, filename, bytes) -> bool`.
  It refuses (no mutation) unless the target resolves to a non-group,
  non-adjustment layer whose `smart_object` is `Embedded` with a payload, and
  unless `bytes` parse as a PSD/PSB document. On success it sets the payload,
  `filename`, `filetype` `8BPB`, and `creator` `8BIM`, clears the stored `uuid`,
  clears the layer's pixel channels so the new source renders through the
  embedded-source path scaled into the existing `rect`, and leaves the
  transform/geometry and every other layer property untouched.
- Drop the layer's preserved config block (`SoLd`/`SoLE`/`plLd`/`PlLd`) and,
  when the old uuid was non-empty, the matching document-level linked-source
  record via `pictura_codec::remove_linked_source`, so the re-save authors a
  fresh `SoLd`/`lnk2` for the new payload instead of re-emitting the old source.
- Add the app command `Layer > Smart Objects > Replace Contents…` (id
  `LayerSmartObjectReplaceContents`), enabled only for a replaceable smart
  object, showing a `*.psd *.psb` file dialog; the bridge reads the file, calls
  the engine op, and records exactly one `"Replace Contents"` history state on
  success (none on refusal).
- Add a Rust engine test (fixture-backed, self-skipping) and a C++ self-test.
- No new dependencies.

## Capabilities

### New Capabilities

<!-- None: the action extends the existing smart-object-layer-actions capability. -->

### Modified Capabilities

- `smart-object-layer-actions`: adds a requirement for replacing a smart
  object's embedded source in place while preserving its transform, plus the
  drop/re-author rule for the preserved blocks and linked record. The existing
  convert, place, and rasterize requirements are unchanged.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` (extend)
  plus registration and `crates/pictura-render/src/tests/smart_object.rs`.
- `crates/pictura-app/cpp/{commands.h,command_tree.cpp,frame_menus.cpp}` and
  `crates/pictura-app/cpp/selftest_layers_controls.cpp` (existing suite, no new
  file).
- `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs` bridge and
  `crates/pictura-app/src/cxxqt_object.rs` declarations.
- No new `.cpp`/`.h`, so `CMakeLists.txt` is unchanged; no new dependencies.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- **File > Open As Smart Object…**, **Edit Contents**, **Export Contents…**.
- The interactive transform session and re-placement transform editing.
- Non-PSD/PSB replacement sources (the codec reader only).
- Linked (external) objects.
