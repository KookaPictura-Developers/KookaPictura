## Why

M27 gave the compositor a GPU-native data path and put the pointwise and
separable-convolution filter families on the GPU through
`pictura_render::gpu_filter`. The remaining interactive stalls are the
deterministic neighbourhood kernels that still call `pictura_filters::apply` on
the CPU. Profiling that function at 1024×1024 (release, best of 2) shows:

- `SurfaceBlur` 7077 ms, `OilPaint` 1446 ms, `Median` 1418 ms — each over a
  second at 1 MP, and Surface Blur is ~7 s;
- `Custom` 193 ms, `LensFlare` 190 ms, `Maximum` 95 ms, `Minimum` 91 ms.

`MotionBlur` (1455 ms), `GaussianBlur` (901 ms) and `HighPass` (182 ms) from the
same profile are already accelerated by M27, so they are excluded. The stochastic
families (`Crystallize` 3114, `Watercolor` 336, `ConteCrayon` 326, `PaintDaubs`
321, `DryBrush` 266, `OceanRipple` 199, `Spatter` 125, `Sponge` 121,
`PaletteKnife` 110, `AddNoise` 97, `ColoredPencil` 95) cannot be moved: their
seeded RNG stream is not bit-reproducible on the GPU, so they stay on the CPU
and are deferred. M28 extends the M27 mechanism to the deterministic
window/neighbourhood kernels that profiling says are worth it.

## What Changes

- **Extend the existing mechanism, not the architecture.** New WGSL modes and
  branches are added to the one compute shader in
  `crates/pictura-render/src/gpu_filter.rs`; the shared device, cached pipeline,
  packed planar 8-bit upload/readback, and the never-write-alpha rule are reused
  unchanged. No second device, no new pipeline family.
- **Unchanged public contract.** `filter_gpu_available()` and
  `apply_filter_active(filter, buf, gpu_enabled) -> Result<Backend, FilterError>`
  keep their signatures and semantics; the filter path stays default-on when the
  M26 `gpuCompute` preference is set and an adapter exists, and the CPU fallback
  stays byte-identical to `pictura_filters::apply`.
- **New deterministic kernels.** Surface Blur, Median, Maximum, Minimum,
  Custom (5×5), and Oil Paint. `Custom` and the small square-neighbourhood
  convolutions reuse the existing `KERNEL` mode where possible; the window
  kernels (Surface Blur, Median, Maximum, Minimum) extend the separable/window
  dispatch. Each is parity-checked to ±1 LSB (aim 0) against the CPU oracle.
- **Profile-driven selection.** A filter is GPU-accelerated only if profiling
  shows ≥ ~150 ms at 1 MP **and** the GPU result matches the CPU oracle within
  ±1 LSB. A kernel that cannot reach parity stays on the CPU (byte-identical
  fallback) and is reported; the tolerance is never widened to make a kernel
  pass. `LensFlare` (190 ms) is deterministic but belongs to the render family,
  which is a non-goal, so it stays on the CPU.
- **Oil Paint resolves its M25 deferral.** M25 shipped Oil Paint as a CPU
  behavioural model because CS6's OpenCL kernel is closed and GPU-only. The GPU
  kernel mirrors that same M25 model (edge-stopping directional aggregation plus
  Lambert/Blinn-Phong relief) to within ±1 LSB — not Adobe's closed kernel.
- **Performance expectation.** A 1024×1024 Surface Blur should be at least ~10×
  faster on the GPU than the CPU baseline (7.1 s → sub-second).
- **Verification.** Extend `crates/pictura-render/tests/gpu_parity.rs` with
  parity for every new kernel and a Surface Blur timing test; the app
  `--self-test` reports the filter backend.

## Capabilities

### New Capabilities

None. M28 extends an existing capability.

### Modified Capabilities

- `gpu-filter-acceleration`: the accelerated kernel set grows from the pointwise
  and separable-convolution families to include the deterministic
  window/neighbourhood kernels Surface Blur, Median, Maximum, Minimum, Custom
  (5×5), and Oil Paint; admission becomes profile-driven (≥ ~150 ms at 1 MP and
  ±1 LSB parity), and any kernel that cannot reach parity stays on the CPU with
  a byte-identical fallback rather than a widened tolerance.

## Impact

- `crates/pictura-render/src/gpu_filter.rs` — new `Plan`/`Params` branches and
  WGSL modes for Surface Blur, Median, Maximum, Minimum, Custom, and Oil Paint;
  extended `plan()` validation mirroring `pictura_filters::apply`. The public
  `filter_gpu_available` / `apply_filter_active` API is unchanged.
- `crates/pictura-render/tests/gpu_parity.rs` — parity across the extended
  accelerated set (including a no-parity filter falling back to the CPU) and the
  Surface Blur timing evidence.
- `crates/pictura-app/src/cxxqt_object.rs` — `--self-test` reports the extended
  filter backend; the filter routing already goes through `apply_filter_active`,
  so no signature change.
- No new dependency (wgpu/pollster already present); `pictura-filters`, the CPU
  oracle, and the compositor are unchanged.
