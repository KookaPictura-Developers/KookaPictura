# M26 — GPU compute by default

Goal: make the wgpu compositor (`crates/pictura-render/src/gpu.rs`) the
**default** document-compositing path, with the unchanged CPU compositor as the
fallback when the user disables GPU compute or no usable Vulkan adapter exists.
Today the compositor is parity-tested but unused: `crates/pictura-app/src/cxxqt_object.rs`
composites through `composite_rgba` on the CPU. Extend GPU coverage to the four
non-separable modes and the five app adjustment layers, cache the pipeline, and
wire a default-on `gpuCompute` preference with a user-visible toggle and status.
OpenSpec change `m26-gpu-compute` (new capability `gpu-compute-backend`; MODIFIED
`gpu-compositing`).

## Scope

- Backend selection in `pictura-render::gpu`: `Backend`, `gpu_available()`,
  `composite_active(doc, gpu_enabled) -> (PixelBuffer, Backend)`; any `GpuError`
  falls back to CPU without panicking.
- GPU completeness: Hue, Saturation, Color, Luminosity in WGSL; the five
  adjustment layers (invert, posterize, threshold, brightness/contrast,
  hue/saturation) at the adjustment layer's stack position; Dissolve stays
  CPU-only.
- Performance: cache the compute pipeline and bind-group layout; upload each
  layer's rect instead of a full-canvas source.
- App: GPU compute on by default; `gpuCompute` session preference (schema 2,
  default `true`); checkable `view.gpuCompute` command; status-bar backend
  indicator (`GPU` / `CPU` / `CPU (no GPU)`).
- Verification: extend `crates/pictura-render/tests/gpu_parity.rs` and the app
  `--self-test`.

## Out of scope (later milestones)

- On-screen zero-copy present (manual QRhi + QWindow swapchain; blocked per
  `crates/pictura-app/GPU-INTEROP-NOTES.md`).
- Off-GUI-thread / async compositing.
- GPU filters and GPU painting (follow-up change `m27-gpu-filter-acceleration`).
- Dissolve on the GPU.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Waves: (1) shared context + backend selection + probe;
(2) compositor completeness + performance; (3) app wiring (default GPU, toggle,
status, session); (4) parity tests + app self-test; (5) close-out.

## Verification

- `cargo test -p pictura-render` (parity across all modes and adjustments) and
  `cargo test --workspace`
- `cmake --build build`; fixture and no-argument self-tests exit 0 with the
  backend report
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
