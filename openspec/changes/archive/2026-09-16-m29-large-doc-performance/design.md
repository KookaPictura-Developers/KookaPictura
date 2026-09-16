## Context

M27/M28 gave the compositor and filters a wgpu compute path: one WGSL shader
each, one cached pipeline, packed planar-8 upload/readback, and a never-write-alpha
rule. The compositor's `cs_main` computes `i = gid.x` and dispatches
`self.n.div_ceil(64)` 1-D workgroups (`gpu.rs`); `gpu_filter`'s `cs_main` computes
`wi = gid.x` and dispatches `count.div_ceil(64)` (`gpu_filter.rs`).

Both backends gate on `max_compute_workgroups_per_dimension` (65535 on the RTX
3090). The 1-D count is `n.div_ceil(64)`, so **4000² (16.7 M px) is 250 000
workgroups — over the limit**. `Gpu::new` returns `Err(GpuError::TooLarge)`,
`composite_gpu` fails, and `composite_active` silently returns the CPU path
(`Backend::Cpu`). Above ~4.19 MP the GPU is never used; at 1024² the same call is
3.4× faster than the CPU. `gpu_filter` has the identical ceiling at
`count.div_ceil(64)`.

With the GPU disabled, a 4000² layer move runs the CPU composite. Profiling at
4000² (release, RTX 3090): `translate_layer` → `recompute` → `composite_rgba`
670 ms, `commit_move` 745 ms, `begin_move_preview` 610 ms. The move preview also
`doc.clone()`s (60 ms) and re-composites the whole document to build a base
image. Off-canvas layers are ~30% cheaper from rect clamping, but there is a
~265 ms floor from the always-full-canvas allocation regardless of layer size.

Separately, the panels re-scan the document every cycle: `InfoPanel::refresh`
calls `sample_argb` → `current_buffer` → `composite_active`, re-compositing the
whole document to fetch one pixel colour (~0.44 s per cycle, ≈ 1.3 s total
panel refresh); `HistogramPanel::recompute` materializes a full-resolution
`QImage::Format_RGB32` and scans 16 M pixels (145 ms); `layer_thumbnail(24)`
builds a full-size RGBA image before scaling (94 ms per layer).

Constraints: the CPU compositor (`composite_rgba`) and `pictura_filters::apply`
are the frozen oracles; parity is ±1 LSB; wgpu 30.0.1 on Vulkan is the only
backend; no new dependency; `docs/` is the long-form contract.

## Goals / Non-Goals

**Goals:**

- Lift the GPU compositor and GPU-filter ceiling from ~4.19 MP to the 2-D
  workgroup product ceiling (65535² ≈ 2.8×10¹⁴ px) by dispatching 2-D compute
  and deriving the linear index from a row stride.
- Composite a 4000² layer move on the active backend, so `commit_move` does not
  pay the CPU `composite_rgba` cost.
- Make the per-refresh panel cost O(viewport/downsample), not O(document):
  `sample_argb` reads the current cached composite, thumbnails downsample
  directly, and the histogram bins a ≤512² downsample.
- Start the move preview by hiding the moved layer in place instead of cloning
  the document.
- Keep the CPU fallback byte-identical and compositing/filter parity at ±1 LSB.
- Ship a 4000² timing test and a self-test backend/size report.

**Non-Goals:**

- History copy-on-write / tile diffs. `Document::clone()` (60 ms) is the next
  bottleneck once compositing is cheap; it is deferred and noted as the follow-up
  if PSB-size documents hit RAM.
- Off-GUI-thread / async compute.
- Zero-copy present; GPU painting.
- New working precision, color-management changes, or a different panel visual.

## Decisions

### 1. 2-D compute dispatch (frozen)

`gpu.rs` and `gpu_filter.rs` switch from
`dispatch_workgroups(count.div_ceil(64), 1, 1)` to a 2-D grid:

```
let gx = 65535u32.min(count.div_ceil(64).max(1));
let gy = count.div_ceil(gx * 64);
// reject when count > limit*limit*64 (the 65535² workgroup product)
pass.dispatch_workgroups(gx, gy, 1);
```

The shader derives the linear index from the stride the host passes in the
uniform:

```wgsl
let i = gid.x + gid.y * params.stride;   // stride == gx * 64
```

`stride` is a whole multiple of the 64-wide workgroup, so `gid.y` advances by
exactly one row of invocations and no invocation is double-covered or skipped;
`if (i >= count) { return; }` still guards the tail. `gpu.rs` writes `stride`
into one of its currently-zero trailing `Params` words; `gpu_filter.rs` adds a
`stride` field to `Params`. The 1-D
`n.div_ceil(64) > max_compute_workgroups_per_dimension` check in `Gpu::new` and
the 1-D `count.div_ceil(64) > ...` check in `gpu_filter` are replaced by the
2-D product check. No second pipeline, no new bind group, no shader-architecture
change; the packed `u32` path and alpha rule are untouched.

- *Why:* 2-D dispatch is the standard way past a per-dimension workgroup limit
  and needs only a stride uniform. The single linear index keeps the parity
  reasoning identical to the 1-D path.
- *Alternatives:* chunked 1-D dispatches (rejected — multiple submits per layer,
  more CPU overhead); tiling with a more complex index (rejected — same result,
  more code).

### 2. Commit through the active backend (frozen)

Add `translate_layer_active(doc: &mut Document, dx: i32, dy: i32, gpu_enabled: bool) -> bool`
in `document_ops`: it shifts the topmost pixel layer's rect (and mask) exactly as
`translate_layer` does, then refreshes `doc.composite` with
`composite_active(doc, gpu_enabled)` instead of the unconditional CPU
`recompute`. The CPU `translate_layer` keeps `recompute` and stays the oracle and
the tests' entry point; `recompute` never changes. `commit_move` calls
`translate_layer_active` with the view's `gpu_compute`, then converts the
already-current `doc.composite` to a `QImage` (the existing `buffer_to_image`
fast path, 42 ms) instead of the full `document_to_image`.

- *Why:* `commit_move` was CPU-bound for the same reason the compositor was; once
  2-D dispatch works, routing the commit through the active backend removes the
  dominant 670 ms.
- *Alternatives:* a separate GPU move kernel (rejected — movement is a rect
  shift, so re-compositing the shifted tree is the whole operation); making
  `translate_layer` GPU-aware in place (rejected — would entangle the oracle with
  the device).

### 3. Panels stop doing O(document) work per refresh (frozen)

- **`sample_argb`** reads the current cached image (`rust.image`) / current
  `doc.composite` at `(x, y)` instead of calling
  `current_buffer` → `composite_active`. `commit_move` already keeps
  `doc.composite` current after an edit; when it is stale the existing
  `image`/`document_to_image` path is the fallback. Sampling one pixel must not
  composite 16 M of them.
- **`layer_thumbnail(i, size)`** downsamples from the planar source at the
  target scale without first materializing a full-resolution RGBA `QImage` — a
  box/nearest gather at the thumbnail grid, then Qt scaling for the residual.
  The thumbnail's visual result is unchanged.
- **`HistogramPanel::recompute`** bins a ≤512² downsample of the current image
  instead of `convertToFormat` plus a full-resolution scan. A 512² (≤262 k
  pixels) sample is statistically indistinguishable from the full 16 M-pixel
  histogram at the panel's 256-bin, ~120 px resolution.
- **`begin_move_preview`** composites the document with the moved layer hidden in
  place: set `layer.visible = false`, `document_to_image`, restore
  `layer.visible = true` — no `doc.clone()`. The base image is identical; the
  60 ms clone and its extra composite disappear.

- *Why:* these are all reads of data that is already current; re-deriving it at
  document resolution is pure waste.
- *Ceiling:* the histogram downsample is a fixed 512² sample; if a future panel
  needs exact full-resolution counts, the full scan returns (ponytail: fixed
  ≤512² sample, exact counts if a panel ever needs them).

### 4. Oracles and parity unchanged (frozen)

`composite_rgba`, `recompute`, `translate_layer`, and `pictura_filters::apply`
are untouched and remain the test oracles. The GPU compositor and filter output
stay within ±1 LSB of the oracle; the CPU fallback stays byte-identical. Panel
sampling and histogram output stay visually identical. There is no public API or
behaviour change beyond the new `translate_layer_active` entry point.

### 5. Process (frozen)

Waves: (1) 2-D dispatch in the GPU compositor and GPU filters plus large-size
parity; (2) app — active-backend translate plus panel/thumbnail/histogram/preview
fixes; (3) 4000² timing evidence and self-test; (4) close-out.

## Risks / Trade-offs

- **2-D stride off-by-one** → the tail guard `i >= count` and a large-size
  parity test (4000²) catch any overlap or gap; the linear index is the same
  sequence the 1-D dispatch produced, only chunked.
- **Workgroup-product limit** → the new check rejects only above
  65535² × 64 ≈ 2.8×10¹⁴ px, far beyond any real document; a genuinely huge
  document still falls back to the CPU rather than erroring.
- **`sample_argb` reading a stale composite** → `commit_move` already refreshes
  `doc.composite`; the fallback recomputes when the cached image is not current.
  The parity/timing tests exercise the move-commit path.
- **Histogram downsample changes a bin count** → the panel's contract is a
  visual distribution, not exact per-pixel counts; a ≤512² sample preserves it.
  If exact counts are ever needed, the full scan returns.
- **Thumbnail downsampling quality** → the box gather plus residual Qt scale is
  visually equivalent at 24 px; a poor result is caught by the panel self-test.
- **`Document::clone()` remains 60 ms** → intentionally out of scope; it is the
  next ceiling and the trigger for history copy-on-write / tile diffs.

## Migration Plan

Additive. 2-D dispatch is an internal change to two dispatch helpers and the two
shaders' index lines; `translate_layer_active` is a new function; the panel
changes are local. The CPU oracle, parity contract, and CPU fallback are
unchanged. Rollback: restore the 1-D dispatch and the 1-D limit check, drop
`translate_layer_active` and the panel fast paths, and the prior CPU behaviour
returns; no public signature changes to revert.

## Open Questions

- Whether the histogram's 512² sample should be a strided sample or a Qt
  `scaled()` downsample — both preserve the visual; pick from the first timing
  run.
- Whether `commit_move`'s `buffer_to_image` can share the panel's cached image
  conversion — an optimization, not a contract.
- Whether 2-D dispatch alone removes the whole 4000² stall or whether the
  always-full-canvas ~265 ms allocation floor becomes the next target — decided
  by the wave-3 timing evidence.
