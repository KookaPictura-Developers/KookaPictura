# Proposal: free-transform-quad

## Why

Free Transform ships scale, rotate, and move only: `pictura_render::transform_layer`
is a similarity map (`scale_x`/`scale_y`/`angle`/`dx`/`dy`) and the app session
holds those five scalars. `TOOL-001` (docs/03-tools/move-and-transform.md) states
that **Skew** is affine but **Distort and Perspective require the full projective
(homography) form**, and the CS6 `Edit > Transform` submenu lists Skew, Distort,
and Perspective. Those three menu entries exist in `command_tree.cpp` as
`implemented = false` placeholders, and the engine can express none of them. This
adds the projective transform op and the three commit modes.

## What Changes

- `pictura_render::transform_layer_quad(doc, path, quad)` SHALL apply the
  projective map that sends the source layer rect's four corners (TL, TR, BR, BL)
  to `quad`, resampling every channel plane (and the layer mask) with bilinear
  interpolation, and SHALL share the existing refusal and channel-less
  materialization rules with `transform_layer`.
- The Free Transform session SHALL gain a mode: `Free` (today's similarity),
  `Skew`, `Distort`, and `Perspective`. In a projective mode the session holds the
  live target quad; a corner drag sets a corner (Distort), sets a corner while
  moving the opposite corner by the negated delta (Perspective), or an edge drag
  slides that edge parallel to itself (Skew).
- `Edit > Transform > Skew`, `Distort`, and `Perspective` SHALL become real
  commands that enter the matching mode on the active layer.
- Commit SHALL call `transform_layer` for `Free` and `transform_layer_quad` for
  the projective modes, recording one `"Free Transform"` history state; Escape
  SHALL still restore the document bit-identically.

## Capabilities

### Modified Capabilities

- `free-transform`: adds a projective (quad) transform op, the three CS6
  transform modes, and their menu commands.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/transform.rs`: add a
  homography `Map` path and `transform_layer_quad`; factor the shared refusal,
  materialization, and resample skeleton so the similarity op is unchanged.
- `crates/pictura-app/src/cxxqt_object/impl_transform/*` and `state.rs`: mode +
  live quad on the session, gestures, mode-aware hit-test, commit routing,
  `begin_transform_mode`.
- `crates/pictura-app/cpp/commands.h`, `command_tree.cpp`, `frame_menus.cpp`,
  `tool_transform.cpp`: the three command ids, specs, handlers, and enablement.
- Tests: engine quad tests (identity, translation parity with `transform_layer`,
  corner mapping, degenerate refusal, out-of-source transparency) and an app
  self-test that each mode commits one state and cancels cleanly.
- No new dependency. Behavior beyond the documented corner/edge semantics is
  inferred (no Photoshop oracle for the gestures) and SHALL be marked as such.
