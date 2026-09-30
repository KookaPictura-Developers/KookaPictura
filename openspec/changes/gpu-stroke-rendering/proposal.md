# Proposal

## Why

The live stroke is CPU-bound. A dab is rasterized pixel by pixel on the CPU
(`Stroke::sample` / `TipParams::coverage`) and each presented region is
composited through `composite_region_active`, whose GPU path still assembles and
uploads every layer's source planes per call (M37 in
`docs/dev/canvas-compositing-plan.md`). The largest brushes therefore cost
O(brush area) on one core: a ⌀5000 dab measures 933 ms of rasterization on a
4000² document even after the tip profile and layer lookups were hoisted, and
the 262 144 px raster budget caps the exact path below ⌀512. The GPU is idle
during all of it, and the earlier changes proved only that the *number* of CPU
pixels can be bounded — not that the pixels themselves can move off the CPU.

## What Changes

- **A GPU stroke path.** When a Vulkan device is available, the "Use Graphics
  Processor" flag is on, and the dab is over the raster budget, the live stroke
  is seeded once from the target layer (not the composited frame) and stays
  resident on the GPU for the stroke's duration; a dab is rasterized on the GPU
  over its bounding box — computing the tip coverage and compositing it in one
  pass — and its changed pixels are written back into the exact `Stroke`'s
  working document. The present composites that document through the existing
  `refresh_region`, so the live view is exact rather than an approximation, and
  the commit needs no replay. A sub-budget dab stays on the CPU: it is faster
  than the full-frame GPU seed it would pay for.
- **CPU stays the oracle and the fallback, and its verdict is the tolerance.**
  The CPU `Stroke` and `composite_region_active` remain the reference; the GPU
  path is selected only when it is available and the stack is supported, and
  its committed pixels must lie within the documented ±1 LSB of the CPU oracle.
  Dissolve, unsupported adjustments, layer effects, smart objects, Blend If, and
  transparency-locked layers stay on CPU.
- **Committed pixels are GPU-authored, within ±1 LSB.** The commit no longer
  replays the dabs through the exact CPU `Stroke`; the GPU already patched the
  working document, and `stroke.finish()` commits it. The CPU engine stays the
  golden verifier and the fallback.
- The stroke-LOD preview (`stroke-lod-preview`) stays the fallback for stacks
  the GPU path does not support and when no adapter exists.

## Capabilities

### New Capabilities

- `compositing/gpu-stroke-rendering`: the GPU-resident stroke path, its adapter
  gate, its CPU-parity requirement, and readback of only the presented region.

### Modified Capabilities

(none — the new capability carries the committed-pixel tolerance; the CPU
stroke path, the live CPU stroke and the LOD preview are unchanged.)

## Impact

- `crates/pictura-render/src/gpu/` — the resident stroke buffer (`stroke.rs`),
  its `STROKE_SHADER` and resources, and the region readback; the CPU-oracle
  parity test lives with it. The shader keeps an immutable base plane and a
  per-pixel coverage plane so overlapping dabs recomposite from the base and
  cannot compound or race.
- `crates/pictura-paint/src/{tip.rs,stroke.rs}` — the tip scalars,
  `Stroke::tip_params()`, and the `base_layer_rgba_doc` /
  `patch_working_layer` bridge the app seeds and patches through.
- `crates/pictura-app/src/cxxqt_object/{impl_paint.rs,state.rs,impl_core.rs}` —
  GPU stroke selection, the per-dab working-document patch, the frame-bounded
  present, and the fallback to the exact CPU / LOD-preview paths.
- No new dependencies (wgpu is already present); the GPU remains an accelerator,
  never an oracle (AGENTS rule 6).
