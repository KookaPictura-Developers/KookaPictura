## Context

`pictura-render` ships two compositors. The CPU compositor `composite_rgba` is
the oracle: 27 blend modes, group isolation, masks, and adjustment layers,
executed as one per-layer loop with `blend_into`. The GPU compositor
`composite_gpu` (`crates/pictura-render/src/gpu.rs`) runs Normal plus the
separable modes in a single WGSL compute shader, is proven against the oracle
within ±1 LSB by `tests/gpu_parity.rs`, and returns `GpuError` for CPU-only
modes and adjustment layers so `composite_gpu_or_cpu` can fall back.

The app never calls it. `crates/pictura-app/src/cxxqt_object.rs` `current_buffer`
composites with `composite_rgba` on the CPU, so the accelerator is unused on the
interactive path. The M0.5 interop investigation (`crates/pictura-app/GPU-INTEROP-NOTES.md`)
showed on-screen zero-copy present is blocked by `QRhiWidget`'s internal `QRhi`;
the offscreen → readback → `QImage` path remains the display route. GPU compute
accelerates the composite itself, not presentation.

Constraints: the CPU compositor is frozen as the oracle; `docs/` is the
long-form contract and only changes with a `TASK-ALLOWS-DOCS` marker; wgpu 30.0.1
on Vulkan is the only backend; no new dependency.

## Goals / Non-Goals

**Goals:**

- GPU compute is the default document-compositing path whenever an adapter is
  usable and the user has not disabled it.
- Full-stack parity within ±1 LSB: all 27 modes (non-separable included), masks,
  opacity, groups, and the five app adjustment kinds.
- A persisted, user-visible backend toggle, an active-backend status indicator,
  and a never-fail CPU fallback.
- Eliminate the per-composite pipeline rebuild and the full-canvas per-layer
  upload.

**Non-Goals:**

- On-screen zero-copy present (manual QRhi + QWindow swapchain; blocked today).
- Off-GUI-thread / async compositing.
- GPU filters and GPU painting (follow-up change `m27-gpu-filter-acceleration`).
- Dissolve on the GPU.
- New working precision (16F/32F) or color-management changes; the GPU path keeps
  the same 8-bit straight-alpha planar output as the oracle.

## Decisions

### Frozen backend interface

```rust
pub enum Backend { Gpu, Cpu }

pub fn gpu_available() -> bool;
pub fn composite_active(doc: &Document, gpu_enabled: bool) -> (PixelBuffer, Backend);
```

- `composite_active` returns the GPU composite with `Backend::Gpu` when
  `gpu_enabled && gpu_available()`, otherwise the CPU composite with
  `Backend::Cpu`. It never panics and catches any `GpuError` (including
  `UnsupportedMode` and `UnsupportedAdjustment`) by falling back to CPU for that
  call.
- `composite_gpu`, `composite_gpu_or_cpu`, and `GpuError` remain public and
  unchanged for compatibility and the existing tests.
- `gpu_available()` is a cached capability probe: it reports whether the shared
  device exists and meets the required limits. `composite_active` still checks
  per-document size limits and degrades to CPU when a document exceeds them.

### One shared device, cached pipeline and layout

`devices()` already creates the instance/adapter/device/queue once behind a
`OnceLock`. Extend the same pattern to create the bind-group layout and compute
pipeline once, and let `Gpu` hold per-document buffers only.

- *Why:* `Gpu::new` currently rebuilds the layout, shader module, and pipeline on
  every composite — a pipeline recompile on the interactive path. The pipeline
  and layout are size-independent because buffer sizes travel as bindings and a
  uniform.
- *Alternatives:* keep per-call (rejected — measurable overhead); a long-lived
  `Gpu` cached per document size (rejected for now — buffers change with every
  edit and size).

### Non-separable modes in WGSL

Implement Hue, Saturation, Color, and Luminosity with the PDF/CSS branch-free
triplet functions (`Lum`, `ClipColor`, `SetLum`, `Sat`, `SetSat`), extend
`mode_id`, and add them to the `blend` function. These are the blend-mode
formulas the CPU oracle already follows.

- *Why:* leaving them CPU-only would put the fallback on the common path for any
  document containing a Luminosity or Color layer. Branch-free triplet math is
  deterministic and keeps the one-shader design.
- *Alternatives:* a shader per mode (rejected — more pipelines to cache and
  test); keeping the CPU path (rejected — see above).

### Five adjustment layers on the GPU

At a visible adjustment layer's stack position, run the adjustment kernel over
the running-canvas RGB and blend the adjusted colour back using the layer's
mask, opacity, and blend mode — the same order as `composite_adjustment`. The
supported kinds are the five the app can create and decode: invert (`nvrt`),
posterize (`post`), threshold (`thrs`), brightness/contrast (`brit`), and
hue/saturation (`hue2`). The kernels are ported to WGSL; `pictura_adjust::apply`
stays the CPU oracle the parity test compares against. Any other adjustment kind
returns `GpuError::UnsupportedAdjustment`, falling the whole composite back to
CPU.

- *Why:* adjustment layers are the largest remaining fallback trigger; covering
  the five the app supports removes it for real documents while leaving unknown
  PSD adjustment keys safely on the CPU path.

### Dissolve stays CPU-only

Dissolve is the one deliberate GPU exception. Its per-pixel randomness is not
bit-reproducible on the GPU, so a visible Dissolve layer returns
`GpuError::UnsupportedMode(BlendMode::Dissolve)` and the whole composite falls
back to CPU. Documented here so the exception is intent, not an omission.

### Rect-scoped uploads and buffer reuse

Upload only each layer's rect rather than a full-canvas source: carry the rect
origin and size in the params uniform, read source at rect-local coordinates,
and early-out canvas writes outside the rect. Reuse the source/mask/params
buffers across layers, growing them to the largest size needed, where practical.

- *Why:* a full-canvas upload per layer dominates memory traffic for small layers
  on a large canvas.
- *Ceiling:* channel sampling stays on the CPU per layer (the existing
  `ponytail:` note in `gpu.rs`); move it into WGSL only if upload bandwidth shows
  up in a profile.

### Default-on toggle and session persistence

GPU compute is the default. Add `bool gpuCompute = true` to `SessionState`,
bump `schemaVersion` to 2, and read a missing field as `true` so a store written
by schema 1 enables GPU. Add a checkable command `view.gpuCompute` (label "Use
GPU Compute"; the app agent picks the exact View-menu placement) that flips the
preference, recomposites, and repaints. When no adapter is available the command
is disabled/greyed and the status bar reads `CPU (no GPU)`; otherwise the status
bar shows `GPU` or `CPU`.

- *Why:* one boolean is the whole user surface; the app reports the backend from
  the same `composite_active` result it renders, so the indicator cannot drift
  from reality.

### Never fail to open a document

Every composite call returns a buffer: an unavailable device, an oversized
document, a readback failure, or an unsupported mode/adjustment degrades to the
CPU composite for that call. Opening and rendering never depend on the GPU.

## Risks / Trade-offs

- **±1 LSB parity on non-separable and adjustment math is precision-sensitive** →
  the parity test covers every mode and adjustment against the unchanged CPU
  oracle; the test skips with a printed note when no adapter exists, so a
  Vulkan-less CI stays green (the target machine has an RTX 3090/Vulkan).
- **CPU/GPU readback rounding drift** → the CPU oracle does not change and the
  tolerance is pinned; a regression fails on a specific byte instead of shifting
  silently.
- **A cached pipeline shared across document sizes** → the pipeline and layout
  are size-independent and per-document buffers are still created per composite.
- **More WGSL than one shader comfortably holds** → the pipeline stays one
  compute entry point with a mode selector; split into a family of shaders only
  if the compiler or profile demands it.
- **Additional adjustment kinds still fall back** → documented and tested; each
  future kind extends the GPU set rather than the CPU oracle.

## Migration Plan

Additive at the API level. The app switches its single composite call site
(`current_buffer`) to `composite_active`. `SessionState` gains one field
(schema 2) that reads as `true` from an old store, so existing sessions enable
GPU by default. Rollback: call `composite_rgba` again, drop the command and
status indicator, and ignore `gpuCompute`; the persisted field is harmless.

## Open Questions

- Exact View-menu placement and label casing for `view.gpuCompute` — the app
  agent decides within the frozen label "Use GPU Compute".
- Whether the adjustment WGSL helpers should be shared with the GPU filter work
  planned for `m27-gpu-filter-acceleration` — deferred; no shared module until
  both exist.
- Whether `gpu_available()` should re-probe after a device/driver reset — out of
  scope; a cached probe is an accepted ceiling for this change.
