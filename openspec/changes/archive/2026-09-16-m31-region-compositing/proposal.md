## Why

Every canvas update composites the **whole** document, reads the whole packed
RGBA buffer back to the CPU, and converts it to a `QImage`
(`document_to_image` → `current_buffer` → `composite_active` →
`buffer_to_image`, `crates/pictura-app/src/cxxqt_object.rs`). At 4000² that is a
16.7 M-pixel composite plus a ~64 MB readback per update, and it runs for any
change however small: a one-pixel paint dab recomposites and transfers the whole
document. M29 made the composite itself fast (2-D GPU dispatch, active-backend
commit); what remains is that it is always done in full.

Only the changed rectangle is needed. Mature editors compute a damage region and
re-render only that region; GIMP's projection documents this model
(`gegl_node_blit` per tile). Because compositing is per-pixel — each pixel's
blend reads only its own source sample and the same pixel's running canvas —
disjoint regions do not affect each other, so a sub-rectangle composite is
**byte-identical** to the corresponding slice of the full composite. That makes
the dirty-rect path directly checkable against the existing oracle.

## What Changes

- **Region render API.** `pictura_render::composite_region_active(doc, rect,
  gpu_enabled) -> (PixelBuffer, Backend)` composites only the clamped `rect`
  through the active backend and returns a `rect`-sized buffer, byte-identical to
  the corresponding sub-rectangle of `composite_active(doc, gpu_enabled)` for
  separable, non-separable, adjustment, group and masked scenes. The
  full-document `composite_active` stays and remains the oracle.
- **GPU region dispatch.** The compute dispatch covers only the region's pixels;
  the region origin/size and the row layout are threaded through the shader, the
  per-layer source and mask buffers are sized to the region, and only the region
  is read back.
- **CPU region loop.** The same compositing loop restricted to the region
  (document coordinates preserved), so byte-identity with the full oracle is
  direct.
- **Dirty-region canvas cache.** `PictureView` keeps the full-document canvas
  `QImage` it already holds as the cache. `refresh_region(rect)` composites the
  rect through the active backend, blits it into the cached canvas at the rect
  origin, and emits `changed`. The Move commit invalidates
  `old_layer_rect ∪ new_layer_rect`; a paint dab invalidates the dab's bounding
  box. Other mutations keep the full recomposite for now (a documented
  incremental extension).

## Capabilities

### New Capabilities

None. M31 extends existing capabilities.

### Modified Capabilities

- `gpu-compositing`: adds region compositing
  (`composite_region_active`) that composites and reads back only a clamped
  document rectangle, byte-identical to the corresponding slice of the full
  composite across separable, non-separable, adjustment, group and masked
  scenes; `composite_active` remains the full-document oracle.
- `document-canvas`: the canvas caches the composited document and, on a
  mutation that reports a dirty rectangle (Move: `old ∪ new`; paint dab: the dab
  bbox), recomposites and updates only that rectangle; a region-refreshed canvas
  is byte-identical to a full recomposite after a sequence of such mutations.

## Impact

- `crates/pictura-render/src/gpu.rs` — `composite_region_active`, region
  dispatch (region origin/size in the uniform), region-sized source/mask/canvas,
  region-only readback.
- `crates/pictura-render/src/lib.rs` — the CPU region loop; `composite_rgba` and
  `composite_active` remain the oracles.
- `crates/pictura-render/src/document_ops/crop.rs` — the Move path drives the
  region refresh from the dirty union of the shifted rect; `translate_layer` and
  `translate_layer_active` stay for the oracle and tests.
- `crates/pictura-app/src/cxxqt_object.rs` — `refresh_region`, the `commit_move`
  dirty union, the `paint_dab` dab bbox, and the cached-canvas update in place of
  `document_to_image` per update.
- `crates/pictura-paint/src/stroke.rs` — expose the last sample's dirty
  rectangle so a paint dab reports its own bounding box.
- No new dependency. The CPU compositor and the GPU ±1 LSB parity contract are
  unchanged. Display-resolution proxies / LoD (M32/M33), GPU-resident present
  without readback (M32), 256² tiles (M33), and off-GUI-thread compute are
  deferred.
