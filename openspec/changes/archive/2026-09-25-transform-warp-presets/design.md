# Design: transform-warp-presets

## Context

`warp-mesh` shipped `transform_layer_warp` and `identity_mesh`; `surface()`
already evaluates a tensor-product Bézier of degree `cols-1`/`rows-1`, so a
preset is just another mesh producer. Patchy's `generate_style_warp_mesh` (MIT)
reproduces Photoshop's own bakes, so it is the oracle for the geometry.

## Goals / Non-Goals

**Goals**

- The 15 CS6 preset nets, exact against the Patchy golden table.
- A one-shot `Edit > Transform > Warp` command through the shipped engine op.

**Non-Goals**

- Interactive mesh dragging, the curved cage overlay, `View > Extras`, Warp
  Text, Puppet Warp, Content-Aware Scale.

## Decisions

**Styles and grids.** `WarpStyle` mirrors the CS6 pop-up order (`None, Custom,
Arc, ArcLower, ArcUpper, Arch, Bulge, ShellLower, ShellUpper, Flag, Wave, Fish,
Rise, Fisheye, Inflate, Squeeze, Twist`). `style_mesh(style, bend,
rotate_vertical, w, h)` returns the Patchy construction: a 4×2 net for
Arc/ArcLower/ArcUpper/Arch/Bulge/Flag/Fish/Rise, 4×3 for Wave, 4×4 for
ShellLower/ShellUpper/Fisheye/Twist, quadratic 3×3 for Inflate/Squeeze.
`rotate_vertical` transposes via the same `swap_axes` construction (`Twist`
ignores it, matching Photoshop).

**None/Custom.** Neither is a preset construction, so `style_mesh` returns
`None`. The command treats that as "no change": no history state, no mutation. A
custom net remains the interactive tool's job, which is not shipped.

**Distortion.** `style_mesh` returns the undistorted net; the app passes
`WarpParams { distort_h: X, distort_v: Y }` and `transform_layer_warp` applies
the shipped row-then-column scaling, matching Patchy's `apply_warp_distortion`.

**App.** A modal `WarpPresetDialog` (style combo, Bend, X/Y distortion, a
vertical-orientation check) calls the `apply_warp_preset` bridge, which builds
the mesh from the target layer's rect, calls `transform_layer_warp`,
recomposites, and records one `"Warp"` state. This mirrors the HDR Conversion
command; the interactive cage is deferred.

## Risks / Trade-offs

- [Patchy's captures are Photoshop 2026] → CS6 preset equivalence is assumed
  (all 15 names match); marked `ponytail:` in the module and here.
- [No interactive editing] → the command is one-shot; a user cannot drag the
  cage. Marked a ceiling.
- [Grid identity rounding] → `bend == 0` equals `identity_mesh` of the style's
  grid within `1e-9` (the transposed construction reassociates a float), not
  bit-exact.

## Migration Plan

Additive; revert is a revert.

## Sources

- `https://github.com/SethRobinson/Patchy` at commit
  `7d14d1f6ede2dc8fb52c11eefcc7cc8783473711` (MIT): `src/core/warp_mesh.cpp`
  `generate_style_warp_mesh` / `apply_warp_distortion` and `docs/warp.md`. The
  preset mesh constructions are ported as math; no Patchy code is vendored.
  Patchy validated them against Photoshop screenshots/COM captures; this repo
  reproduces the exact control points as a golden table.
