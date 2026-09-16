## Why

The M31–M33 plan assumed the 4000² canvas was **readback-bound** and made a
GPU-resident zero-copy present (M32) the next step. That premise is measured
false. A full 4000×4000 GPU composite (`composite_gpu_region`, 2 RGB pixel
layers, RTX 3090) costs ~254 ms and is split into:

| Phase | Time | Share |
|---|---:|---:|
| `build_source` (CPU planar source assembly + upload, 2 layers) | 146 ms | 58% |
| `to_pixel_buffer` (CPU de-interleave of the readback) | 35 ms | 14% |
| `mapped.to_vec()` (64 MB copy of the mapped readback) | 22 ms | 9% |
| `zero_canvas` | 18 ms | 7% |
| `build_mask` | 16 ms | 6% |
| readback submit + `poll(wait)` + `map` | 7 ms | 3% |
| GPU compute dispatch (submit is async) | 0.2 ms | ~0% |
| allocations | ~0.01 ms | ~0% |

The composite is dominated by **host-side CPU per-pixel work**, not by the GPU
and not by the readback: assembly + de-interleave + copy is ~203 ms (80%), the
64 MB readback is ~7 ms, and `buffer_to_image` alone is ~40 ms. A zero-copy
present would remove only ~57 ms of ~254 ms while the 146 ms source assembly
stays, and the present is already served from a cached `QImage` — so the Qt
Quick present is deferred.

Two interactive paths still composite the whole document for a small change:
`begin_move_preview` (293 ms) and `set_layer_visible`. `ImageView::paintEvent`
also re-scales the entire image through `painter.scale(zoom_)` on every paint, so
a pan or hover repaint resamples the full-resolution document on the CPU. M32
completes the region path for those two interactive mutations and caches the
presented image at the current zoom.

## What Changes

- **Move-preview base is region-composited.** `begin_move_preview` builds the
  preview base by region-compositing only the moved layer's document rectangle
  with that layer hidden into the existing cached canvas, instead of hiding the
  layer and compositing the whole document. Byte-identical to the previous full
  recomposite with the layer hidden.
- **Visibility toggle is region-composited.** `set_layer_visible` refreshes only
  the toggled layer's rectangle (a raster layer's `rect`; an adjustment layer's
  mask rect only when the mask bounds the effect) when that is provably equivalent
  to a full recomposite, and otherwise falls back to a full recomposite. The
  refreshed canvas is byte-identical to a full recomposite.
- **Present uses a zoom cache.** `ImageView` caches the document scaled to the
  current zoom and invalidates it when the image or the zoom changes, so a
  pan/hover repaint does not resample the full-resolution document. The result
  matches the previous transform-based paint.

## Capabilities

### New Capabilities

None. M32 completes existing capabilities.

### Modified Capabilities

- `document-canvas`: the Move-preview base and the layer visibility toggle are
  region-composited; and the canvas presents from a zoom cache.

## Impact

- `crates/pictura-app/src/cxxqt_object.rs` — `begin_move_preview` region base and
  `set_layer_visible` region path (with the full-recomposite fallback).
- `crates/pictura-app/cpp/image_view.{h,cpp}` — the zoom-cached present.
- `crates/pictura-app/cpp/main.cpp` — self-test checks for the move-preview base,
  the visibility region path, and the zoom cache.
- No new dependency. Deferred to later changes: full-composite throughput (M33),
  GPU-resident zero-copy present (M34), 256² tiles/LRU/mipmaps (M35),
  off-GUI-thread compute, cheap undo/redo + composite coherence, and the C++
  region blit / budget removal.
