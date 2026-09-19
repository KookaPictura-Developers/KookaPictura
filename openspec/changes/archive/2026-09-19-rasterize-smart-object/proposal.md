## Why

The app can Convert to Smart Object (`convert-to-smart-object`), but there is no
way to rasterize the result back to pixels. `Layer > Rasterize > Smart Object`
is a disabled placeholder in `command_tree.cpp`, and even a manual
`smart_object = None` would still re-emit the preserved `SoLd` block and leave
an orphan `lnk*` record on save. Rasterizing must drop both so the result
persists as a plain pixel layer.

## What Changes

- Add an engine operation `pictura-render::rasterize_smart_object(doc, path) ->
  bool`. It refuses (no mutation) unless the target is a non-group,
  non-adjustment layer with `smart_object.is_some()`. On success it materializes
  the object's content into the layer's pixel channels — a layer that already
  has a color channel (`id == 0`) keeps its raster proxy untouched; otherwise
  the embedded payload is decoded through the existing embedded-source render
  path, scaled into the layer rect, and written to channels
  `0..mode.color_channels()` plus `-1` alpha. It clears `smart_object`, removes
  the preserved `SoLd`/`SoLE`/`plLd`/`PlLd` config block from `extra_blocks`,
  and removes the matching document-level linked-source record so a re-save
  carries neither a smart object nor an orphan `lnk*` record.
- Add a codec helper `pictura-codec::remove_linked_source(layer_section_extra,
  uuid) -> Option<Vec<u8>>` that rebuilds a `lnkD`/`lnk2`/`lnk3`/`lnkE` block
  without the matching record and copies every other block byte-for-byte.
- Add the app command `Layer > Rasterize > Smart Object` (id
  `LayerRasterizeSmartObject`), enabled only for a rasterizable smart-object
  layer, recording exactly one `"Rasterize Smart Object"` history state on
  success.
- Add a Rust engine test, a codec unit test, and a C++ self-test covering
  convert→rasterize, refusals, and the save→load outcome.
- No new dependencies.

## Capabilities

### New Capabilities

<!-- None: the action extends the existing smart-object-layer-actions capability. -->

### Modified Capabilities

- `smart-object-layer-actions`: adds a new requirement for rasterizing a
  smart-object layer back to pixels and dropping the preserved smart-object
  blocks and linked-source record. The existing conversion requirements are
  unchanged.

## Impact

- `crates/pictura-codec/src/smart_object.rs` (or a new `linked.rs`) plus a
  `lib.rs` re-export.
- `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` (extend)
  and `crates/pictura-render/src/tests/smart_object.rs`.
- `crates/pictura-app/cpp/{commands.h,frame_menus.cpp,command_tree.cpp}` and
  `crates/pictura-app/cpp/selftest_layers_controls.cpp` (existing suite, no new
  file).
- `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs` bridge.
- No new dependencies; no new `.cpp`/`.h`, so `CMakeLists.txt` is unchanged.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- **Edit Contents**, **Replace Contents**, **Export Contents…**.
- **Place / Open As Smart Object** from a file, and linked (external) objects.
- **New Smart Object via Copy**.
