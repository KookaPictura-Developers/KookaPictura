## Why

A 4000×4000 layer move costs 3–5 s per update (release, RTX 3090). Profiling at
4000² attributes it:

- `translate_layer` → `recompute` → `composite_rgba` (the CPU composite) is
  670 ms; `commit_move` end-to-end 745 ms, `begin_move_preview` 610 ms.
- `InfoPanel::refresh` re-composites every cycle (`sample_argb` → `current_buffer`
  → `composite_active`, ~0.44 s), so the panel refresh total is ≈ 1.3 s/cycle.
- The histogram does `convertToFormat` plus a full 16 M-pixel scan (145 ms);
  `layer_thumbnail(24)` builds a full-size RGBA image before scaling (94 ms per
  layer).

Two root causes:

1. **The GPU is disabled above ~4.19 MP.** `Gpu::new` rejects when
   `n.div_ceil(64) > max_compute_workgroups_per_dimension` (65535). 4000² is
   16.7 M pixels ⇒ 250 000 1-D workgroups, so `composite_active(..., true)`
   silently returns the CPU path. At 1024² (4.19 MP) the GPU is already 3.4×
   faster, so the limit — not the hardware — is the crossover. `gpu_filter` has
   the same 1-D ceiling (`count.div_ceil(64) > max_compute_workgroups_per_dimension`).
2. **The panels do O(document) work on every refresh.** Sampling one pixel
   colour re-composites the whole document; the histogram scans every pixel; the
   thumbnail scales after materializing a full-resolution image; the move preview
   clones and re-composites the document to obtain a base image.

M29 lifts both GPU ceilings with 2-D dispatch, commits movement through the
active backend, and makes the panel refresh cheap.

## What Changes

- **2-D compute dispatch in place.** `crates/pictura-render/src/gpu.rs` and
  `crates/pictura-render/src/gpu_filter.rs` dispatch with
  `dispatch_workgroups(gx, gy, 1)` and derive the linear index from
  `gid.x + gid.y * (grid_x * 64)`, with the row stride passed in the uniform.
  The 1-D workgroup-count rejection is replaced by a 2-D product check
  (65535² workgroups ≈ 2.8×10¹⁴ px). No second pipeline, no new binding, no
  shader-architecture change; the packed `u32` data path and alpha rule are
  unchanged.
- **Commit through the active backend.** A GPU-aware
  `translate_layer_active(doc, dx, dy, gpu_enabled)` (or equivalent) lets
  `commit_move` composite on the GPU. The CPU `translate_layer`, `recompute`,
  and `composite_rgba` remain the oracles and the test path.
- **Panels stop doing O(document) work per refresh.** `sample_argb` reads the
  already-current cached image/`doc.composite` instead of re-compositing;
  `layer_thumbnail` downsamples without building a full-size RGBA image; the
  histogram is computed from a ≤512² downsample (visually identical to the full
  scan); `begin_move_preview` hides the moved layer in place (composite, restore)
  rather than `doc.clone()`.
- **Parity is unchanged.** Compositing stays ±1 LSB against the CPU oracle, the
  CPU fallback stays byte-identical, and there is no public behaviour change.
  Panel sampling and histogram values remain visually identical.
- **Performance expectation.** A 4000² layer move commits well under the 3–5 s
  CPU baseline on a GPU host, and the fixed per-refresh panel cost drops from
  ~1.3 s to the cache-read/downsample cost.

## Capabilities

### New Capabilities

None. M29 extends existing capabilities.

### Modified Capabilities

- `gpu-compositing`: the compute dispatch is 2-D, so the compositor runs at
  4000² and beyond instead of silently falling back to the CPU above ~4.19 MP;
  the linear-index derivation and workgroup-limit check change, the ±1 LSB parity
  contract and the CPU oracle do not.
- `gpu-filter-acceleration`: the filter dispatch is 2-D, lifting the same
  ~4.19 MP ceiling (1-D `count.div_ceil(64)` rejection) for the accelerated
  filter kernels, with byte-identical CPU fallback preserved.
- `info-histogram-panel`: `InfoPanel` samples the already-current cached image
  instead of re-compositing the document; the histogram bins a ≤512² downsample
  instead of a full-resolution scan, with the same visual output.
- `layers-panel`: layer thumbnails are produced by downsampling the source
  without materializing a full-size RGBA image.
- `document-canvas`: a document edit that goes through
  `translate_layer_active` composites on the active backend, and the move
  preview hides the moved layer in place instead of cloning the document.

## Impact

- `crates/pictura-render/src/gpu.rs` — 2-D dispatch and row stride in the blend
  shader's `Params`; 2-D product limit check in `Gpu::new` replacing the 1-D
  `n.div_ceil(64)` rejection.
- `crates/pictura-render/src/gpu_filter.rs` — 2-D dispatch, row stride in
  `Params`, and the 2-D product limit check replacing the 1-D
  `count.div_ceil(64)` rejection.
- `crates/pictura-render/src/document_ops/` — GPU-aware `translate_layer_active`
  alongside the unchanged CPU `translate_layer`; `recompute` stays the CPU
  oracle.
- `crates/pictura-app/src/cxxqt_object.rs` — `commit_move` uses the active
  backend; `sample_argb` reads the cached image; `layer_thumbnail` downsamples
  directly; `begin_move_preview` no longer clones the document.
- `crates/pictura-app/cpp/panels/histogram_panel.cpp` — histogram bins a ≤512²
  downsample.
- Tests — large-size (4000²) compositor and filter parity plus a 4000² move
  timing check; the app `--self-test` reports the backend and size.
- No new dependency; the CPU compositor and `pictura_filters::apply` are
  unchanged oracles.
