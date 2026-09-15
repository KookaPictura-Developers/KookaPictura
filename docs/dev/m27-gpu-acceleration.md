# M27 — GPU acceleration: native data path + GPU filters

Goal: make the default GPU path actually fast, and extend GPU compute to
filters. After M26 the wgpu compositor is the default but only ~1.2× faster
than the CPU (release: CPU 70 ms vs GPU 57 ms for 1024²×4 layers), because each
layer is materialized as a full-canvas host `f32` RGBA source plus an `f32`
mask, the canvas is kept as `f32`, and the result is read back as `f32` and
converted to planar on the host. M27 replaces that data path with a GPU-native
one — raw planar 8-bit channel uploads over the layer's rect, a `u8` mask
plane, a packed 8-bit RGBA canvas, and a packed readback — and adds a GPU
filter path for the pointwise and separable-convolution families. OpenSpec
change `m27-gpu-acceleration` (new capability `gpu-filter-acceleration`;
MODIFIED `gpu-compositing`).

## Scope

- GPU-native compositor data path in `pictura-render::gpu`: per-rect planar
  8-bit source uploads, `u8` mask planes, packed RGBA8 canvas storage, packed
  RGBA8 readback; group layers dispatch the already-8-bit inner canvas as the
  source.
- Unchanged public contract: `composite_gpu`, `composite_gpu_or_cpu`,
  `composite_active`, `gpu_available`, `Backend`, `GpuError`; identical
  `PixelBuffer` shape; ±1 LSB vs `composite_rgba` on the parity corpus; CPU
  stays the oracle.
- GPU filter path: `pictura_render::gpu::{filter_gpu_available,
  apply_filter_active}` applies `pictura_filters::Filter` kernels as wgpu
  compute passes; default-on when the M26 `gpuCompute` preference is enabled
  and an adapter is available, CPU otherwise.
- Accelerated kernels (first pass): the separable-convolution blur family
  (Gaussian, Box, Motion, Blur, BlurMore), Sharpen / SharpenMore / UnsharpMask,
  HighPass, and simple pointwise filters. Painterly/stochastic and other
  neighbourhood kernels fall back.
- App: filter application routes through the GPU-backed path, gated by the M26
  `gpuCompute` preference; `--self-test` reports the filter backend.
- Verification: extend `crates/pictura-render/tests/gpu_parity.rs` (filter
  parity + the M27 timing test) and the app `--self-test`.

## Out of scope (later milestones)

- On-screen zero-copy present (manual QRhi + QWindow swapchain; blocked per
  `crates/pictura-app/GPU-INTEROP-NOTES.md`).
- Off-GUI-thread / async compute.
- GPU painting / brush (the dab loop).
- The painterly/stochastic filter families (M22 Artistic, M25 Brush
  Strokes/Sketch/Texture, Oil Paint) and Dissolve on GPU.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Waves: (1) GPU-native compositor data path + parity/timing;
(2) GPU filter path + convolution family + parity; (3) app filter routing
(default GPU) + self-test; (4) close-out.

## Verification

- `cargo test -p pictura-render` (compositor and filter parity, M27 timing) and
  `cargo test --workspace`
- `cmake --build build`; fixture and no-argument self-tests exit 0 with the
  backend report
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
