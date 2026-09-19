## Why

The codec can author embedded smart objects (`psd-smart-object-roundtrip`):
`write_psd` emits a valid `SoLd` plus embedded `lnk2` when a layer carries an
embedded `SmartObject` payload and no preserved config block. Nothing in the app
can create one, so the feature is unreachable: `Layer > Smart Objects > Convert
to Smart Object` exists in the command tree only as a disabled placeholder.

## What Changes

- Add an engine operation `pictura-render::convert_to_smart_object(doc, path) ->
  bool`. It accepts only a raster pixel layer, authors an embedded source
  document from the layer's raster, serializes it with `write_psd`, and attaches
  it to the layer as an embedded `SmartObject`. It keeps the layer's pixel
  channels (the raster proxy), so rendering is unchanged. Any ineligible target
  refuses and mutates nothing.
- Add the app command `Layer > Smart Objects > Convert to Smart Object`
  (command id `LayerSmartObjectConvertTo`). It is enabled only when the current
  layer is convertible, calls the engine op, clears link sets, recomposites, and
  records exactly one `"Convert to Smart Object"` history state on success; a
  refusal records nothing.
- Add a C++ self-test covering conversion, proxy-intact rendering, save→load
  preservation, and refusal for a group and the Background. Add a Rust engine
  test at the op level covering conversion, refusal cases, and payload
  round-trip.
- No new dependencies.

## Capabilities

### New Capabilities

- `smart-object-layer-actions`: converting a raster layer into an embedded smart
  object from the Layer menu, and the engine operation behind it. The action
  authors an embedded source, keeps the raster proxy, and records one undo step.

### Modified Capabilities

<!-- None. psd-smart-objects owns authoring/resolve/round-trip and is consumed
     unchanged; smart-object-rendering already renders a proxy-backed layer from
     that proxy. This change adds the app/engine action that creates the object. -->

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` (new) plus
  registration in `layer_ops/mod.rs` and re-export in `document_ops/mod.rs`.
- `crates/pictura-app/cpp/commands.h`: the `LayerSmartObjectConvertTo` id.
- `crates/pictura-app/cpp/command_tree.cpp`: register the existing Smart Objects
  leaf with the id instead of leaving it a disabled placeholder.
- `crates/pictura-app/cpp/frame_menus.cpp`: handler and enabled provider.
- `crates/pictura-app/src/cxxqt_object.rs` and a
  `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs` bridge
  method.
- A new/extended `crates/pictura-app/cpp/selftest_*.cpp` (+ `.h`); any new
  `.cpp`/`.h` MUST be added to `CMakeLists.txt` explicitly (no globbing).
- `crates/pictura-render/src/tests/smart_object.rs`: op-level coverage.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- **Rasterize Smart Object** and the `Layer > Rasterize > Smart Object` variant.
- **Edit Contents**, **Replace Contents**, and **Export Contents…**.
- **Place / Open As Smart Object** from a file, and linked (external) objects.
- **New Smart Object via Copy** and the **Stack Mode** submenu.
- Editing the embedded contents in the UI.
