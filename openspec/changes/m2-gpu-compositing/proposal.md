## Why

M2 made the layer stack compositable in `pictura-render`, but nothing showed it
in the app, and compositing ran only on the CPU. M2.5 closes both gaps: the Qt
shell displays the composited stack, and the separable blend modes get a wgpu
compute-shader accelerator. The CPU compositor stays the oracle — the GPU path
is only accepted because a test proves it matches within ±1 LSB, and because it
degrades to the CPU path instead of failing when no adapter exists.

## What Changes

- App: `PictureView` now renders `pictura_render::composite_rgba` of the layer
  stack when a PSD has layers, falling back to the embedded PSD composite when
  it does not. Layer visibility, adjustment layers, and selection-masked
  adjustments refresh the displayed composite.
- App: the offscreen wgpu render → readback → `QImage` path is wired through
  `PictureView::render_gpu`, and `--self-test` runs headless (Xvfb/offscreen),
  proving the displayed image is non-blank and exiting non-zero on failure.
- Render: new `pictura_render::composite_gpu(doc)` executes Normal plus the
  separable blend modes (including whole-RGB Darken/Lighten comparisons) in one
  WGSL compute shader, producing the same planar RGBA8 buffer as the CPU oracle
  within ±1 LSB.
- Render: CPU-only modes (Hue, Saturation, Color, Luminosity, Dissolve) and
  adjustment layers are rejected with `GpuError` before any dispatch;
  `composite_gpu_or_cpu(doc)` falls back to the CPU compositor.
- Render: adapter/device creation, oversized documents, and readback failures
  return `GpuError` — the GPU path never panics when a GPU is missing.

## Capabilities

### New Capabilities

- `gpu-compositing`: the wgpu compute-shader compositor in `pictura-render` for
  Normal + separable blend modes, its ±1 LSB parity contract against the CPU
  oracle, rejection of CPU-only modes, and graceful fallback to the CPU path.
- `composite-view`: the Qt app displaying the composited layer stack (or the
  embedded PSD composite when there are no layers), the offscreen
  wgpu→readback→`QImage` display path, and the headless `--self-test` runner.

### Modified Capabilities

None. No existing capability's requirements change; the CPU compositor contract
is preserved.

## Impact

- `crates/pictura-render/src/gpu.rs` — new GPU compositor, `GpuError`,
  `composite_gpu`, `composite_gpu_or_cpu`, a process-lifetime wgpu device.
- `crates/pictura-render/tests/gpu_parity.rs` — parity, CPU-only rejection, and
  fallback tests; skip gracefully without an adapter.
- `crates/pictura-app/src/cxxqt_object.rs` — `document_to_image`,
  `current_buffer`, `render_gpu`, interop handle getters.
- `crates/pictura-app/src/gpu.rs` — offscreen render/readback and the interop
  probe.
- `crates/pictura-app/cpp/main.cpp`, `cpp/interop.{h,cpp}` — display refresh,
  `--self-test`, `--interop-probe`; build wiring in `CMakeLists.txt`.
- Dependencies: wgpu (Vulkan + WGSL), pollster, and ash (interop probe only)
  are already present; no new dependency is introduced by this change.
