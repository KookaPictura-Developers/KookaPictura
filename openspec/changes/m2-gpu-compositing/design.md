## Context

M2 established the CPU compositor (`pictura_render::composite_rgba`) as the
oracle for the layer stack and finished the PSD layer reader, but the Qt shell
still showed only a generated test image or the embedded PSD composite, and all
compositing was single-threaded CPU work. M2.5 makes the compositor visible and
adds a GPU accelerator without weakening the oracle contract.

The relevant constraints from the specs:

- **CPU is the oracle** (`docs/dev/m2.5-app-and-gpu.md`): GPU parity is proven
  by a test, never assumed, and the CPU compositor is not modified.
- **Never fail to open a document** (`docs/01-architecture/gpu-rendering-pipeline.md`,
  `qt6-ui-design.md`): a missing or blocked GPU must degrade to CPU, not panic.
- **The canvas host is provisional.** `ARCH-006` favours a staged approach:
  Qt owns the on-screen device, Rust computes offscreen, and results cross as a
  CPU buffer (Stage 1) before any zero-copy shared-image design (Stage 2).
- The M0.5 spike already proved offscreen wgpu→readback→`QImage` works and is
  the safe default; it also probed the zero-copy path
  (`crates/pictura-app/GPU-INTEROP-NOTES.md`).

## Goals / Non-Goals

**Goals:**

- Display the composited layer stack in the app when the document has layers,
  and the embedded PSD composite otherwise.
- Execute Normal plus every separable blend mode on a wgpu compute shader and
  prove, by test, that the output matches the CPU oracle within ±1 LSB.
- Keep the CPU compositor byte-for-byte authoritative and provide deterministic
  fallback whenever the GPU path cannot run.
- Keep the app runnable headless via `--self-test`, with a non-blank check that
  does not depend on a display server.

**Non-Goals:**

- GPU implementations of the non-separable modes (Hue, Saturation, Color,
  Luminosity) or Dissolve; they remain CPU-only.
- On-screen zero-copy presentation through `QRhiWidget`.
- Tile streaming, dirty-rectangle partial redraw, or real-time interactive
  recompositing.
- GPU color management, LUTs, or HDR precision (later milestones).

## Decisions

### One compute shader with a mode selector, not one shader per mode

The WGSL `cs_main` reads a `mode` id and dispatches to a small `sep` function.
Blend ids match a Rust `mode_id` so the fused and separable branches stay in one
place. A family of generated shaders would multiply pipeline-compile cost and
test surface for no measurable win at this stage. DarkenColor/LighterColor are
whole-RGB comparisons and are handled as explicit branches before the per-channel
path, matching the CPU formulas.

**Alternative considered:** one shader per blend mode. Rejected — 22 pipelines
to build and cache, and the mode is a per-dispatch uniform anyway.

### CPU assembles source and mask buffers; the shader does blend + source-over

The shader operates on flat storage buffers: a `vec4<f32>` per canvas pixel for
source and canvas, an `f32` coverage per pixel for the mask, and a uniform
params block. Channel sampling, grayscale replication, and mask coverage reuse
the CPU helpers (`channel`, `sample`, `mask_alpha`), so the GPU and CPU paths
cannot drift on how a layer's pixels are read. Only the arithmetic is on the
device.

This leaves a per-layer host→device upload. `ponytail:` in `gpu.rs` records the
ceiling: move channel sampling into WGSL if upload bandwidth ever shows up in a
profile.

### CPU-only modes fail fast to the CPU path

`check_supported` walks the visible stack before any GPU work and rejects
adjustment layers (`UnsupportedAdjustment`) and the five CPU-only modes
(`UnsupportedMode`). The caller uses `composite_gpu_or_cpu`, which catches every
`GpuError` and returns `composite_rgba(doc)`. This keeps the branch at one
boundary and means a mixed stack degrades whole, rather than compositing part of
it on the GPU and part on the CPU (which would risk a semantics mismatch at the
seam).

### Device is created once, lazily, and kept for the process

A `OnceLock<Option<Devices>>` holds the wgpu instance/adapter/device/queue.
`Devices` owns the objects that keep the raw Vulkan handles valid. No surface is
requested, so this works under Xvfb and `offscreen` with no platform Vulkan
instance.

### Offscreen wgpu → readback → QImage is the display path; QRhiWidget present is deferred

The M0.5 probe established that same-device, same-image sharing works: QRhi's
Vulkan backend adopts the wgpu-created `VkDevice` (`importDevice`) and
`QRhiTexture::createFrom` wraps a wgpu-owned `VkImage`. But `QRhiWidget` owns
its `QRhi` internally and exposes no `setRhi`/adopt hook, so an imported wgpu
device cannot back it. On-screen zero-copy present would require a manually
managed `QRhi` + `QWindow` swapchain (Qt's `rhiwindow` pattern), and the wgpu
device lacks `VK_KHR_swapchain` because no surface is ever created. That is an
architecture change, not a spike, and `ARCH-006` explicitly stages it behind
profiling. Decision: keep the offscreen→readback→`QImage` path (Stage 1) as the
M2.5 default, and record zero-copy present as blocked, not attempted.

**Alternative considered:** adopt `QRhiWidget` anyway and render via QRhi.
Rejected — it discards the wgpu compute shader and the already-proven device
import, and the widget's API does not permit the import.

## Risks / Trade-offs

- **On-screen present is blocked.** → Recorded as a limit, not a defect:
  offscreen import works; zero-copy present needs a manual QRhi + `QWindow`
  swapchain with `VK_KHR_swapchain` enabled at device creation. Keep readback
  until profiling shows it dominates.
- **Readback cost.** Every composite crosses the PCIe bus to the host. → Accept
  for M2.5; the GPU path still returns a host buffer because the display path is
  `QImage`. Tile/zero-copy work is deferred.
- **Storage-buffer format and limits.** Source/canvas are `f32` storage buffers;
  device `max_storage_buffer_binding_size`, `max_buffer_size`, and
  `max_compute_workgroups_per_dimension` bound document size. → `TooLarge` is
  returned and the caller falls back to CPU.
- **±1 LSB is a tolerance, not exact equality.** f32 shader math differs from
  the CPU's rounding at ties. → The parity test asserts the per-channel bound and
  reports the observed max delta; a regression beyond 1 LSB fails the test.
- **CPU-only modes silently route to CPU.** A caller that ignores the fallback
  contract would get CPU-speed output. → `composite_gpu` returns a typed
  `GpuError`; only the explicit `composite_gpu_or_cpu` helper falls back.
- **Device-loss / driver reset.** Not handled in this milestone. → The next
  `composite_gpu` call after a lost device returns a `GpuError` and the caller
  uses the CPU path; full device rebuild is deferred to the threading spec.
- **Two Vulkan engines touching one image.** `createFrom` returns success but
  does not establish a layout/synchronization contract (`setNativeLayout` plus
  semaphores is unverified). → Not used in the shipping path; recorded as an
  open question for the Stage 2 design.
