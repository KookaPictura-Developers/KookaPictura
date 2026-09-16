## Context

Every canvas update goes through `document_to_image` → `current_buffer` →
`composite_active` → `buffer_to_image` (`crates/pictura-app/src/cxxqt_object.rs`).
M31 added `composite_region_active(doc, rect, gpu_enabled)` and moved Move and
paint onto a dirty-rect canvas cache. Two interactive paths still pay a
full-document composite, and the canvas re-scales the whole document on every
paint.

The measured 4000×4000 GPU composite (~254 ms, 2 RGB layers, RTX 3090) is
host-side, not GPU-side:

| Phase | Time | Share |
|---|---:|---:|
| `build_source` (CPU planar assembly + upload, 2 layers) | 146 ms | 58% |
| `to_pixel_buffer` (CPU de-interleave of the readback) | 35 ms | 14% |
| `mapped.to_vec()` (64 MB copy of the mapped readback) | 22 ms | 9% |
| `zero_canvas` | 18 ms | 7% |
| `build_mask` | 16 ms | 6% |
| readback submit + `poll(wait)` + `map` | 7 ms | 3% |
| GPU compute dispatch (async) | 0.2 ms | ~0% |
| allocations | ~0.01 ms | ~0% |

The GPU dispatch is negligible and the 64 MB readback is ~7 ms; host-side
per-pixel work (assembly + de-interleave + copy ≈ 203 ms, 80%) dominates. A
zero-copy present would remove ~57 ms of 254 ms while the 146 ms source assembly
stays. Separately: `begin_move_preview` is 293 ms, and
`ImageView::paintEvent` re-scales the full `image_` through `painter.scale(zoom_)`
on **every** paint.

Constraints: the CPU compositor (`composite_rgba`) is the frozen oracle, the GPU
path is ±1 LSB against it, `composite_active`/`composite_region_active` are the
full/region oracles, wgpu on Vulkan is the only backend, no new dependency, and
`docs/` is the long-form contract. The canvas stays the QWidget/QImage path; the
Qt Quick zero-copy present is deferred (M34).

## Goals / Non-Goals

**Goals:**

- Remove the whole-document composite from the two remaining interactive paths:
  `begin_move_preview` and `set_layer_visible` become region composites, each
  byte-identical to the full recomposite it replaces.
- Present from a zoom cache so a pan/hover repaint does not resample the
  full-resolution document, with a visual result equal to the transform draw.

**Non-Goals:**

- Persisting the composited buffer into `doc.composite`, cheap undo/redo, and the
  resulting composite/Save coherence — deferred to its own proposal.
- The C++ region blit (`ImageView::blitRegion`) and the removal of
  `REGION_REFRESH_BUDGET`; the per-pixel blit and its budget cap stay as the
  bounded fallback.
- Full-composite throughput (row-wise `copy_from_slice` source assembly, a fused
  planar readback, resident per-layer GPU source buffers) — M33.
- GPU-resident zero-copy present via Qt Quick — M34 (deferred).
- 256² GPU tiles + LRU + seam gutters + mipmaps, display-time LoD — M35
  (deferred).
- Off-GUI-thread compute; history copy-on-write / tile diffs.
- Region refresh for mutations that do not report a dirty rectangle; they keep
  the full recomposite.

## Decisions

### 1. Move-preview base is a region composite (frozen)

`begin_move_preview` currently hides the topmost pixel layer in place and calls
`document_to_image(doc)` (`cxxqt_object.rs:1093`), a full composite.

M32 builds `move_base` from the cached canvas plus a region composite of the
moved layer's rectangle:

1. Let `rect` be the topmost pixel layer's clamped `PsdRect`, and `image` the
   current cached full-document image (invariant: `image` is a full composite of
   the current document).
2. With the layer hidden in place, call
   `composite_region_active(doc, rect, gpu_compute)` and convert the result to a
   `QImage`.
3. Blit that region into a copy of `image` (the existing per-pixel region blit),
   store it as `move_base`; do **not** write it into `doc.composite` and do **not**
   touch the persistent cached `image`.
4. Restore `layer.visible = true`; cache `move_layer`, `move_x`, `move_y`,
   `move_opacity` as today.

- *Why:* hiding a pixel layer changes the composite only where the layer's source
  coverage is non-zero, which is inside its `rect`; compositing is per-pixel, so a
  region composite equals the full composite's slice there, and outside the rect a
  hidden and a visible layer composite identically. Therefore `move_base` equals
  the old full recomposite with the layer hidden.
- *Alternatives:* painting the layer over the base at commit (rejected — the
  base must equal the hidden composite for the drag preview to match the commit);
  a transient document clone (M29 removed it for cost).

### 2. Visibility toggle is a region composite or a full fallback (frozen)

`set_layer_visible` (`cxxqt_object.rs:702`) flips `layer.visible`, records, and
calls `recomposite()` (full). M32 flips the flag, records, then, when the toggle
is provably confined, calls `refresh_region(influence_rect(layer))`; otherwise it
falls back to `recomposite()`.

`influence_rect(layer)` is the rectangle outside which the toggle cannot change a
pixel:

- raster layer → its clamped `rect` (its channels exist only there);
- adjustment layer → its `mask.rect` only when the mask is enabled, carries
  `data`, and has `default_color == 0`; otherwise the whole document (an
  adjustment transforms the entire backdrop, and a mask with a non-zero
  `default_color`, or no data at all, applies outside its rect too);
- group → no bounded rectangle, so the full recomposite.

- *Why:* the same per-pixel independence argument. A bounded `influence_rect`
  keeps the byte-identity and still avoids the full document for the common raster
  case; the conservative full recomposite is correct, just not cheaper, and is
  named here so it is not mistaken for a bug.
- *Ceiling:* groups and unmasked adjustments still recomposite the full document
  (ponytail: conservative influence bound; tighten to a computed child union or a
  per-adjustment bound if it shows up).

### 3. Zoom-cached present (frozen)

`ImageView::paintEvent` (`image_view.cpp:209`) does `painter.translate(offset_)`,
`painter.scale(zoom_)`, then `drawImage` — resampling the full document each
paint. M32 adds a `QImage presentCache_` scaled at the current zoom:

- invalidated in `setImage`, `replaceImage`, `setZoom`, `fitOnScreen`,
  `actualPixels`, and `applyInitialView`;
- built lazily in `paintEvent` by rendering `image_` into `presentCache_` through
  the **same** painter transform and render hints the direct path used, so the
  pixels match;
- `paintEvent` then draws `presentCache_` at `offset_` with no scale, so a pan or
  hover repaint is a single un-scaled blit.

Because the cache scales with zoom, it may be large at high zoom; caching is
skipped (falling back to the transform draw, which is visually identical) when the
scaled footprint exceeds a fixed pixel bound. The checkerboard and the
move-preview/overlay paths are unchanged.

- *Why:* pan/hover is the common repaint, and the previous code resampled the
  whole document for every one. The cache turns that into a copy.
- *Ceiling:* the whole document is still re-scaled on invalidation (a region
  update changes `image_`), so an interactive move re-scales once per update;
  `ponytail:` invalidate-only-what-changed if a profile demands it.
- *Alternative:* `QPixmap` instead of `QImage` (server-side backing). Either is
  correct; pick from the first profile.

### 4. Oracles, equivalence, and checks (frozen)

The CPU compositor, `composite_active`, `composite_region_active`, the ±1 LSB GPU
parity contract, and `pictura_filters::apply` are unchanged. Byte-identity
arguments: per-pixel independence makes a region composite equal the full
composite's slice (M31), so (1) the move-preview base equals a full composite with
the layer hidden, and (2) a bounded visibility-toggle refresh equals a full
recomposite while the unboundable cases fall back to one. The zoom cache is
validated by (3) rendering with and without the cache and asserting pixel
equality.

### 5. Process (frozen)

Waves: (1) brief and frozen interfaces; (2) app region completion (move-preview,
visibility); (3) present zoom cache; (4) self-test and evidence; (5) close-out.

## Deferred to later milestones

- **Cheap undo/redo + composite coherence/save** — its own proposal. Persisting
  the rendered composite into `doc.composite` changes what `pictura_codec::
  write_psd` serializes and needs its own design (capture ordering, backend
  parity across capture/restore, and the Save contract). Until then every other
  mutation keeps the full `recomposite`, and M32 makes no claim about Save writing
  a stale composite.
- **C++ region blit / budget removal** — `ImageView::blitRegion`
  (`QPainter` + `CompositionMode_Source`) replacing the per-pixel
  `QImage::set_pixel_color` loop, and dropping `REGION_REFRESH_BUDGET`. The
  per-pixel blit and its budget cap stay as a bounded fallback.
- **Zoom-level details beyond the single cached scaled image** — LoD, mipmaps,
  and per-level caches are M35; M32 caches exactly one scaled image at the
  current zoom.

## Risks / Trade-offs

- **`influence_rect` under-report.** A too-small region leaves stale pixels. The
  conservative fallback (full recomposite) is used for adjustment/group layers,
  and the self-test compares a toggled canvas to a full recomposite.
- **GPU/CPU toggle across capture.** A stored composite captured on one backend
  and compared to a recomposite on the other differs by ≤1 LSB, not 0; documented
  rather than overclaimed.
- **Preview/selection layer mismatch.** `move_base` is the hidden composite; the
  live callback still source-overs the layer (non-Normal blends/masks are not
  reproduced mid-drag) — unchanged from M29, and the committed image is exact.
- **Zoom-cache memory at high zoom.** A full-document scaled cache can be huge;
  the fixed-bound fallback keeps it bounded and visually identical.

## Migration Plan

Additive. The zoom cache is new; `begin_move_preview` and `set_layer_visible`
swap the full path for the region path (with the visibility fallback). The
per-pixel region blit, the `REGION_REFRESH_BUDGET` cap, `recomposite`,
`doc.composite` write-back, and the history/undo path are unchanged. The oracles,
the parity contract, and all other public behaviour are unchanged. Rollback:
restore the full `document_to_image`/`recomposite` calls and the direct transform
draw, and the previous behaviour returns.

## Open Questions

- Whether `influence_rect` is computed inline in `set_layer_visible` or as a
  helper — either preserves the oracle; pick the smaller diff.
- Whether the zoom cache is a `QImage` or a `QPixmap` — pick from the first
  profile.
