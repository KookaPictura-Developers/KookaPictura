# Canvas Compositing & Large-Layer Move Plan

- **Status:** dev note (research synthesis + M31–M38 roadmap; not an OpenSpec
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

## 3. Plan: M31–M38

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

Deferred from M32 to a later change: cheap undo/redo + composite coherence (which
became **M34**, below) and the C++ region blit (`ImageView::blitRegion`) with the
`REGION_REFRESH_BUDGET` removal (now staged as **M35**, below). Zoom-level details
beyond the single cached scaled image stay deferred.

**Re-scoped (M34):** composite coherence + cheap undo/redo is carved out as the
next change, **M34 — composite coherence and cheap undo/redo**
(`openspec/changes/m34-composite-coherence`, brief
`docs/dev/m34-composite-coherence.md`). It is app-local and bounded — persist the
rendered composite into `doc.composite` preserving its colour-plane count, make
Save serialize that current composite, and have `undo`/`redo` restore from the
snapshot composite instead of recompositing — with **no `write_psd` format
change**. The canvas-throughput tracks below stay deferred: resident per-layer GPU
source buffers, GPU-resident zero-copy present, 256² tiles + LoD, and history
copy-on-write / tile diffs each still need their own design, and the perf series
is paused while M34 lands.

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

All five preserve output byte-for-byte (0 LSB). **A2 — shader-side planar
output (shipped).** A follow-up to M33 removes the host de-interleave entirely:
`PLANAR_SHADER` (`cs_planar`) de-interleaves the packed RGBA canvas into four
byte planes in one storage buffer, and `read_canvas` copies each plane's `n`
bytes into the planar `PixelBuffer`; `to_pixel_buffer` remains only for the
profile micro-benchmark. Byte-identical (`gpu_parity` green); measured 4000²
(2 RGB layers, release, RTX 3090) `readback` ~41 ms → ~27–32 ms, total ~130 ms →
~117–131 ms. Deferred: resident per-layer GPU source buffers across a composite
session (needs content versioning).

### GPU-resident zero-copy present via Qt Quick (deferred; was planned as M34)

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

### M35 — region blit in C++ (proposed)

The interactive cost that remains after M31–M34 is the **per-pixel FFI blit**: at
4000² a 1024² paint dab composites in ~5.6 ms but the Rust `blit_image_region`
writes it into the cached `QImage` with one `QImage::set_pixel_color` call per
pixel at ~27.3 ms (~5×); at 10000² that is ~170 ms. `REGION_REFRESH_BUDGET =
1_000_000` px only hides the cost by dropping a large dirty region to a full
recomposite (130 ms at 4000², 878 ms at 10000²). M35 holds the authoritative
canvas as the planar `doc.composite`, has `refresh_region` emit a small
`regionBlitted(QImage, x, y)` signal instead of `changed`, and blits it in C++
(`ImageView::blitRegion`, `QPainter` + `CompositionMode_Source`), deleting the
budget and the per-pixel loop. `image()` becomes rebuild-if-dirty from
`doc.composite`, `sample_argb` reads the composite directly, and
`move_preview_base` is derived from the current composite. OpenSpec change
`m35-cpp-region-blit` (MODIFIED `document-canvas`); brief
`docs/dev/m35-cpp-region-blit.md`.

### Move-preview base cache (shipped)

`begin_move_preview` rebuilds the base (topmost pixel layer hidden) from the
authoritative composite on every mouse-press: at 4000² that measured 247 ms and
made the Move tool's drag start lag. The base does not change when the topmost
layer moves, so it is now cached alongside the layer image. The cache is valid
while `content_revision`, the topmost-layer index, and the layer's clamped
`(left, top, right, bottom)` rect are unchanged; `prepare_move_preview` warms it
and `ToolController::applyToolPolicy` calls that when Move is selected or the
canvas rebinds, so the first press is a cache hit. `translate_layer`,
`commit_move`, undo/redo, `history_jump`, and `history_restore_snapshot` are the
only paths that change document content without a plain `record`; the move
paths use `record_move`, which skips the revision bump because they only move
the topmost layer, and the history paths bump it explicitly. The fresh-compute
path is byte-identical to the region composite it replaced; at 4000² the hit
costs <1 ms against ~250 ms fresh (self-test `move_preview_cache`, exit 197).

### Described-change coverage (this change)

Every pixel-changing operation now either describes a bounded rectangle that the
region path refreshes, or explicitly falls back to a full recomposite. The
bounded set (each refreshes `refresh_region(rect)` and, when not painting, emits
`regionBlitted` instead of `changed`, so no panel read forces a full image
rebuild):

- **Paint** — `paint_dab` uses the dab's rect; `end_paint`/`cancel_paint` use the
  stroke's cumulative `StrokeOutcome::dirty`.
- **Layer opacity / blend / fill** — `mutate_layer` uses the mutated layer's
  bounded influence rect (`layer_visibility_region`). The generic
  `batch_changed` multi-layer path still recomposites fully.
- **Filter / adjustment** — `apply_filter` uses the target layer's rect;
  `add_adjustment` uses the adjustment's mask rect when it is bounded;
  `apply_pictura_raw` uses the target layer's bounded rect.
- **Clipboard** — `clear` uses the cleared layer's rect; `paste_clip` uses the
  newly pasted layer's rect.
- **Move / transform** — `translate_layer`, `move_preview`, `commit_move`, and
  `commit_transform` use `old_rect ∪ new_rect` (the source and destination
  bounds).

The unbounded set keeps the full recomposite: document size changes
(resize/crop/rotate/flip/canvas-size), flatten/merge/reorder/group, color-profile
convert, the GPU-backend toggle, an unmasked adjustment, a group, and any layer
whose influence cannot be bounded (a disabled or non-zero-default mask). A layer
carrying an object-based effect block still takes the region path but
`composite_rgba_region` internally composites in full for it.

Selection-only edits no longer run the compositor at all; they emit `changed`
only to move the selection overlay. The histograms and navigator read a
view-pyramid level rather than `PictureView::image()`.

### M36 — history copy-on-write / tile diffs (deferred)

`History::capture` clones the whole document per undoable state (~60 ms at
4000² and up to 20 states of RAM). Copy-on-write layer-channel sharing or
per-tile diffs removes both the latency and the memory. Needs its own design
(content versioning / tile identity), so it is staged after M35.

### M37 — resident per-layer GPU source buffers (deferred)

Deferred from M33. The remaining full-composite cost is the per-composite
source/mask **upload** (~128 MB + 16 MB at 4000²), not the dispatch (~0.2 ms) or
the ~27–32 ms readback; keeping a layer's source planes resident on the GPU
across a composite session would remove it, but needs content versioning to
detect a changed layer. (The shader-side planar output shipped as the M33 A2
follow-up above.)

### M38 — 256² GPU tiles + LRU + seam gutters + mipmaps (Graphite-style, deferred)

Only if pan/zoom over documents larger than VRAM demands it. Sparse 256² tiles
with an LRU budget, per-tile mipmaps, a border/gutter to kill seams, and
display-time LoD so a zoomed-out view composites a proxy level instead of the
full-resolution document. Deliberately last: the most code and the least certain
payoff for this project's document sizes.

> **The CPU display pyramid is a separate, landed track**
> (`openspec/changes/canvas-view-performance`). `pictura-render::ViewPyramid`
> caches the *full composite's* halved, premultiplied levels (it stores no level
> 0; each call borrows a planar straight-alpha level-0 view) and the canvas crops
> the level chosen for the zoom. The app caches one full-resolution sRGB `level0`
> frame as the pyramid and display source, so the common RGBA path adds one
> full-resolution buffer over `doc.composite` (the navigator adds none). It is
> display-only and does **not** composite layer proxies, so it is distinct from
> M38's GPU 256²-tile + mipmap + proxy-compositing track. At 16000² it peaks at
> ≈ **2.0 GB** (one full-resolution level-0 frame + halved levels); storing a
> premultiplied level-0 copy as well would cost ≈ **3.0 GB** (measured,
> `crates/pictura-render/examples/mem_probe_16k.rs`).

### Non-goals for these milestones

- A mipmap pyramid by default (GIMP opted out; Krita's GPU path opted in).
  GPU-resident LoD and proxy-level *compositing* are an M38 concern; the CPU
  display pyramid that crops a cached composite level is the landed
  `canvas-view-performance` track.
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
