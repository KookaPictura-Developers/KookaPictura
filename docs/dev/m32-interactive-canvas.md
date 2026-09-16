# M32 — Interactive-path region completion and present caching

Goal: remove the whole-document composite from the remaining interactive paths
and stop re-scaling the full-resolution canvas on every repaint.

OpenSpec change `m32-interactive-canvas` (MODIFIED `document-canvas`). It corrects
the M31–M33 plan's readback premise; the plan and `dev/STATE.md` are updated in the
same change.

## Measured evidence

A full 4000×4000 GPU composite (`composite_gpu_region`, 2 RGB pixel layers,
RTX 3090) costs ~254 ms, split into:

| Phase | Time | Share | What it is |
|---|---:|---:|---|
| `build_source` | 146 ms | 58% | CPU planar source assembly + upload of both layer planes |
| `to_pixel_buffer` | 35 ms | 14% | CPU de-interleave of the readback into planar `PixelBuffer` |
| `mapped.to_vec()` | 22 ms | 9% | 64 MB copy of the mapped readback |
| `zero_canvas` | 18 ms | 7% | |
| `build_mask` | 16 ms | 6% | |
| readback submit + `poll(wait)` + `map` | 7 ms | 3% | the actual 64 MB transfer |
| GPU compute dispatch (submit is async) | 0.2 ms | ~0% | |
| allocations | ~0.01 ms | ~0% | |

The composite is dominated by **host-side CPU per-pixel work**, not the GPU and
not the readback: assembly + de-interleave + copy is ~203 ms (80%), the 64 MB
readback is ~7 ms, and `buffer_to_image` alone is ~40 ms. A zero-copy present
would remove only ~57 ms of ~254 ms while the 146 ms source assembly stays.

Other measured interactive costs: `begin_move_preview` 293 ms, and
`ImageView::paintEvent` re-scales the whole `image_` through
`painter.scale(zoom_)` on every paint, so a pan or hover repaint resamples the
full-resolution document on the CPU.

## Scope

1. **Move-preview base is region-composited.** `begin_move_preview` builds the
   preview base by region-compositing only the moved layer's clamped document
   rectangle with that layer hidden into the existing cached canvas, instead of
   hiding the layer and compositing the whole document (293 ms). The base is
   byte-identical to the previous full recomposite with the layer hidden, and the
   document's stored `composite` and the persistent cached canvas are untouched.
2. **Visibility toggle is region-composited.** `set_layer_visible` refreshes only
   the toggled layer's rectangle when that is provably equivalent to a full
   recomposite — a raster layer's clamped `rect`; an adjustment layer's mask rect
   only when the mask is enabled, has data, and has `default_color == 0` — and
   otherwise falls back to a full recomposite (groups and unmasked/non-zero-mask
   adjustments). The result is byte-identical to a full recomposite.
3. **Zoom-cached present.** `ImageView` caches the document scaled at the current
   zoom and invalidates on image/zoom change, so a pan/hover repaint does not
   resample the full-resolution document. The pixels match the previous
   transform-based paint.
4. **Evidence.** Self-tests assert each byte-identity above and the zoom cache; a
   timing check confirms a region move stays materially cheaper than the full
   composite.

## Design by item

### 1. Move-preview base (`begin_move_preview`)

Today (`crates/pictura-app/src/cxxqt_object.rs:1093`): hide the topmost pixel
layer in place, `document_to_image(doc)`, restore. M32:

1. `rect` = the topmost pixel layer's clamped `PsdRect`; `image` = the current
   cached full-document image (invariant: it is a full composite of the document).
2. With the layer hidden in place, `composite_region_active(doc, rect, gpu_compute)`
   and convert to a `QImage`.
3. Blit the region into a copy of `image` (the existing per-pixel region blit) and
   store it as `move_base`; do **not** write `doc.composite` and do **not** touch
   the persistent cached `image`.
4. Restore `layer.visible = true`; cache `move_layer`/`move_x`/`move_y`/
   `move_opacity` as today.

Equivalence: hiding a pixel layer changes the composite only where its source
coverage is non-zero, i.e. inside its `rect`; compositing is per-pixel, so the
region composite equals the full composite's slice there, and outside `rect` a
hidden and a visible layer composite identically. Therefore the base equals the
old full recomposite with the layer hidden.

### 2. Visibility toggle (`set_layer_visible`)

Today (`cxxqt_object.rs:702`): flip `visible`, record, `recomposite()` (full).
M32: flip, record, then `refresh_region(influence_rect(layer))` when the
influence rectangle is a provable bound, otherwise `recomposite()`.

`influence_rect`: raster layer → its clamped `rect`; adjustment layer → its
`mask.rect` only when the mask is enabled, has data, and has
`default_color == 0`, otherwise no bound (an adjustment transforms the whole
backdrop, and a mask with a non-zero `default_color`, or no data, applies outside
its rect too); group → no bound (a child adjustment covers it and a pass-through
group composites onto the whole canvas). The conservative full recomposite is
correct but not cheaper; it is named so it is not mistaken for a bug.

### 3. Zoom-cached present

`ImageView::paintEvent` (`cpp/image_view.cpp:209`) translates by `offset_`, scales
by `zoom_`, then `drawImage`. M32 adds `presentCache_` (the document scaled at the
current zoom), invalidated by `setImage`, `replaceImage`, `setZoom`,
`fitOnScreen`, `actualPixels`, and `applyInitialView`; built lazily in
`paintEvent` through the same transform and render hints as the direct draw, then
presented un-scaled at `offset_`. Caching is skipped (direct transform draw,
visually identical) when the scaled footprint exceeds a fixed pixel bound.

## Frozen interfaces

- `begin_move_preview` returns a base byte-identical to a full composite with the
  topmost pixel layer hidden; it composites only that layer's clamped rect and
  leaves `doc.composite` and the cached canvas unchanged.
- `set_layer_visible` region-refreshes the provable `influence_rect(layer)` and is
  byte-identical to a full recomposite; unboundable toggles fall back to a full
  recomposite.
- `ImageView` presents from a zoom cache invalidated on image/zoom change; pan and
  hover repaints do not resample the full-resolution document.
- The per-pixel region blit, `REGION_REFRESH_BUDGET`, `recomposite`, the
  `doc.composite` write-back, and the history/undo path are unchanged.
- Oracles unchanged: `composite_active`, `composite_region_active`, the CPU
  compositor, the ±1 LSB GPU parity contract, and `pictura_filters::apply`.

## Deferred to later milestones

- **Cheap undo/redo + composite coherence/save** — its own proposal: persisting
  the rendered composite into `doc.composite` changes what `write_psd` serializes
  and needs its own design.
- **C++ region blit / budget removal** — `ImageView::blitRegion`
  (`QPainter` + `CompositionMode_Source`) replacing the per-pixel
  `QImage::set_pixel_color` loop, and dropping `REGION_REFRESH_BUDGET`. The
  per-pixel blit and its budget cap stay as a bounded fallback.
- **M33 — full-composite throughput.** Row-wise/`copy_from_slice` source assembly
  instead of the per-pixel loop; a fused planar readback that de-interleaves
  directly from the mapped slice and skips the packed `Vec`; resident per-layer
  GPU source buffers across a composite session (targets 146 + 35 + 22 ms).
- **M34 (deferred) — GPU-resident zero-copy present** via Qt Quick
  (`QQuickRhiItem` / `QQuickWindow::createTextureFromRhiTexture` /
  `QQuickGraphicsDevice::fromDeviceObjects`); `QRhiWidget` cannot adopt the wgpu
  device. Deferred because the measurement shows it removes only ~57 ms of
  ~254 ms.
- **M35 (deferred) — 256² GPU tiles + LRU + seam gutters + mipmaps**, plus
  display-time LoD; this is where zoom-level details beyond the single cached
  scaled image belong.
- Off-GUI-thread compute; history copy-on-write / tile diffs.

## Verification

- `cargo test --workspace`; `cmake --build build`.
- Self-test: move-preview base equals a full composite with the layer hidden; a
  bounded visibility toggle equals a full recomposite and an unboundable one falls
  back; a pan repaint uses the zoom cache and matches the transform draw.
- `xvfb-run -a ./build/pictura --self-test` (and the fixture variant).
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`.
