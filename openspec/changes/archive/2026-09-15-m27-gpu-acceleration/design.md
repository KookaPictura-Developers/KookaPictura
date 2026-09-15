## Context

`pictura-render` ships two compositors. The CPU compositor `composite_rgba` is
the oracle: it accumulates straight-alpha RGBA in `f32` (`Canvas`/`Px`), applies
27 blend modes, group isolation, masks, and adjustment layers, and quantizes to
planar 8-bit only at the end. The GPU compositor `composite_gpu`
(`crates/pictura-render/src/gpu.rs`) runs the same per-layer loop in one WGSL
compute shader and is proven against the oracle within ±1 LSB by
`tests/gpu_parity.rs`.

After M26 the GPU compositor is the default interactive path
(`composite_active`, gated by the `gpuCompute` session preference), but it is
only about 1.2× faster than the CPU at 1024²×4 layers (release: CPU 70 ms vs
GPU 57 ms). Profiling the data path explains why: `Gpu::build_source` assembles
a full-canvas `f32` RGBA buffer on the host for every layer (16 bytes per canvas
pixel even for a small layer), `build_mask` assembles a full-canvas `f32` mask,
the canvas itself is `array<vec4<f32>>`, and `read_canvas` maps back `f32` and
converts to planar on the host. Transfer and conversion dominate the compute.

Filters are entirely on the CPU. `PictureView::apply_filter`
(`crates/pictura-app/src/cxxqt_object.rs`) maps a UI kind to a
`pictura_filters::Filter` and calls `pictura_render::apply_filter`
(`crates/pictura-render/src/filter.rs`), which assembles the layer's three color
planes into a 3-channel `PixelBuffer`, calls `pictura_filters::apply`, and blends
the result back through the selection mask. No filter kernel runs on the GPU.

Constraints: the CPU compositor and `pictura_filters::apply` are frozen as the
oracles; `docs/` is the long-form contract; wgpu 30.0.1 on Vulkan is the only
backend; `pictura-render` already depends on `pictura-filters`, so the filter
API can live there and the app keeps depending only on `pictura-render`; no new
dependency.

## Goals / Non-Goals

**Goals:**

- Remove the `f32` host round-trip from the GPU compositor: upload raw planar
  8-bit channel data over each layer's rect, upload a `u8` mask plane, keep the
  canvas as packed 8-bit RGBA, and read back packed 8-bit RGBA.
- Keep the compositor contract identical: same public API, same
  `PixelBuffer` shape, ±1 LSB versus `composite_rgba` on the parity corpus.
- Run the pointwise and separable-convolution filter families on the GPU by
  default when `gpuCompute` is enabled and an adapter exists, with a never-fail
  CPU fallback and ±1 LSB parity versus `pictura_filters::apply`.
- One shared device and cached pipelines for both compositing and filters.

**Non-Goals:**

- On-screen zero-copy present (manual QRhi + QWindow swapchain; blocked per
  `crates/pictura-app/GPU-INTEROP-NOTES.md`).
- Off-GUI-thread / async compute.
- GPU painting / brush (the dab loop).
- The painterly/stochastic filter families (M22 Artistic, M25 Brush
  Strokes/Sketch/Texture, Oil Paint) and other neighbourhood kernels.
- Dissolve on the GPU.
- New working precision or color-management changes; the output stays the
  planar straight-alpha 8-bit buffer the oracle produces.

## Decisions

### Frozen compositor contract (unchanged)

```rust
pub enum Backend { Gpu, Cpu }

pub fn gpu_available() -> bool;
pub fn composite_gpu(doc: &Document) -> Result<PixelBuffer, GpuError>;
pub fn composite_gpu_or_cpu(doc: &Document) -> PixelBuffer;
pub fn composite_active(doc: &Document, gpu_enabled: bool) -> (PixelBuffer, Backend);
```

All stay public with the M26 signatures and behavior. The output is the same
4-channel planar straight-alpha 8-bit `PixelBuffer`, within ±1 LSB of
`composite_rgba` on the parity corpus. The CPU compositor is the oracle and
does not change.

### GPU-native data path

Per layer, upload:

- **Source**: the layer's raw planar 8-bit channels over its clamped rect, not a
  host-assembled full-canvas `f32` RGBA buffer. RGB sources upload channels
  `0,1,2`; grayscale-family documents upload channel `0` once (the shader
  replicates it); the alpha channel uploads as its own plane and defaults to
  `255` when absent. Upload size drops from 16 bytes per canvas pixel to one
  byte per source pixel per plane.
- **Mask**: mask coverage as a `u8` plane, sized to the rect for a pixel layer,
  and to the canvas for a group or adjustment layer (they act across it).

Keep the running **canvas as packed 8-bit RGBA** in a storage buffer, with no
`f32` canvas. The shader unpacks the canvas, the source planes, and the mask to
`f32`, runs the existing blend/source-over math, and packs the result back to
8-bit on store. Output is read back as packed 8-bit RGBA and de-interleaved to
the planar `PixelBuffer` on the host; the `f32` readback and the host
planar→RGBA source assembly are gone. Packed 8-bit data is carried in `u32`
words (`array<u32>` storage bindings) with channels selected by byte index.

- *Why:* transfer and conversion dominate the current cost, and the compositing
  math is the same either way. Packed 8-bit uploads shrink source traffic by up
  to 16×, masks by 4×, and readback by 4×, which is where the 1.2× becomes a
  real speedup.
- *Ceiling:* the CPU oracle accumulates in `f32` and quantizes once, so a
  packed 8-bit canvas rounds after every layer. Photoshop composited 8-bit
  documents in 8-bit, and the parity corpus (two-layer scenes and one
  adjustment over a base) stays within ±1 LSB. If a deep stack ever exceeds the
  tolerance, the escape hatch is an `f32` canvas under the same inputs; do not
  widen the tolerance.
- *Alternatives:* keep the `f32` canvas and shrink only the uploads (rejected —
  the `f32` readback and conversion remain); a half-float canvas (rejected — no
  parity gain over 8-bit for an 8-bit oracle, and more transfer).

### Group layers dispatch the inner canvas

A group composites its children into an inner canvas as before; the inner canvas
is already packed 8-bit RGBA, so the group dispatch binds it directly as the
source (a source-layout flag in the params uniform selects packed RGBA versus
the planar pixel-layer layout). No re-materialization to `f32`.

### Frozen filter API

```rust
// pictura_render::gpu
pub fn filter_gpu_available() -> bool;
pub fn apply_filter_active(filter: &Filter, buf: &mut PixelBuffer, gpu_enabled: bool) -> Backend;
```

- `apply_filter_active` applies `filter` to a 3-channel planar 8-bit
  `PixelBuffer` in place. It uses the GPU when `gpu_enabled &&
  filter_gpu_available()` and a GPU kernel exists for `filter`; otherwise it
  runs `pictura_filters::apply` on the CPU. It returns `Backend::Gpu` when the
  GPU produced the buffer and `Backend::Cpu` otherwise. It never panics and
  never returns an error: a missing kernel or any `GpuError` is the sentinel
  (`Backend::Cpu`) that triggers the CPU fallback.
- `filter_gpu_available()` reuses the M26 cached capability probe
  (`devices().is_ok()`); it is cheap and never panics.
- Internally, the GPU filter path returns `Result<(), GpuError>` and a new
  `GpuError::UnsupportedFilter` variant reports a kernel-less filter, mirroring
  `UnsupportedMode`/`UnsupportedAdjustment`.

The layer-level entry keeps the app's dependency on `pictura-render` only:

```rust
// pictura_render::filter
pub fn apply_filter_backend(
    layer: &mut Layer,
    filter: &Filter,
    mask: Option<&LayerMask>,
    gpu_enabled: bool,
) -> Result<Backend, FilterError>;

pub fn apply_filter(layer: &mut Layer, filter: &Filter, mask: Option<&LayerMask>) -> Result<(), FilterError>;
```

`apply_filter_backend` builds the layer's 3-channel buffer, calls
`apply_filter_active`, blends the filtered result over the original through the
selection mask exactly as today, and writes it back. `apply_filter` keeps its
signature and behavior as the CPU oracle; it forwards to `apply_filter_backend`
with `gpu_enabled = false`. Both live in `pictura-render`, so `pictura-app`
needs no new dependency.

### Default-on filter routing

`PictureView::apply_filter` calls `apply_filter_backend(layer, &filter,
mask.as_ref(), rust.gpu_compute)` and records/ recomposites as today, so the M26
`gpuCompute` preference governs filters and compositing together. `gpuCompute`
off, or no adapter, gives a buffer byte-identical to `pictura_filters::apply`.
The status-bar backend indicator already reflects the composite backend; the
`--self-test` additionally reports the filter backend.

### Accelerated kernel scope (first pass)

GPU kernels cover the pointwise and separable-convolution families:

- **Separable convolution**: `GaussianBlur`, `BoxBlur`, `MotionBlur`, `Blur`,
  `BlurMore`, `Sharpen`, `SharpenMore`, `UnsharpMask`, `HighPass`.
- **Pointwise**: `Solarize` and the per-pixel gather family (`Offset`).

Any other `Filter` variant returns `GpuError::UnsupportedFilter` and falls back
to the CPU. The painterly/stochastic families (M22 Artistic, M25 Brush
Strokes/Sketch/Texture, Oil Paint) and the remaining neighbourhood kernels
(`Median`, `Despeckle`, `Maximum`, `Minimum`, `Crystallize`, `Mosaic`, and the
distort/stylize/render families) are the deferred extension.

- *Why:* separable convolutions reduce to two 1-D passes (`O(2·N·r)`) and
  pointwise kernels are one pass, so they map directly to compute dispatches;
  the stochastic families are random and not bit-reproducible on the GPU, the
  same reason Dissolve stays CPU-only.
- *Alternatives:* one generic kernel dispatch per family (rejected — a
  separable pass needs row/column staging, so two cached pipelines, not one);
  porting the painterly families (rejected — randomness parity and cost, and
  they are not the interactive hot path).

### GPU filter parity and the fallback sentinel

Every accelerated filter matches `pictura_filters::apply` within ±1 LSB on
representative inputs (gradients, edges, and small radii), asserted in
`tests/gpu_parity.rs`. A filter with no kernel, or any `GpuError`, returns the
CPU result; `apply_filter_active` reports `Backend::Cpu` and the buffer equals
the CPU oracle byte for byte when the CPU ran.

### One shared device, cached pipelines

`devices()` already creates the instance/adapter/device/queue once behind a
`OnceLock`, and M26 caches the compositor pipeline and bind-group layout
size-independently. Add the filter pipelines (pointwise and separable) to the
same cached resources; per-document buffers stay on the per-call `Gpu`.

### Never fail

Every composite and every filter application returns a buffer: an unavailable
device, an oversized document, a readback failure, a kernel-less filter, or an
unsupported blend mode or adjustment degrades to the CPU path for that call.
Opening, rendering, and filtering never depend on the GPU.

## Risks / Trade-offs

- **Packed 8-bit canvas rounding** → parity is asserted on the existing corpus
  plus filter scenes; the tolerance is pinned at ±1 LSB and a regression fails
  on a specific byte. The `f32` canvas remains the escape hatch, not a wider
  tolerance.
- **Planar 8-bit source layout differs from the `f32` layout** → the shader
  owns unpacking; the parity test is the guard, and CPU-side tests of the
  buffer builders can assert sizes and channel selection.
- **Filter parity across kernel families is precision-sensitive** → each
  accelerated kernel is compared against the CPU oracle; the test skips with a
  printed note when no adapter exists, so a Vulkan-less CI stays green.
- **Two filter pipelines plus the compositor pipeline** → they are
  size-independent and cached once; a separable pass still needs a row and a
  column stage.
- **Stochastic kernels stay CPU** → documented and tested; each future family
  extends the GPU set rather than changing the oracle.

## Migration Plan

Additive at the API level, with one behavior change (filters default to GPU
when enabled). `composite_gpu`, `composite_active`, and the CPU compositor are
unchanged. `apply_filter` keeps its signature and CPU behavior;
`PictureView::apply_filter` switches to `apply_filter_backend` with
`gpu_compute`. Rollback: route filters back to `apply_filter` and restore the
`f32` canvas/readback; the public API and the oracles are unaffected.

## Open Questions

- Whether the packed 8-bit canvas needs a per-layer `f32` shadow for very deep
  stacks — deferred; resolve only if the ±1 LSB parity test fails on a real
  stack.
- Whether the separable pass should use shared memory for the row stage — an
  optimization, not a contract; decide from the M27 timing evidence.
- Whether the adjustment WGSL helpers from M26 should share a module with the
  filter kernels — deferred; no shared module until both exist.
