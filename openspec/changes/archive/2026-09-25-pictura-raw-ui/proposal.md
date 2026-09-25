# Proposal: pictura-raw-ui

## Why

`pictura-raw-core` ships the Pictura Raw engine (`PicturaRawSettings`, the `Fltr` codec, `render_pictura_raw`,
`apply_pictura_raw`) but leaves `Filter > Pictura Raw…` a disabled leaf and gives
the user no way to convert a raster layer and assign the camera-raw filter as its
smart filter. This change wires the dialog, the command, and the bridge so the
end-to-end raster → smart object → camera-raw filter flow works in one undo step.

The menu label is now `Pictura Raw…`; Adobe's on-disk filter name stays `"Camera Raw Filter"`.

## What Changes

- Add a modal `Pictura Raw` dialog with the 11 PV2012 Basic controls and
  OK/Cancel. No ACR tabs, filmstrip, histogram, or live preview.
- Add bridge commands: `layer_pictura_raw_settings(path)` for dialog prefill and
  `apply_pictura_raw_filter(path, <11 values>)` that converts a plain raster pixel
  layer to an embedded smart object when needed, then applies Pictura Raw and records one
  `"Pictura Raw"` history state.
- Replace the disabled `Filter > Pictura Raw…` leaf with a real command,
  gated enabled for a document with a convertible raster or an embedded
  smart-object target.
- Extend the C++ self-test with the convert+apply+re-edit path.

## Capabilities

### New Capabilities
- (none)

### Modified Capabilities
- `pictura-raw`: adds the UI/command surface (dialog, menu command, bridge) over the
  `pictura-raw-core` engine.

## Impact

- New files: `crates/pictura-app/cpp/pictura_raw_dialog.{h,cpp}`,
  `crates/pictura-app/src/cxxqt_object/impl_pictura_raw.rs`,
  `crates/pictura-app/cpp/selftest_pictura_raw.{h,cpp}`, plus OpenSpec artifacts.
- Modified: `crates/pictura-app/src/cxxqt_object.rs`, `commands.h`,
  `command_tree.cpp`, `frame_menus.cpp`, `frame_includes.h`, `CMakeLists.txt`,
  `selftest_layers_controls.cpp`, `docs/dev/STATE.md`,
  `docs/dev/psd-support-roadmap.md`.
- No new dependency. Behavioural parity only (no Adobe pixel oracle): the dialog
  ranges beyond the documented ones are inferred.
