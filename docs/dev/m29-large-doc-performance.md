# M29 — Large-document performance

Goal: make a 4000×4000 layer move interactive. Today it costs 3–5 s per update
(release, RTX 3090). Profiling at 4000² attributes the cost:

| cost | ms @ 4000² |
| --- | ---: |
| `translate_layer` → `recompute` → `composite_rgba` (CPU composite) | 670 |
| `commit_move` end-to-end | 745 |
| `begin_move_preview` end-to-end | 610 |
| `InfoPanel::refresh` (re-composites; panel total ≈ 1.3 s/cycle) | 440 |
| histogram `convertToFormat` + full 16 M-pixel scan | 145 |
| `layer_thumbnail(24)` (full-size RGBA before scaling) | 94/layer |
| `Document::clone()` (history capture) | 60 |
| `buffer_to_image` | 42 |

Two independent faults compound:

1. **The GPU is dead above ~4.19 MP.** `Gpu::new` rejects when
   `n.div_ceil(64) > max_compute_workgroups_per_dimension` (65535). 4000² is
   16.7 M pixels ⇒ 250 000 1-D workgroups, so `composite_active(..., true)`
   silently falls back to the CPU. At 1024² (4.19 MP) the GPU is already 3.4×
   faster than the CPU; the crossover is the workgroup-dimension limit, not the
   hardware. `gpu_filter` has the identical 1-D ceiling at
   `count.div_ceil(64) > max_compute_workgroups_per_dimension`.
2. **Panels re-scan the document every cycle.** `InfoPanel::refresh` calls
   `sample_argb` → `current_buffer` → `composite_active`, re-compositing the
   whole document for one pixel colour; the histogram materializes a full-size
   `QImage` and scans all 16 M pixels; `layer_thumbnail` builds a full-resolution
   RGBA image before scaling to 24 px. The move preview clones the document,
   hides the layer, and re-composites just to get a "base" image.

Fixes: 2-D compute dispatch (lifts both GPU ceilings), commit through the active
backend, and O(1)-per-refresh panels (sample the cached image, downsample
thumbnails, histogram a ≤512² downsample). OpenSpec change
`m29-large-doc-performance` (MODIFIED `gpu-compositing`, `gpu-filter-acceleration`,
`info-histogram-panel`, `layers-panel`, `document-canvas`).

## Scope

- **2-D dispatch, in place.** `gpu.rs` and `gpu_filter.rs` switch to
  `dispatch_workgroups(gx, gy, 1)` and derive the linear index from
  `gid.x + gid.y * (grid_x * 64)` with the row stride passed in the uniform. The
  1-D workgroup-count rejection is replaced by a 2-D product check
  (65535² workgroups ≈ 2.8×10¹⁴ px). No shader architecture change, no second
  pipeline, no new binding.
- **GPU-aware translate.** Add `translate_layer_active(doc, dx, dy, gpu_enabled)`
  (or equivalent) so `commit_move` composites on the active backend. The CPU
  `translate_layer` and `recompute` stay the oracles and the test path.
- **Panels stop doing O(document) work per refresh.** `sample_argb` reads the
  already-current cached image/`doc.composite`; `layer_thumbnail` downsamples
  without materializing a full-size RGBA buffer; the histogram bins a ≤512²
  downsample (visually identical); `begin_move_preview` hides the moved layer in
  place (composite, restore) instead of `doc.clone()`.
- **Parity is unchanged.** The CPU compositor and `pictura_filters::apply` remain
  oracles; compositing parity stays ±1 LSB; no public behaviour change.

## Out of scope (later milestones)

- History copy-on-write / tile diffs. `Document::clone()` (60 ms) becomes the
  next bottleneck once compositing is cheap; noted as the follow-up if PSB-size
  documents hit RAM.
- Off-GUI-thread compute; zero-copy present; GPU painting.

## Process

Waves: (1) 2-D dispatch in the GPU compositor and GPU filters plus large-size
parity; (2) app active-backend translate plus panel/thumbnail/histogram/preview
fixes; (3) 4000² timing evidence and self-test; (4) close-out.

## Verification

- `cargo test -p pictura-render` (large-size parity at 4000², filter parity) and
  `cargo test --workspace`
- A 4000² layer move commits in well under the 3–5 s CPU baseline on a GPU host
  (skip with a printed note when no adapter exists)
- `cmake --build build`; fixture and no-argument self-tests exit 0 with the
  backend report
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
