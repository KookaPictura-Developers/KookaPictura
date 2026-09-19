## Why

Smart objects can be created by `Convert to Smart Object`, but not by placing a
file, which is Photoshop's primary create path. `File > Place…` is a disabled
leaf (`crates/pictura-app/cpp/command_tree.cpp:60`). The engine can already
author an embedded source and render a channel-less layer from it
(`pictura-render::composite_smart_source`), so the create path is missing only
wiring, not machinery.

## What Changes

- Add an engine operation `pictura-render::place_smart_object(doc, filename,
  bytes) -> Option<String>`. It decodes `bytes` with `pictura_codec::read_psd`;
  on parse failure it returns `None` and mutates nothing. On success it appends
  a new top layer named by `filename`, sized to the decoded document at
  `(0, 0)`, with no pixel channels, carrying an embedded `SmartObject` whose
  payload is `bytes`, and returns the new layer's path.
- Add the app command `File > Place…` (id `FilePlace = "file.place"`). It opens
  a `*.psd *.psb` file dialog, is enabled only when a document is open, and
  calls the bridge on the chosen path. The bridge (`place_smart_object(path) ->
  QString`) reads the file, derives the display name from its base name, runs
  the engine op, and on success clears link sets, recomposites, and records
  exactly one `"Place"` history state, returning the new layer path (empty on
  refusal; a refusal records nothing and leaves the document unchanged).
- Add a Rust engine test in `pictura-render` and a C++ self-test covering place,
  render, save→load, and malformed-source refusal.
- No new dependencies.

## Capabilities

### New Capabilities

<!-- None: the action extends the existing smart-object-layer-actions capability. -->

### Modified Capabilities

- `smart-object-layer-actions`: adds placing a PSD/PSB file as a channel-less
  embedded smart-object layer at native size from `File > Place…`, and the
  bridge command that records its undo state. Existing convert and rasterize
  requirements are unchanged.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` (extend)
  plus registration in `layer_ops/mod.rs`, `document_ops/mod.rs`, and `lib.rs`.
- `crates/pictura-render/src/tests/smart_object.rs`: engine coverage.
- `crates/pictura-app/cpp/{commands.h,command_tree.cpp,frame_menus.cpp}` and
  `crates/pictura-app/cpp/frame.cpp` (or `frame_menus.cpp`) for the dialog.
- `crates/pictura-app/cpp/selftest_layers_controls.cpp` (existing suite, no new
  file, so `CMakeLists.txt` is unchanged).
- `crates/pictura-app/src/cxxqt_object.rs` and
  `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs`.
- No new dependencies.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- **`File > Open As Smart Object…`**, the interactive Place transform session
  (Photoshop drops the object in and starts Free Transform), non-PSD/PSB source
  formats, **Edit Contents**, **Replace Contents**, **Export Contents…**, and
  linked (external) objects.
