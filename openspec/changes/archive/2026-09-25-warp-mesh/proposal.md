# Proposal: warp-mesh

## Why

`Edit > Transform > Warp` is an unimplemented CS6 command. `TOOL-001`
(docs/03-tools/move-and-transform.md) specifies the model: a control mesh
displaced by the user, interpolated as a **bicubic Bézier patch** (Adobe UXP
`CustomWarp4X4` is a 4×4 grid; `EmilDohne/PhotoshopAPI` authors "a cubic bezier
patch"; `SethRobinson/Patchy` publicly documented the same 4×4 cubic Bézier
cage). The exact Adobe patch degree and the 15 named preset parameters are
closed (no Photoshop oracle), so this change ships the exact, self-verifiable
core — a **custom** 4×4 mesh warp — and leaves the preset formulas to a later
change that can pin them against Patchy.

## What Changes

- Add `pictura_render::transform_layer_warp(doc, path, mesh, params)`: resample a
  layer through the tensor-product cubic Bézier surface defined by a 4×4
  control net in the source rect's local space, with the options-bar
  `distort_h`/`distort_v` percentages applied as row/column scaling about the
  edge midpoints. It reuses the similarity op's refusal contract (missing path,
  group, adjustment, Background, position-locked, zero-area, non-materializable
  channel-less target) and drops unmodeled raw channels.
- A uniform control net SHALL be the identity map; the four outer corner
  control points SHALL stay fixed under an interior-only edit.
- The 15 named `WarpStyle` presets, the interactive mesh overlay, and the CS6
  `Bend`/`X`/`Y` preset geometry are **out of scope** (they need the Patchy
  oracle to avoid guessing); the `Edit > Transform > Warp` command stays a stub.

## Capabilities

### Modified Capabilities

- `free-transform`: adds the mesh-warp op.

## Impact

- New `crates/pictura-render/src/document_ops/layer_ops/warp.rs` and its
  re-exports; no PSD read/write change (a raster warp is destructive and not
  serialized).
- Self-consistency tests (identity bit-exact, corner-fixed, determinism,
  out-of-source transparent, refusals bit-identical); behavioral parity only, no
  Photoshop oracle, marked as such.
- No new dependency.
