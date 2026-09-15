## Why

The wgpu compositor exists and matches the CPU oracle within ±1 LSB for the
separable blend modes, but the app never calls it: `PictureView` composites
through `pictura_render::composite_rgba` on the CPU. GPU compute is dead code on
the interactive path. M26 makes GPU compute the default for document
compositing, with the CPU oracle as the explicit fallback, so the accelerator is
actually used and the oracle keeps verifying it.

## What Changes

- `pictura-render::gpu` gains backend selection: `Backend { Gpu, Cpu }`,
  `gpu_available()`, and `composite_active(doc, gpu_enabled) -> (PixelBuffer,
  Backend)`; any `GpuError` falls back to CPU for that call and never panics.
- GPU compositor completeness: Hue, Saturation, Color, Luminosity in WGSL
  (branch-free non-separable triplet math), plus the five app adjustment kinds
  (invert, posterize, threshold, brightness/contrast, hue/saturation) applied at
  the adjustment layer's stack position.
- Performance: cache the compute pipeline and bind-group layout instead of
  rebuilding them per composite, and upload only each layer's rect rather than a
  full-canvas source.
- Dissolve stays **CPU-only**: it is random and not bit-reproducible on the GPU,
  so a visible Dissolve layer returns `GpuError::UnsupportedMode` and the whole
  composite falls back to CPU. This is the one deliberate exception.
- App: GPU compute is the default. A `gpuCompute` boolean session preference
  (default `true`, schema version 2) disables it; a checkable command
  (`view.gpuCompute`, "Use GPU Compute") toggles it; the status bar shows the
  active backend (`GPU` / `CPU`, or `CPU (no GPU)` when the probe fails).
- The CPU compositor is the unchanged oracle.

## Capabilities

### New Capabilities

- `gpu-compute-backend`: default-on GPU compute for document compositing, the
  `gpuCompute` session preference and its schema-v2 persistence, the toggle
  command, backend status reporting, and the CPU fallback when GPU compute is
  disabled or no adapter is usable.

### Modified Capabilities

- `gpu-compositing`: the compositor now implements the four non-separable modes
  and the five adjustment layers on the GPU, caches its compute pipeline and
  bind-group layout, uploads per-layer rects, and is the default compositing
  path — superseding the requirement that non-separable modes and adjustment
  layers are rejected before dispatch.

## Impact

- `crates/pictura-render/src/gpu.rs` — backend selection, non-separable blend
  math, adjustment passes, cached pipeline/layout, rect-scoped uploads.
- `crates/pictura-render/tests/gpu_parity.rs` — parity for the new modes and
  adjustments, backend selection, and fallback.
- `crates/pictura-app/src/cxxqt_object.rs` — composite through
  `composite_active`; expose backend availability and the active backend.
- `crates/pictura-app/cpp/session.{h,cpp}` — `gpuCompute` field, schema v2.
- `crates/pictura-app/cpp/commands.*`, `command_tree.cpp`, `frame.*` — the
  `view.gpuCompute` command and the status-bar backend indicator.
- No new dependency (wgpu/pollster already present); the CPU compositor and its
  tests are unchanged.
