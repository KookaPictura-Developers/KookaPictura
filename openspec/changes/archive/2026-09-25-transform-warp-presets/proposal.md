# Proposal: transform-warp-presets

## Why

`Edit > Transform > Warp` shipped only the custom 4×4 mesh (`warp-mesh`): the 15
named `WarpStyle` presets and the CS6 `Bend`/`X`/`Y` geometry were deferred
because no Photoshop oracle existed. SethRobinson/Patchy (MIT) has since
publicly documented Photoshop's exact preset mesh constructions against COM
captures to ~2.4e-6 px, so the presets can now be pinned to an oracle instead of
guessed.

## What Changes

- Add `pictura_render::{WarpStyle, style_mesh}`: the 15 named preset control-net
  constructions (plus `None`/`Custom`) at `bend ∈ [-100, 100]` and both
  orientations, ported from Patchy's `generate_style_warp_mesh` (math only, no
  vendored code). `None`/`Custom` produce no mesh; `bend == 0` is the style's
  identity grid.
- Make `Edit > Transform > Warp` a real command: a modal Warp dialog (Style,
  Bend, X/Y distortion, orientation) applying `style_mesh` through the existing
  `transform_layer_warp` in one undo state. Bend/X/Y/orientation are disabled
  for None/Custom.
- Golden control-point tests for all 15 styles at five bends and both
  orientations, produced from the pinned Patchy commit, plus identity, mirror,
  orientation, and distortion-order checks.
- Interactive mesh dragging, the curved cage overlay, `View > Extras`, Warp
  Text, Puppet Warp, and Content-Aware Scale stay out of scope.

## Capabilities

### Modified Capabilities

- `free-transform`: adds the preset warp mesh table and the Warp command.

## Impact

- New `crates/pictura-render/src/document_ops/layer_ops/warp_styles.rs` (+
  `warp_styles_tests.rs`) and re-exports; `WarpMesh::new`.
- App: `commands.h`, `command_tree.cpp`, `frame_menus.cpp`, `frame_includes.h`,
  a new `warp_preset_dialog.{h,cpp}` + `CMakeLists.txt`, and an
  `apply_warp_preset` bridge in `impl_transform/dispatch.rs` + `cxxqt_object.rs`.
- Tests: the golden engine suite and one C++ self-test check (code 527).
- No new dependency.
