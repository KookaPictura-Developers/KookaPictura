## Context

Every canvas update goes through `document_to_image` →
`current_buffer` → `composite_active` → `buffer_to_image`
(`crates/pictura-app/src/cxxqt_object.rs`): the whole layer stack is composited,
the whole packed RGBA buffer is read back to the CPU, and the whole document is
converted to a `QImage`. At 4000² the composite is 16.7 M pixels and the readback
~64 MB, and it happens for any change — `commit_move` after a one-pixel drag and
`paint_dab` after every stroke sample. M29 already removed the two *per-composite*
costs: 2-D GPU dispatch lifts the ~4.19 MP ceiling and `translate_layer_active`
commits through the active backend. What is left is that the composite is always
full.

The document's dirty information already exists but is unused for rendering:

- `translate_layer`/`translate_layer_active` move a layer **rect** (no pixels
  move), so the only pixels that can change are the vacated rect and the newly
  covered rect.
- `Stroke` accumulates a `dirty: PsdRect` and already exposes `dirty()` in
  document coordinates; a paint sample changes only the dabs it placed.

Compositing is per-pixel: `composite_pixels`/`blend_into` on the CPU and the
WGSL `cs_main` on the GPU read only the pixel's own source sample and the same
pixel's running canvas. Adjustment layers, masks, opacity, groups, and clipping
are all per-pixel too. Disjoint regions therefore cannot affect each other, so a
sub-rect composite is byte-identical to the corresponding slice of the full
composite. This mirrors GIMP's projection model, where `gegl_node_blit` evaluates
the graph per tile and tiles are independent.

Constraints: the CPU compositor (`composite_rgba`) is the frozen oracle, the GPU
path is ±1 LSB against it, `composite_active` is the full-document oracle; wgpu on
Vulkan is the only backend; no new dependency; `docs/` is the long-form contract.
The canvas is still the CPU `QImage` path (the zero-copy GPU present is M32).

## Goals / Non-Goals

**Goals:**

- Composite only the requested (clamped) document rectangle and return a
  rectangle-sized buffer, through the active backend, byte-identical to the
  corresponding slice of the full composite for separable, non-separable,
  adjustment, group and masked scenes.
- On the GPU, dispatch only the region's pixels, size the per-layer source/mask
  buffers to the region, and read back only the region.
- Cache the composited document in `PictureView` and update only a reported dirty
  rectangle: `old_layer_rect ∪ new_layer_rect` for a Move commit, the dab's
  bounding box for a paint dab.
- Keep the CPU oracle, the GPU ±1 LSB parity contract, and the full
  `composite_active` oracle unchanged.
- Ship a self-test that a region-refreshed canvas equals a full recomposite, and
  timing evidence that a small dirty rect is cheaper than the full path.

**Non-Goals:**

- Display-resolution proxies / LoD (M32/M33), GPU-resident present without
  readback (M32), 256² tiles with LRU and seam gutters (M33).
- Off-GUI-thread compute.
- History copy-on-write / tile diffs: `Document::clone()` (60 ms at 4000²) stays
  on the `commit_move`/stroke-commit path.
- Region refresh for mutations that do not report a dirty rectangle; they keep
  the full recomposite.

## Decisions

### 1. Region API and the byte-identity contract (frozen)

```rust
pub fn composite_region_active(
    doc: &Document,
    rect: PsdRect,
    gpu_enabled: bool,
) -> (PixelBuffer, Backend)
```

- `rect` is the **existing** `pictura_core::PsdRect` (`top`/`left`/`bottom`/
  `right`, document pixel space, `right`/`bottom` exclusive), taken by value. No
  `DocRect` type is introduced: every layer rect already uses this type and it
  carries exactly these semantics.
- `rect` is clamped to `[0, width] × [0, height]` before any work. An empty
  intersection returns a zero-dimension `PixelBuffer` (never panics).
- Otherwise it returns a `PixelBuffer` of `clamped.width() × clamped.height()`,
  4-channel planar straight-alpha, and reports the `Backend` that ran.
- The returned buffer MUST be byte-identical to the corresponding sub-rectangle
  of `composite_active(doc, gpu_enabled)`. `composite_active` keeps its signature
  and observable output and remains the oracle.

- *Why:* per-pixel independence makes the region result exactly the full
  result's slice, so the change is provable against code that already exists
  rather than a visual approximation. Region-identical output makes the app's
  cached canvas valid by construction.
- *Alternatives:* a mask/region parameter on a still-full composite (rejected —
  does not reduce the readback, which is the point); tiles (deferred to M33 —
  no measured need at these sizes).

### 2. GPU region dispatch (frozen)

`Gpu` is generalized from "the whole document" to "a region of the document":

- The packed canvas is allocated for `region_w * region_h` pixels, not
  `doc.width * doc.height`; `count = region_w * region_h`.
- The params uniform threads the region origin and width: `region_x0`,
  `region_y0`, `region_w` (the region's row width). The existing workgroup
  linearization `stride` (`gx * 64`) is unchanged; `grid_2d(count, limit)` is
  unchanged.
- `cs_main` maps invocation `i` to region-local `(rx, ry)`, then to document
  `(x, y) = (region_x0 + rx, region_y0 + ry)`. `sample_src` tests the layer rect
  in document coordinates and indexes source planes / the region canvas in
  region-local coordinates; `blend` reads and writes only the same pixel's canvas
  word, so no cross-region read occurs.
- `build_source` and `build_mask` clamp the layer (or group/adjustment) rect to
  the region and size their planes to `region_w * region_h`. A group's inner
  canvas is region-sized and is bound packed as the source, indexed region-local.
- `read_canvas` copies and maps only `region_w * region_h * 4` bytes;
  `to_pixel_buffer` de-interleaves at the region dimensions.

`composite_gpu`/`composite_active` keep their signatures and observable output;
the full canvas is the same kernel with the region equal to the full rect.
Whether `composite_active` delegates to the region kernel with the full rect or
keeps its own allocation is an implementation detail — its output cannot change.

- *Why:* the region reduces both the compute dispatch and, more importantly, the
  readback by the region/canvas area ratio. Keeping one shader with a region
  origin keeps the parity reasoning identical to the full-canvas path and avoids
  a second pipeline.
- *Alternatives:* a per-tile shader with tile-local coordinates and a gutter
  (rejected — gutters are an M33 seam concern, not needed for CPU-deterministic
  pixel math); a full dispatch with a discard predicate (rejected — no readback
  reduction).

### 3. CPU region loop (frozen)

The CPU region path runs the same per-pixel compositing loop as `composite_rgba`,
restricted to the clamped region and preserving document coordinates, then
extracts the region's four planes into a region-sized `PixelBuffer`. The
region-limited iteration is shared by pixel layers, adjustment layers, and group
canvas merges, so the only difference from the full oracle is which pixels are
visited. `composite_rgba` continues to visit everything and remains the oracle.

- *Why:* document coordinates are preserved, so no origin remapping is needed and
  byte-identity with the full oracle is direct.
- *Ceiling:* the CPU accumulator is still document-sized (`Canvas::new(w, h)`);
  only the visited pixels shrink. If CPU memory ever matters, the accumulator can
  become region-sized with an origin offset (ponytail: document-sized CPU
  accumulator, region-sized only if a memory profile demands it).

### 4. Cached canvas and `refresh_region` (frozen)

`PictureViewRust.image` already holds the full-document composited `QImage`; M31
treats it as the canvas cache (no new field). `refresh_region`:

1. clamps the rect to the source document; an empty rect is a no-op (no
   composite, no signal);
2. calls `composite_region_active(source, rect, gpu_compute)`;
3. writes the returned planes back into `source.composite` at the rect origin, so
   `doc.composite` stays byte-equal to a full composite (the M29 invariant that
   `sample_argb` and the panels rely on);
4. converts the region buffer to a `QImage` and blits it into the cached canvas at
   `rect`'s origin with a scanline copy (both images are tightly packed
   `Format_RGBA8888`, so it is `region_w * 4` bytes per row);
5. emits `changed`.

The source is the app document for a Move and the stroke's working document for a
paint dab. The stroke updates only the cached canvas live; its `composite` is
refreshed by the existing full `recomposite` at `end_paint`.

- *Why:* the cache is already the image the Qt canvas paints; keeping
  `doc.composite` and the cached image region-current means every existing reader
  sees a consistent full image without a full recomposite.
- *Alternatives:* a separate canvas buffer from `doc.composite` (rejected — two
  caches to keep coherent); updating only the `QImage` and leaving
  `doc.composite` stale (rejected — breaks the M29 panel/sample invariant).

### 5. Move and paint invalidation rectangles (frozen)

- **Move commit.** `commit_move(dx, dy)` reads the topmost pixel layer's `rect`
  before the shift, applies the rect/mask shift as `translate_layer` does, reads
  the rect after, and calls `refresh_region(old ∪ new)`. `old ∪ new` covers both
  the vacated and newly covered pixels; content spilling outside the canvas is
  cropped by the clamp. `translate_layer`/`translate_layer_active` remain the
  oracle and test path and are no longer on `commit_move`'s path. History
  (`record("Move Layer")`) is unchanged.
- **Paint dab.** On a sample that changed pixels, `paint_dab` gets the sample's
  dirty document rect and calls `refresh_region(dab_rect)` against the stroke's
  working document. `Stroke` exposes the last sample's dirty rectangle (`Stroke`
  already tracks `dirty`/`dirty_doc`), so a dab does not refresh the whole
  accumulated stroke. `end_paint` keeps its full `recomposite`.
- Everything else (`recomposite`) stays a full composite for now; the region path
  is extended to more mutations incrementally.

- *Why:* a Move is a pure rect shift with no resampling, so no pixel outside
  `old ∪ new` can change; a dab changes only its own coverage box. These two are
  the interactive paths the M31 plan targets, and both already expose their exact
  bounds.

### 6. Oracles, parity, and correctness checks (frozen)

`composite_rgba`, `recompute`, `translate_layer`, `translate_layer_active`, and
the ±1 LSB GPU parity contract are unchanged; `composite_active` remains the
full-document oracle. Region compositing is asserted byte-identical to the
corresponding slice of `composite_active`; the existing GPU parity test ties that
slice to the CPU oracle within ±1 LSB. The CPU fallback stays byte-identical. A
region-refreshed canvas is asserted equal to a full recomposite after a sequence
of Move/paint invalidations.

### 7. Process (frozen)

Waves: (1) brief and frozen interfaces; (2) region render API, GPU region
dispatch, byte-identity parity; (3) app cached canvas and dirty-rect Move/paint;
(4) self-test and timing evidence; (5) close-out.

## Risks / Trade-offs

- **Off-by-one in the region or dirty union** → the byte-identity test probes
  every pixel of the region against the full slice; `right`/`bottom` exclusive
  and the clamp are asserted, so a wrong edge shows up immediately.
- **GPU region origin/stride wrong** → `sample_src`'s document-coordinate test
  plus the byte-identity test catch a wrong origin; the full-rect region
  (region == canvas) is the degenerate case that must equal `composite_active`.
- **Cached canvas diverges from a full composite** → `refresh_region` patches
  `doc.composite` and the `QImage` from the same buffer; the sequence test
  compares the result with a full recomposite.
- **A mutation under-reports its dirty rect leaves stale pixels** → only Move and
  paint take the region path and both report exact bounds; every other mutation
  falls back to the full recomposite, so the exposure is bounded and documented.
- **Move content outside `old ∪ new`** → impossible for a pure rect shift (no
  resampling); the layer mask rect shifts with the layer.
- **History clone stays** → `record` still clones the document; intentionally out
  of scope and noted as the next bottleneck.
- **CPU accumulator stays document-sized** → the region path reduces visited
  pixels, not the CPU allocation; acceptable at M31 sizes and named as the
  ceiling.

## Migration Plan

Additive. `composite_region_active` and `refresh_region` are new; `commit_move`
and `paint_dab` swap the full conversion for the region path; `Stroke` gains a
last-dirty accessor. The CPU oracle, the parity contract, and all other public
behaviour are unchanged. Rollback: drop `refresh_region` and restore the
`translate_layer_active`/`document_to_image` calls, and the previous
whole-document behaviour returns.

## Open Questions

- Whether `composite_active` should delegate to the region kernel with the full
  rect or keep its own allocation — either preserves the oracle; pick whichever
  keeps the diff smaller.
- Whether `refresh_region`'s canvas blit should be a scanline copy or
  `QPainter::drawImage` — both are correct; pick from the first profile.
- Whether the paint path should also region-refresh the stroke's
  `working.composite` or only the cached canvas — the stroke's composite is not
  read until `end_paint`'s full recomposite, so either works.
