# Proposal

## Why

The `Layer > Matting` submenu still ships all three of its leaves as disabled
stubs in `crates/pictura-app/cpp/command_tree.cpp`: `Defringe…`,
`Remove Black Matte`, and `Remove White Matte`. CS6 uses these to clean the
edge of a cut-out layer: the two Remove-Matte commands un-multiply a black or
white background that was composited into the layer's alpha, and Defringe
replaces the colour of edge pixels with the nearest interior colour. Users
cannot clean a matte today, and the engine has no un-matte or defringe
primitive, so the remaining work is the two pixel ops plus their wiring.

## What Changes

- Add engine ops in `crates/pictura-render/src/document_ops/matting.rs`:
  - `remove_matte(layer, background, mask)`: for every pixel with alpha > 0,
    recover the un-composited colour `out = (px − bg·(1−a)) / a` per channel,
    clamped to `0..=255`, where `bg` is `0` (black) or `255` (white). Alpha is
    unchanged.
  - `defringe(layer, width, mask)`: replace the colour of pixels within `width`
    of the transparent region with the colour of the nearest interior pixel,
    bounded by the selection mask.
  Both refuse a locked layer and an empty layer, and confine their writes to the
  active selection when one is present.
- Bridge free functions in
  `crates/pictura-app/src/cxxqt_object/impl_layers/matting.rs`:
  `layer_remove_black_matte`, `layer_remove_white_matte`, `layer_defringe`
  (each recomposites and records exactly one undo state), plus a
  `matting_target_ready` enablement read.
- Add a minimal `Defringe…` dialog (`cpp/defringe_dialog.{h,cpp}`) matching the
  house style: a `Width` spin box in pixels, default `1`, and OK / Cancel.
- Wire the three leaves: frozen ids in `cpp/commands.h`, `registry.add(...)`
  rows in `cpp/command_tree.cpp`, and handlers plus enabled-providers in a new
  `cpp/frame_menus_matting.cpp` (added to `CMakeLists.txt`).

## Capabilities

### New Capabilities

- `compositing/layer-matting`: the `Layer > Matting` edge-cleanup commands —
  Remove Black Matte, Remove White Matte, and Defringe — over the active layer,
  honouring the active selection and recording one undo state each.

### Modified Capabilities

<!-- none -->

## Impact

- Engine: `crates/pictura-render/src/document_ops/matting.rs` (new), re-
  exported from `document_ops/mod.rs` and the crate root.
- Bridge: `crates/pictura-app/src/cxxqt_object/impl_layers/matting.rs` (new),
  `crates/pictura-app/build.rs` (register the bridge).
- C++: `cpp/commands.h` (three frozen ids), `cpp/command_tree.cpp` (three
  rows), `cpp/frame_menus_matting.cpp` (new), `cpp/frame.h` (wiring
  declaration), `cpp/frame_menus.cpp` (call the wiring), `cpp/defringe_dialog.*`
  (new), `CMakeLists.txt` (new sources).
- Tests: engine unit tests, a Qt Test case under `cpp/tests/`.
- No new dependencies. No `docs/` changes. No `scripts/file-size-allowlist.txt`
  change.
