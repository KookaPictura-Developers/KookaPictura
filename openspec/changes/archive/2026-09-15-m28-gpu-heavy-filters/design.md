## Context

M27 added `pictura_render::gpu_filter`, a wgpu compute path for
`pictura_filters::Filter` kernels. One WGSL compute shader carries several modes
behind a `Params` uniform and a normalized weight buffer: `KERNEL` (KxK
convolution), `SEP_H`/`SEP_V` (separable 1-D convolution with an unrounded `f32`
intermediate), `MOTION`, and `COMBINE` (Unsharp Mask / High Pass). Samples are
uploaded planar 8-bit, packed four bytes per `u32`; one invocation owns a whole
word so the shared word never races; alpha is never uploaded and never written,
so a 4-channel buffer keeps its alpha bit-for-bit. The pipeline and bind-group
layout are cached against the compositor's shared device
(`crate::gpu::shared_device`), and the module already imports
`pictura_filters::kernel::{gaussian_kernel, sigma_from_radius}`.

The accelerated set today is the pointwise and separable-convolution families:
`GaussianBlur`, `BoxBlur`, `MotionBlur`, `Blur`, `BlurMore`, `Sharpen`,
`SharpenMore`, `UnsharpMask`, `HighPass`, `Solarize`, `Offset`. Any other
`Filter` returns no plan and falls back to `pictura_filters::apply` on the CPU.
`apply_filter_active` is the entry point; `pictura-render` already depends on
`pictura-filters`, so the app keeps a single render dependency.

The heavy CPU orphans are the deterministic window/neighbourhood kernels.
`pictura-filters/tests/profile.rs` measures `apply` on a structured 1024×1024
RGB buffer (release, best of 2): Surface Blur 7077 ms, Oil Paint 1446 ms, Median
1418 ms, Custom 193 ms, Maximum 95 ms, Minimum 91 ms. (Motion Blur 1455,
Gaussian Blur 901, High Pass 182 are already GPU.) The stochastic families are
not portable: their seeded RNG stream cannot be reproduced bit-exactly on the
GPU, so they remain CPU-only.

Constraints: `pictura_filters::apply` is the frozen oracle; wgpu 30.0.1 on
Vulkan is the only backend; no new dependency; `docs/` is the long-form
contract; every kernel must degrade to a byte-identical CPU result rather than
an error.

## Goals / Non-Goals

**Goals:**

- Add deterministic GPU kernels for Surface Blur, Median, Maximum, Minimum,
  Custom (5×5), and Oil Paint by extending the existing `gpu_filter.rs`
  mechanism in place.
- Keep the public API (`filter_gpu_available`, `apply_filter_active`), the
  default-on routing, the packed planar 8-bit data path, and the byte-identical
  CPU fallback unchanged.
- Meet ±1 LSB (aim 0) parity against `pictura_filters::apply` for every new
  kernel; a kernel that cannot reach parity stays on the CPU.
- Make a 1024×1024 Surface Blur at least ~10× faster on the GPU than the CPU
  baseline (7.1 s → sub-second).

**Non-Goals:**

- The stochastic / painterly families (Crystallize, Watercolor, Conte Crayon,
  Paint Daubs, Dry Brush, Ocean Ripple, Spatter, Sponge, Palette Knife, Add
  Noise, Colored Pencil).
- Warps / distort kernels and render filters (Lens Flare).
- Dissolve on the GPU.
- On-screen zero-copy present, off-GUI-thread / async compute, GPU painting.
- New working precision or color-management changes; the output stays the
  planar straight-alpha 8-bit buffer the oracle produces.

## Decisions

### Frozen API and default-on routing (unchanged)

```rust
// pictura_render::gpu
pub fn filter_gpu_available() -> bool;
pub fn apply_filter_active(filter: &Filter, buf: &mut PixelBuffer, gpu_enabled: bool)
    -> Result<Backend, FilterError>;
```

Both stay public with the M27 signatures and behavior. `apply_filter_active`
applies `filter` to a 3- or 4-channel planar 8-bit `PixelBuffer` in place, uses
the GPU when `gpu_enabled && filter_gpu_available()` and a kernel exists for the
filter, and otherwise runs `pictura_filters::apply`. It never panics and never
returns a GPU error: a missing kernel or any `GpuError` degrades to the CPU
sentinel (`Backend::Cpu`). The app's `PictureView::apply_filter` continues to
thread the M26 `gpuCompute` preference through `apply_filter_backend`, so
filters and compositing share the toggle and `gpuCompute` off is byte-identical
to the oracle.

### Extend the mechanism, not the architecture

Add `Plan` variants, `Params` fields, and shader branches to the existing
`gpu_filter.rs` shader and `run`/`dispatch` plumbing. The shared device,
cached `FilterResources`, packed `u32` storage, one-word-per-invocation
dispatch, and alpha-preservation rule are reused. No second device, no new
pipeline family, no change to `gpu.rs` or the compositor.

- *Why:* the M27 machinery already carries the upload/dispatch/readback and the
  f32-vs-f64 rounding helper, so a new kernel is a validation arm, a params
  setup, and a shader branch.
- *Alternatives:* one pipeline per filter (rejected — more cached resources and
  shaders for no gain); a new module (rejected — same device and layout, and the
  shared `Params` already spans modes).

### Profile-driven selection rule

A filter is GPU-accelerated only if profiling shows ≥ ~150 ms at 1 MP **and**
the GPU result matches the CPU oracle within ±1 LSB on the parity corpus. A
kernel that cannot reach parity stays on the CPU (byte-identical fallback) and
is reported; the tolerance is never widened to make a kernel pass. Stochastic
filters are excluded outright.

- Applied to the profile: Surface Blur (7077), Oil Paint (1446), Median (1418),
  and Custom (193) clear the line; Maximum (95) and Minimum (91) sit just below
  it but are admitted because they are separable morphology that extends the
  existing separable dispatch at near-zero marginal cost. Lens Flare (190) is
  deterministic and above the line but is a render filter, a non-goal, so it
  stays CPU. Motion Blur, Gaussian Blur, and High Pass are already accelerated.
- *Why:* profiling decides the set, so effort tracks the interactive bottleneck
  (Surface Blur alone is ~7 s) instead of a family taxonomy. Parity, not cost,
  is the hard gate: a fast kernel that differs from the oracle is worse than no
  kernel.

### Window kernels: Surface Blur, Maximum, Minimum, Median

CPU models to mirror (`pictura_filters`):

- **Surface Blur** (`blur::surface`): a direct bilateral filter over the
  `(2r+1)²` window with clamp-to-edge. Weight is spatial
  `exp(-(kx²+ky²)/(2σ²))` times range `exp(-(Δluma)²/(2·thr²))`; per-colour
  `acc/weight_sum` rounds half-away-from-zero and clamps to 0..255. `r ∈ 1..=100`,
  `threshold ∈ 1..=255`.
- **Maximum / Minimum** (`other::morphology`): separable `(2r+1)²` square
  footprint, row pass then column pass, `r` clamped to 100, clamp-to-edge.
- **Median** (`noise::median`): per-channel `window[k²/2]` after sorting the
  `(2r+1)²` clamped window; `r == 0` is a no-op, `r` capped at `max(w,h)`.

GPU additions:

- Surface Blur is one window pass: a `SURFACE` mode computes the luma plane,
  sums the bilateral weights and the three colour accumulators in `f32`, and
  writes the rounded result. The `f32` accumulation differs from the CPU's
  `f64` only in the last bits; the final round to `u8` absorbs it.
- Maximum / Minimum reuse the separable staging: extend the `SEP_H`/`SEP_V`
  path with an op selector (weighted-sum vs max vs min) so the column pass reads
  the unrounded intermediate exactly as the Gaussian path does. The row pass
  writes the u8 intermediate for morphology (both passes are order statistics
  on u8, so no fraction is lost).
- Median needs an order statistic, not a sum. Stage the tile plus its halo once
  per workgroup, then build a 256-bin local histogram (shared per workgroup tile
  with a lane stride, or a per-lane histogram for small windows) and walk it to
  the `floor(k²/2)`-th count — exactly the element the CPU's sorted
  `window[len/2]` returns. The window is `(2r+1)²`, always odd, so the median is
  unique and byte-exact.

- *Why:* all three are direct neighbours of the existing separable/window
  machinery and share the upload/readback path.
- *Ceiling:* the direct `O(n·r²)` window is unchanged from the CPU model; a
  separable box-range approximation of the bilateral or a sliding median
  histogram is the upgrade path if large-radius cost bites. Do not change the
  CPU model.

### Neighbourhood / effect kernels: Custom (5×5), Oil Paint

- **Custom** (`other::custom`): a 5×5 convolution with f64 accumulation,
  clamp-to-edge, `out = clamp(Σ kernel·neighbour / scale + offset)`. Reuse the
  existing `KERNEL` mode with `ksize = 5`, `support = 2`, and `norm = scale`,
  and add the `offset` term to the kernel evaluation (carried in an unused
  `Params` slot). The `KERNEL` mode already owns the flat weight buffer and
  border clamp, so Custom is a validation arm plus one addend.
- **Oil Paint** (`oil_paint::oil_paint`): M25's CPU behavioural model,
  deterministic and alpha-preserving, in three stages — (1) luma plane and a
  Sobel gradient per pixel, `theta = atan2(gy, gx) + π/2`, tangent/normal basis;
  (2) edge-stopping directional aggregation over `d ∈ [-along, along]`,
  `k ∈ [-perp, perp]` with `edge = 1/(1+(Δluma/stop)²)` and
  `space = 1/(1+(d²+k²)·0.12)`, rounded sample offsets and clamp-to-edge;
  (3) a height field (`al·(0.2+0.8·scale/10) + (bristle/10)·0.35·sin((37x+17y)·0.11)²`)
  lit with Lambert plus a Blinn-Phong specular, `out = clamp_u8(a·lightf + 255·spec·0.7)`.
  The GPU kernel implements the same three stages in `f32`, using the existing
  `round_away` helper (WGSL `round` is half-to-even) and a `clamp_u8`
  equivalent. `stylization`, `cleanliness`, `scale`, `bristle_detail`, and
  `shine` are `0..=10`; `angular_direction` is `0..=360`.

  This mirrors the M25 model, not Adobe's closed kernel; the kernel's admission
  is exactly the ±1 LSB parity gate, so if `f32` transcendentals push an
  isolated pixel past the tolerance the filter stays on the CPU. Resolves the
  M25 deferral.

- *Why:* Custom is a one-addend extension of an existing mode; Oil Paint is
  deterministic and a top-three CPU cost, and the M25 model is already the
  contract.
- *Alternatives:* porting Adobe's Oil Paint OpenCL kernel (rejected — closed,
  and the parity gate is against the M25 model, not Adobe); a separate Oil Paint
  pipeline (rejected — same device, same upload path).

### Parity, fallback, and never fail

Every new kernel is compared against `pictura_filters::apply` within ±1 LSB per
colour channel on representative inputs, with alpha unchanged on 4-channel
buffers, in `tests/gpu_parity.rs`. A filter with no plan, an unparity-capable
kernel, or any `GpuError` returns the CPU result; `apply_filter_active` reports
`Backend::Cpu`, and when the CPU ran the buffer equals the oracle byte for byte.
The test skips with a printed note when no adapter exists so a Vulkan-less CI
stays green. A filter is never an error: invalid parameters surface as
`Err(FilterError)` from the CPU oracle only.

### Performance evidence

Add a Surface Blur timing test alongside the M27 timing test: time
`pictura_filters::apply` and `apply_filter_active` at 1024×1024, print both, and
assert the GPU is at least ~10× faster when an adapter is present (skip
otherwise). The CPU baseline is ~7.1 s; the target is sub-second.

### One shared device, cached pipelines

`FilterResources` stays a single cached pipeline and bind-group layout; the new
modes are shader branches and `Params` fields, so no new cache entries are
needed for the window kernels. Add reusable scratch storage (a luma plane and,
for Median, a shared histogram staging buffer) sized per call, the same way the
separable `mid` buffer is today.

## Risks / Trade-offs

- **Surface Blur is `O(n·r²)` and the dominant cost** → the GPU parallelizes the
  window directly; the timing test guards the ≥10× claim. A separable box-range
  approximation is the escape hatch if a larger radius regresses.
- **Median's order statistic on the GPU** → a histogram walk returns the exact
  `k²/2` element; if a workgroup-local histogram is awkward for large windows,
  fall back to the CPU and report, do not approximate.
- **Oil Paint f32 transcendentals** → `atan2`/`cos`/`sin`/`powf` in `f32` can
  round differently from the CPU's `f64`, changing a sample offset near a `.5`
  boundary. The ±1 LSB gate decides admission; if it fails, the filter stays on
  the CPU.
- **Bilateral `f32` accumulation** → the final round to `u8` absorbs the
  difference from the CPU `f64` sum; the parity test is the guard.
- **Maximum / Minimum below the nominal 150 ms line** → admitted as cheap
  separable extensions; if parity or timing disappoints, drop them back to the
  CPU (byte-identical fallback) without touching the hard gate.
- **Stochastic kernels stay CPU** → documented and tested; each family would
  need a bit-exact RNG, which is a separate milestone.

## Migration Plan

Additive. New arms in `plan()`, new `Params` fields/modes in `run` and the
shader, and new parity/timing tests. The public API, the CPU oracle, and the
compositor are untouched. Rollback: remove the new `plan()` arms (the filters
then take the existing `None` fallback) and the tests; no public signature
changes to revert.

## Open Questions

- Whether the Median workgroup histogram should be per-lane or shared-with-stride
  — resolve from the first parity/timing run; both return the same byte.
- Whether Surface Blur needs a shared-memory tile for the luma plane to cut
  redundant global reads at radius 100 — an optimization, not a contract.
- Whether Maximum/Minimum belong in the accelerated set after timing evidence —
  decided by the numbers, not the taxonomy.
