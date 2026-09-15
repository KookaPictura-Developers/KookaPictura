# M28 — GPU heavy deterministic filters

Goal: extend the M27 GPU filter path to the high-cost **deterministic** CPU
filters. M27 accelerated the pointwise and separable-convolution families; the
remaining interactive stalls are window/neighbourhood kernels still running
`pictura_filters::apply` on the CPU. Profiling that function at 1024×1024
(release, best of 2) ranks them:

| filter | ms @ 1 MP | M28 |
| --- | ---: | --- |
| Surface Blur | 7077 | add GPU kernel |
| Motion Blur | 1455 | already GPU (M27) |
| Oil Paint | 1446 | add GPU kernel |
| Median | 1418 | add GPU kernel |
| Gaussian Blur | 901 | already GPU (M27) |
| Custom | 193 | add GPU kernel |
| Lens Flare | 190 | deferred (render family) |
| High Pass | 182 | already GPU (M27) |
| Maximum | 95 | add GPU kernel |
| Minimum | 91 | add GPU kernel |

A 7 s Surface Blur dominates a filter session. M28 moves Surface Blur, Median,
Maximum, Minimum, Custom and Oil Paint onto the same wgpu compute mechanism M27
built. Stochastic filters stay on the CPU by design: a seeded RNG stream is not
bit-reproducible across backends. OpenSpec change `m28-gpu-heavy-filters`
(MODIFIED `gpu-filter-acceleration`).

## Scope

- Extend `pictura_render::gpu_filter` in place — no new compositor or pipeline
  architecture. The public API stays `filter_gpu_available()` and
  `apply_filter_active(filter, buf, gpu_enabled) -> Result<Backend, FilterError>`;
  the filter path stays default-on with a byte-identical CPU fallback.
- New kernels, all deterministic, each parity-checked to ±1 LSB (aim 0):
  - **Surface Blur** — bilateral window (spatial Gaussian × luma-range Gaussian).
  - **Median** — `(2r+1)²` order statistic, exact `window[k²/2]`.
  - **Maximum / Minimum** — separable square-footprint morphology.
  - **Custom 5×5** — f64-style convolution with scale and offset.
  - **Oil Paint** — the M25 CPU behavioural model (edge-stopping directional
    aggregation + Lambert/Blinn-Phong relief), mirrored on the GPU.
- Selection rule: a filter is accelerated only if profiling shows ≥ ~150 ms at
  1 MP **and** the GPU result matches the CPU oracle within ±1 LSB. A kernel
  that cannot reach parity stays on the CPU (byte-identical fallback) and is
  reported; tolerances are never widened.
- Verification: extend `crates/pictura-render/tests/gpu_parity.rs` with parity
  for every new kernel and a Surface Blur timing test; the app `--self-test`
  reports the filter backend.

## Out of scope (later milestones)

- Stochastic / painterly families (Crystallize, Watercolor, Conte Crayon, Paint
  Daubs, Dry Brush, Ocean Ripple, Spatter, Sponge, Palette Knife, Add Noise,
  Colored Pencil) — seeded RNG cannot be reproduced bit-exactly on the GPU.
- Warps / distort kernels, render filters (Lens Flare), Dissolve on GPU.
- On-screen zero-copy present, off-GUI-thread compute, GPU painting.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Waves: (1) window kernels (Surface Blur, Maximum, Minimum,
Median) plus parity; (2) neighbourhood/effect kernels (Custom, Oil Paint) plus
parity; (3) app self-test and timing evidence; (4) close-out.

## Verification

- `cargo test -p pictura-render` (new-kernel parity, Surface Blur timing) and
  `cargo test --workspace`
- Surface Blur at 1024² at least ~10× faster on the GPU than the CPU baseline
  (7.1 s → sub-second on the same host)
- `cmake --build build`; fixture and no-argument self-tests exit 0 with the
  filter backend report
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
