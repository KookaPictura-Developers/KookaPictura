# Canvas Compositing & Large-Layer Move Plan

- **Status:** dev note (research synthesis + M31–M35 roadmap; not an OpenSpec
  proposal yet — each milestone is proposed through the normal workflow first)
- **Scope:** how mature editors keep very large layers responsive while moving,
  painting, and zooming; how the current Kooka Pictura path diverges; and the
  staged plan to close the gap.
- **Companion specs:** `01-architecture/gpu-rendering-pipeline.md` (ARCH-006),
  `01-architecture/performance-targets.md` (ARCH-013),
  `01-architecture/threading-and-concurrency.md` (ARCH-005),
  `dev/STATE.md`, `dev/m29-large-doc-performance.md`, `dev/canvas-view-spec.md`,
  `crates/pictura-app/GPU-INTEROP-NOTES.md`.

This note records three sourced web-research passes plus the local M29 timing
evidence. It is deliberately implementation-agnostic about *where* tiles live:
the M31 step is a small change to the existing QWidget/QImage path, and M32/M33
only become worth doing when a measured cost says so.

---

## 1. The consensus architecture

Across GIMP/GEGL, Krita, Photoshop, Chromium, Affinity, Paint.NET, and Graphite
the same shape recurs:

- **Separate document state from display.** The document owns pixels; the screen
  shows a derived view. The two are not the same buffer.
- **Keep pixels resident.** The working set (or the visible part of it) stays in
  a cache — CPU RAM and/or VRAM — rather than being rebuilt from the layer stack
  on demand.
- **Compute a damage region.** A change produces a rectangle (or a set of
  tiles); only that region is re-composited. The damage taxonomy is explicit:
  paint invalidation (content changed) vs raster invalidation (raster cache
  stale) vs draw/expose damage (needs a new frame).
- **Re-render only dirty tiles/region**, and
- **present a texture without reading the whole document back.** The final image
  reaches the screen as a GPU texture; a CPU readback of the whole document is
  never on the interactive path.

The current Kooka Pictura path is the inverse of that last point: for every
update it composites the full document on the active backend, reads the full
packed RGBA buffer back to the CPU, and converts it to a `QImage`
(`document_to_image` → `current_buffer` → `composite_active` → `buffer_to_image`,
`crates/pictura-app/src/cxxqt_object.rs`).

An earlier draft of this note claimed the 4000² readback "dominates" the
composite and that "the transfer is" the bottleneck. That premise was measured
and is **false** — see §2.1. Host-side CPU per-pixel work dominates; the readback
is a small fraction.

### 1.1 GIMP / GEGL

- **Tiles ~128²**, held in a global LRU tile cache whose budget defaults to
  roughly `min(RAM, available)` with a 512 MB floor; swap-to-disk backends extend
  it.
- `GimpProjection` holds a **mipmap pyramid** of the projection, so zoomed-out
  views read a smaller level rather than a full-resolution composite.
- Dirty tracking uses `cairo_region`, aligned to 32² chunks, plus per-tile
  validate.
- `GimpTileHandlerValidate` blits **one tile-sized region per tile** through the
  GEGL graph (`gegl_node_blit`) — the graph is evaluated per tile, not wholesale.
- Viewport-priority chunked rendering happens on an idle callback.
- The Move tool changes a layer **offset**, not pixels.
- **Known scar:** an explicit FIXME notes that a projectable offset change
  re-renders the whole projection; and there is no mipmap-based canvas by default
  (it is opt-in behind a flag).

### 1.2 Krita

- **CPU tiles 64²** in a hash table with **copy-on-write** tile data.
- Each node caches its own **projection** (`KisProjectionPlane`), composited by
  `KisAsyncMerger`.
- `setDirty` propagates up the tree; a rects-walker computes
  `changeRect` / `needRect` / `accessRect`.
- `KisUpdateScheduler` runs two queues (updates vs stroke jobs) on worker
  threads.
- **Move = node offset** (`supportsLodMoves`); a non-destructive **Transform
  Mask** is applied at composite time.
- The GPU canvas uses **256² textures with a `1 << levels` border, per-tile
  mipmaps, PBO uploads, and no host readback**.
- Instant Preview is LOD stroke clones.
- **Known scars:** freezes at the tile RAM+swap ceiling; hidden layers still
  recompute; transform-mask commit region drift.

### 1.3 Photoshop

- The GPU (Mercury) is a **display/effects layer** (OpenGL + OpenCL) over a
  **CPU tiled working set plus scratch disk** — not a fully GPU-resident
  document.
- Adobe patent **US20150062182A1**, "Tile-based caching for rendering complex
  artwork": progressively rendered scale-level tiles; the best cached tile is
  drawn immediately and new tiles render in the background.

### 1.4 Chromium `cc`

- Sparse per-scale **256² tiles**, a three-tree (main / pending / active)
  pipeline, and viewport-priority raster.
- An explicit **damage taxonomy** in `DamageTracker`: paint invalidation vs
  raster invalidation vs draw/expose damage.
- Missing tiles **checkerboard** rather than stall the frame.

### 1.5 Graphite (Rust + wgpu)

- 256² GPU tiles, ~512 MiB LRU, viewport-driven cache. The closest reference
  implementation to what this project would build: same language, same GPU API.

### 1.6 Anti-patterns to avoid

- **VRAM eviction on the main thread** — synchronous eviction blocks the frame.
- **Tile seams** — filtering and feathering need a border/gutter.
- **Unbounded tile cache** — Paint.NET crashed from tile-cache blow-up when
  zoomed out.
- **`device.poll(Wait)` per frame** — stalls the GPU pipeline.

---

## 2. Qt reality (decisive for this project)

### 2.1 Measured phase cost (corrects the readback premise)

A full 4000×4000 GPU composite (`composite_gpu_region`, 2 RGB pixel layers,
RTX 3090) costs ~254 ms, measured by phase:

| Phase | Time | Share | What it is |
|---|---:|---:|---|
| `build_source` | 146 ms | 58% | CPU planar source assembly + upload of both layer planes |
| `to_pixel_buffer` | 35 ms | 14% | CPU de-interleave of the readback into the planar `PixelBuffer` |
| `mapped.to_vec()` | 22 ms | 9% | 64 MB copy of the mapped readback |
| `zero_canvas` | 18 ms | 7% | |
| `build_mask` | 16 ms | 6% | |
| readback submit + `poll(wait)` + `map` | 7 ms | 3% | the actual 64 MB transfer |
| GPU compute dispatch (submit is async) | 0.2 ms | ~0% | |
| allocations | ~0.01 ms | ~0% | |

The composite is dominated by **host-side CPU per-pixel work**: `build_source` +
`to_pixel_buffer` + `mapped.to_vec()` is ~203 ms (80%), the GPU dispatch is
~0.2 ms, and the 64 MB readback is ~7 ms. `buffer_to_image` alone is ~40 ms.
The readback is real but small; the CPU assembly is the cost. That reorders the
plan: a zero-copy GPU-resident present removes only ~57 ms of ~254 ms while the
146 ms source assembly stays, so the present work is deferred (§3) and the next
change targets the interactive paths and present caching instead.

### 2.2 Qt reality

- `QRhiWidget` **cannot adopt our wgpu device**: it owns its `QRhi` internally
  and there is no `setRhi`/`adoptDevice` hook; one backend per window
  (`crates/pictura-app/GPU-INTEROP-NOTES.md`).
- Zero-readback present therefore needs **Qt Quick**:
  - `QQuickRhiItem` shares the window's `QRhi`;
  - `QQuickWindow::createTextureFromRhiTexture()` (Qt 6.6) wraps our canvas
    texture as a scene-graph texture;
  - or share one Vulkan device via
    `QQuickGraphicsDevice::fromDeviceObjects(...)`.
- `QSGSimpleTextureNode` is software-backend only and is not a path here.

The existing M0.5 finding (same-device, same-image sharing works; presentation
is the gap) is consistent with this: the transfer mechanism is proven, the host
widget is the blocker.

---

## 3. Plan: M31–M35

### M31 — region (dirty-rect) compositing (shipped)

Composite only the changed document rect into a cached full-document canvas and
read back only that rect. A moved layer's damage is `old_bounds ∪ new_bounds`; a
paint dab's damage is its bounding box; a global op (mode switch, document size)
still recomposites fully.

Because compositing is per-pixel and order-independent across disjoint rects, a
sub-rect composite is **byte-identical** to the corresponding sub-rect of the
full composite, so the dirty-rect path is checked directly against the existing
CPU oracle. M31 removed the full composite and the full readback from Move and
paint and added `composite_region_active`; it stays on the QWidget/QImage canvas
(no new host widget, no Qt Quick migration).

### M32 — interactive-path region completion and present caching (this change)

Complete the region path for the remaining interactive mutations and cache the
presented image. The measured phase table (§2.1) is why this, not the zero-copy
present, is the next step.

- **Move-preview base** (`begin_move_preview`) region-composites only the moved
  layer's document rectangle with that layer hidden into the cached canvas,
  byte-identical to the old full recomposite with the layer hidden.
- **Visibility toggle** (`set_layer_visible`) refreshes only the toggled layer's
  provably bounded rectangle, byte-identical to a full recomposite; unboundable
  toggles fall back to a full recomposite.
- **Present zoom cache:** `ImageView` caches the document scaled at the current
  zoom, so a pan/hover repaint no longer re-scales the full-resolution document.

OpenSpec change `m32-interactive-canvas`.

Deferred from M32 to a later change: cheap undo/redo + composite coherence
(persisting the rendered composite into `doc.composite` changes what `write_psd`
serializes, so it needs its own proposal), the C++ region blit
(`ImageView::blitRegion`) and the `REGION_REFRESH_BUDGET` removal, and
zoom-level details beyond the single cached scaled image.

### M33 — full-composite throughput: row-wise assembly, fused planar readback, GPU-side clear (proposed)

Target the measured 146 + 35 + 22 + 18 + 16 ms of host-side per-pixel work in
the full composite. OpenSpec change `m33-composite-throughput` (MODIFIED
`gpu-compositing`; no new capability):

- row-wise/`copy_from_slice` source assembly in `build_source`, with the
  per-pixel loop retained as the fallback when a channel plane does not cover the
  row intersection;
- row-wise coverage fill in `build_mask` when the layer has no enabled
  data-carrying mask, with the per-pixel `mask_alpha` path otherwise;
- a fused planar readback that de-interleaves directly from the mapped readback
  slice into the planar `PixelBuffer`, skipping the packed `Vec` and its
  `to_vec()` copy;
- a GPU command-buffer clear for the canvas instead of uploading a full-canvas
  host zero buffer;
- parity plus an `#[ignore]` phase-timing profile.

All five preserve output byte-for-byte (0 LSB). Deferred: resident per-layer GPU
source buffers across a composite session (needs content versioning) and
shader-side planar output (removes the de-interleave entirely).

### M34 — GPU-resident zero-copy present via Qt Quick (deferred)

Keep the composite in a persistent GPU texture and present it directly. Two
routes, both Qt Quick:

- `QQuickRhiItem` + `QQuickWindow::createTextureFromRhiTexture()`, wrapping our
  canvas texture; or
- one shared Vulkan device via `QQuickGraphicsDevice::fromDeviceObjects(...)`.

`QRhiWidget` cannot adopt our wgpu device, and `QSGSimpleTextureNode` is
software-backend only. This removes the readback and the `QImage` conversion from
the interactive path. It is deferred because the readback is measured at ~7 ms
(plus the ~57 ms CPU de-interleave/copy round trip) of a ~254 ms composite: at
most ~57 ms of the interactive path, while the 146 ms source assembly stays until
M33. Revisit only after M33 and a present-bound profile.

### M35 — 256² GPU tiles + LRU + seam gutters + mipmaps (Graphite-style, deferred)

Only if pan/zoom over documents larger than VRAM demands it. Sparse 256² tiles
with an LRU budget, per-tile mipmaps, a border/gutter to kill seams, and
display-time LoD so a zoomed-out view composites a proxy level instead of the
full-resolution document. Deliberately last: the most code and the least certain
payoff for this project's document sizes.

### Non-goals for these milestones

- A mipmap pyramid by default (GIMP opted out; Krita's GPU path opted in). LoD is
  an M35 concern.
- Swap-to-disk tile backends.
- A fully GPU-resident document (Photoshop does not do this either).

---

## 4. Sources

- GEGL features — `https://gegl.org/features.html`
- GEGL environment / cache budget — `https://gegl.org/environment.html`
- GIMP projection (mipmap pyramid, dirty regions) —
  `https://raw.githubusercontent.com/GNOME/gimp/master/app/core/gimpprojection.c`
- Krita tiled data manager (64² tiles, COW) —
  `https://raw.githubusercontent.com/Krita/krita/master/libs/image/tiles3/kis_tiled_data_manager.h`
- Krita transformation masks (composite-time transform) —
  `https://docs.krita.org/en/reference_manual/layers_and_masks/transformation_masks.html`
- Krita OpenGL rendering (256² textures, border, mipmaps, PBO, no readback) —
  `https://deepwiki.com/KDE/krita/3.2-opengl-rendering`
- Adobe patent US20150062182A1, tile-based caching for complex artwork —
  `https://patents.google.com/patent/US20150062182A1/en`
- Adobe hardware-performance white paper (Mercury / OpenCL) —
  `https://www.nvidia.com/content/adobe/pdf/adobe-hardware-performance-white-paper.pdf`
- Chromium `cc` overview (256² tiles, three-tree, viewport-priority) —
  `https://chromium.googlesource.com/chromium/src.git/+/HEAD/docs/how_cc_works.md`
- Chromium `DamageTracker` (damage taxonomy) —
  `https://chromium.googlesource.com/chromium/src/+/f830f3d4ae0191c9095949096e9c3ca1f6dc8d12/cc/damage_tracker.h`
- wgpu `Queue` (write/submit, readback cost model) —
  `https://docs.rs/wgpu/latest/wgpu/struct.Queue.html`
- Qt `QRhiWidget` (owns its QRhi; no device adoption) —
  `https://doc.qt.io/qt-6/qrhiwidget.html`
- Qt `QQuickRhiItem` (shares the window's QRhi) —
  `https://doc.qt.io/qt-6/qquickrhiitem.html`
- Qt `QQuickWindow` (`createTextureFromRhiTexture`, 6.6) —
  `https://doc.qt.io/qt-6/qquickwindow.html`
- Qt `QQuickGraphicsDevice` (`fromDeviceObjects`, shared Vulkan device) —
  `https://doc.qt.io/qt-6/qquickgraphicsdevice.html`
- Graphite tile-based caching (256² GPU tiles, ~512 MiB LRU, viewport-driven) —
  `https://deepwiki.com/GraphiteEditor/Graphite/4.3-tile-based-caching`

Repository inputs read: `crates/pictura-app/src/cxxqt_object.rs`
(`document_to_image`, `current_buffer`, `buffer_to_image`),
`crates/pictura-app/GPU-INTEROP-NOTES.md`, `docs/dev/m29-large-doc-performance.md`,
`docs/01-architecture/gpu-rendering-pipeline.md`.
