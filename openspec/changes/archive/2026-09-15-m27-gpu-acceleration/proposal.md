## Why

After M26 the wgpu compositor is the default document-compositing path, but it
is only about 1.2× faster than the CPU (release: 70 ms CPU vs 57 ms GPU for a
1024² four-layer document). The bottleneck is the data path, not the compute:
for each layer the host assembles a full-canvas `f32` RGBA source and a
full-canvas `f32` mask, the canvas lives as `f32`, and the result is read back
as `f32` and converted to planar on the host. That is roughly 16 bytes per
pixel uploaded per layer and 16 bytes per pixel read back, so transfer and
conversion dominate. Filters never touch the GPU at all: `PictureView::apply_filter`
calls `pictura_render::apply_filter`, which runs `pictura_filters::apply` on the
CPU. M27 removes the `f32` host round-trip from the compositor and extends the
same wgpu device to the filter families that map cleanly to compute.

## What Changes

- **GPU-native compositor data path.** Layer sources are uploaded as raw planar
  8-bit channel data over the layer's rect (RGB, or grayscale replicated plus
  alpha) instead of a CPU-assembled full-canvas `f32` RGBA buffer. Mask coverage
  is uploaded as a `u8` plane. The canvas is kept as packed 8-bit RGBA in a
  storage buffer (no `f32` canvas) and read back as packed 8-bit RGBA, removing
  the `f32` readback and the host planar→RGBA source assembly. Group layers
  dispatch the already-8-bit inner canvas as their source.
- **Unchanged contract.** `composite_gpu`, `composite_gpu_or_cpu`,
  `composite_active`, `gpu_available`, `Backend`, and `GpuError` stay public and
  behave the same; the result is the same 4-channel planar straight-alpha 8-bit
  `PixelBuffer` and matches `composite_rgba` within ±1 LSB on the parity corpus.
  The CPU compositor remains the oracle.
- **GPU filter path.** `pictura_render::gpu` gains `filter_gpu_available()` and
  `apply_filter_active(filter, buf, gpu_enabled) -> Backend`, applying
  `pictura_filters::Filter` kernels as wgpu compute passes: upload the layer's
  8-bit samples, dispatch, read back 8-bit. The app-facing layer entry threads
  the M26 `gpuCompute` preference.
- **Default-on.** When `gpuCompute` is enabled and a usable adapter exists,
  filter application uses the GPU; otherwise `pictura_filters::apply` runs on
  the CPU. `gpuCompute` off, or no adapter, yields a CPU result byte-identical
  to the CPU oracle.
- **Kernel scope (first pass).** The pointwise and separable-convolution
  families: the blur family (Gaussian, Box, Motion, Blur, BlurMore),
  Sharpen/SharpenMore/UnsharpMask, HighPass, and simple pointwise filters. The
  painterly/stochastic families (M22 Artistic, M25 Brush Strokes/Sketch/Texture,
  Oil Paint) and other neighbourhood kernels stay CPU-only and are documented as
  the deferred extension.
- **Parity and fallback.** Every accelerated filter matches the CPU result
  within ±1 LSB for representative inputs. A filter with no GPU kernel returns a
  sentinel (`Backend::Cpu`) that triggers the CPU fallback; it is never an
  error.
- **App.** `PictureView::apply_filter` routes through the GPU-backed entry with
  `gpu_compute`; `--self-test` reports the filter backend alongside the existing
  composite backend.

## Capabilities

### New Capabilities

- `gpu-filter-acceleration`: default-on GPU application of `pictura_filters`
  kernels through wgpu compute passes, the frozen `filter_gpu_available` /
  `apply_filter_active` API, the accelerated pointwise and separable-convolution
  kernel set, ±1 LSB parity versus `pictura_filters::apply`, and the
  CPU fallback when compute is disabled, no adapter exists, or a filter has no
  GPU kernel.

### Modified Capabilities

- `gpu-compositing`: the compositor now uses a GPU-native data path — per-rect
  planar 8-bit source uploads, `u8` mask planes, a packed 8-bit RGBA canvas, and
  packed readback — in place of the full-canvas host `f32` source/mask
  materialization and the `f32` readback and host conversion, while keeping the
  same output shape and ±1 LSB parity as the CPU oracle.

## Impact

- `crates/pictura-render/src/gpu.rs` — raw 8-bit source/mask uploads, packed
  RGBA8 canvas and readback, group source dispatch; filter pipelines
  (`filter_gpu_available`, `apply_filter_active`, WGSL pointwise and separable
  kernels).
- `crates/pictura-render/src/filter.rs` — layer-level filter entry that threads
  `gpu_enabled`, calls `apply_filter_active`, and keeps the mask coverage blend
  on the host; `apply_filter` stays as the CPU oracle.
- `crates/pictura-render/src/lib.rs` — re-export the new filter API.
- `crates/pictura-render/tests/gpu_parity.rs` — filter parity across the
  accelerated set and fallback for a non-accelerated filter; M27 compositor and
  filter timing.
- `crates/pictura-app/src/cxxqt_object.rs` — `apply_filter` passes
  `gpu_compute`; `--self-test` reports the filter backend; the app depends only
  on `pictura-render`, whose `pictura-filters` dependency already exists.
- No new dependency (wgpu/pollster already present); `pictura-filters` and the
  CPU compositor are unchanged.
