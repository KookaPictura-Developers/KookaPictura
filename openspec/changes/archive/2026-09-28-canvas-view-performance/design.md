# Design

## Context

See `proposal.md` for motivation. Current state that shapes the approach:

- The document's authoritative pixels live Rust-side (`doc.composite`, planar
  straight-alpha) and in the view pyramid's future owner, `PictureViewRust`.
  C++ `ImageView` holds a derived full-resolution `QImage` plus a single scaled
  present cache (`kMaxCachePixels = 64 MP`, disabled above it) and falls back to
  `painter.drawImage(image_)` — the full-resolution source — past that bound.
- Partial repaint already exists: `refresh_region` composites a rectangle and
  emits `regionBlitted(QImage, x, y)` without `changed`, while
  `recomposite`/`image()` handle full updates. `display_dirty` marks the derived
  `rust.image` stale.
- The bridge marshals only `QImage`, `QString`, `QStringList`, and primitives;
  rectangles are conventionally encoded as `QString` (`"x y w h"`) and
  coordinates as `i32`.
- The GPU compositor uses storage buffers, bounded by
  `max_storage_buffer_binding_size`/`max_buffer_size`; a 16000² document is
  ~1 GB per buffer and may exceed those limits, so the CPU fallback is
  load-bearing at the target size.
- `docs/dev/canvas-compositing-plan.md` sketched display LoD under the M38 label,
  but M38 as recorded is GPU 256² tiles plus proxy-level *compositing*, and
  `STATE.md` records that the M36–M38 numbers were reclaimed by the panels
  program. This change adds a CPU display pyramid (cache the full composite, crop
  a level) and explicitly does not add proxy-level compositing; the dev note is
  updated to name this work rather than claim M38.
- C++ self-test checks pin region-refresh and present behavior: **75** (large
  region move emits `regionBlitted`, no `changed`), **76** (present cache reused
  across repaints), **82** (region path), **83** (blit equality), **84** (present
  cache patched, not rebuilt), **85** (large region), **344** (per-dab region
  size), **345** (patched cache equals the direct draw), **363** (visibility
  region toggle). Exit codes are append-only and are the failure identity
  (AGENTS), so replacing the present cache requires migrating 76, 84, and 345
  rather than dropping them.
- Shared straight-alpha helpers have non-display consumers: `buffer_to_rgba_bytes`
  feeds the export encoder (`export.rs:72`, `impl_core.rs:326` →
  `encode_image_rgba`, which builds `Format_RGBA8888`), and `rgba_image` feeds
  layer/mask/histogram thumbnails. Their bytes must not change.

## Goals / Non-Goals

**Goals:**

- Present any document, including 16000², at any zoom, from a viewport crop of a
  level, without rescaling the full-resolution document.
- Keep partial repaint provably correct with an explicit damage account.
- Feed the navigator from the same pyramid; the navigator adds no second
  full-resolution copy.
- Make the CPU composite fallback row-parallel without changing a byte of
  output.

**Non-Goals:**

- GPU-resident zero-copy present, GPU tile caches, resident per-layer GPU source
  buffers, and history copy-on-write (separate deferred tracks).
- Level-of-detail *compositing* (compositing a proxy of each layer). The pyramid
  caches the full composite; layer compositing still runs at document resolution
  for the damaged region.
- Any change to blend-mode math, composite semantics, or the on-disk PSD format.
- Migrating the histogram/control-server/self-test consumers off the
  full-resolution `image()`; they keep it as a compatibility path.

## Decisions

### D1 — The pyramid is owned in Rust, beside the document

`ViewPyramid` is a `pictura-render` type; `PictureViewRust` owns it and updates it
in the same paths as `doc.composite`. Rationale: every producer (`recomposite`,
`refresh_region`, undo/redo, move-preview) is Rust-side, and the C++ copy is
derived. Alternatives: a C++ pyramid over `QImage` — rejected because it
duplicates invalidation across the FFI, misses the Rust-side display conversion,
and re-does the halving in Qt.

### D2 — The pyramid stores no level 0; the app caches one sRGB level-0 frame

`ViewPyramid` never stores level 0: each `rebuild`/`update`/`crop` call is supplied
a borrowed planar straight-alpha level-0 view, and levels 1..n are stored
premultiplied (≈⅓ of the document). A crop of any level is premultiplied on the
way out. The shipped app supplies that borrowed view from `level0`, a cached
full-resolution **sRGB** frame (`level0_buffer` =
`into_rgba_frame(buffer_to_srgb(doc, &doc.composite))`), which is also the source
of the premultiplied display image; so the common RGBA path does add one
full-resolution buffer over `doc.composite`, namely the sRGB level-0 frame, not a
stored pyramid level. Rationale: it keeps the color-managed display source in one
place and avoids a *stored premultiplied* level 0, which at 16000² is another ~1 GB
(measured: 2687 MiB vs 1711 MiB peak, see below). `docs/02-ui-ux/panels/navigator-panel.md`
forbids the Navigator adding a second full-resolution copy, and the pyramid itself
stores none — neither claim means the display path holds no extra full-resolution
frame. Trade-off: a level-0 crop pays one premultiply pass; that is the same
conversion Qt would otherwise force per frame, done once per crop.
Whole-level planar buffers with a photorust-style rect update are the first
implementation; tiles, an LRU, and an epoch guard are deferred until a profile or
a stale-frame bug demands them. Measured 16000² peak: one full-resolution level-0
frame + halved levels ≈ 2.0 GB, storing a premultiplied level 0 as well ≈ 3.0 GB,
on a box with 64 GB RAM. The `mem_probe_16k` figures cover the document and the
pyramid buffers; they do not include the display `QImage` (`rust.image` /
`ImageView::image_`).

### D3 — Level shrinking is CPU; the compositor stays GPU-first

Levels below 0 are produced on the CPU from whatever backend produced the
damaged region, because Qt is ultimately handed CPU pixels. The GPU remains the
compositor. Rationale: avoids a readback per level and keeps determinism (rule 6
— golden comparisons run on CPU). This matches photorust's own call.

### D4 — Additive bridge; the old surface is retained

`changed`, `regionBlitted`/`region_blitted`, and the `image()` entry point stay,
but `image()` now returns a premultiplied (`Format_RGBA8888_Premultiplied`) frame
built from the sRGB level-0 frame, not the old straight-alpha bytes. That is a
no-op for QPainter consumers (premultiplied is QPainter's native format); the
export encoder and thumbnail helpers do not read `image()` (they use the
straight-alpha helpers in D9), so their behavior is unchanged. Add
`display_image(level, x, y, w, h) -> QImage`,
`display_level_count() -> i32`, `display_level_size(level) -> QString`,
`take_canvas_damage() -> QString`. `ImageView` KEEPS its full-resolution `image_`
as a compatibility buffer built on demand for `image()`'s consumers (Navigator
today, Histogram, control-server `document` scope, `main.cpp`, and ~40 self-test
probes); the repaint path stops using it. Migrating those consumers to pyramid
levels is deferred with a named ceiling. Present migrates to level crops while the
old signals stay. Alternatives: delete `image_` outright — rejected as it silently
breaks a large consumer set and the self-tests.

### D5 — Bridge marshalling follows the existing convention

Rectangles as `QString` (`"x y w h"`, empty = none), levels as `i32`, and a
level's size as `QString` (`"w h"`), matching the existing
`layer_rect`/`selection_bounds` style. No `QRect`/`QSize` cxx-qt types are
introduced.

### D6 — One landing, no separate stage-A crop

The display conversion, the zoom-dependent filter, and `CanvasDamage` landed
together with `ViewPyramid` and level selection; there is no separate stage-A
crop-from-source at zoom ≥ 100 %. The level-0 crop *is* that present, so the
byte-identity check for the level-0 crop against a full-resolution transform draw
covers what a stage-A crop would have. The 16000² profile is acceptance evidence
for the pyramid; the requirement (any zoom, 16000²) already settles that it is
needed.

### D7 — Level selection

Choose the coarsest level whose scale is still at least as fine as the screen
(`1/2^L >= zoom`): level 0 for zoom > 0.5, level 1 for 0.25 < zoom ≤ 0.5, level 2
for 0.125 < zoom ≤ 0.25, and so on. This guarantees no visible detail is
discarded and picks the smallest source that can fill the viewport. The spec's
prose matches this boundary (coarser only at or below 50 %). The formula is
mirrored in C++ `ImageView::presentLevelForZoom` and in the Rust profile test
(`tests_profiles.rs::present_level`) and must stay in sync across the two.

### D8 — Damage update reads a borrowed level-0 view

`update(level0: Planes<'_>, dirty: PsdRect)` takes the caller's current
straight-alpha level-0 view (the app has already applied the change to its own
composite, or supplies the stroke's document mid-stroke) and recomputes the
affected region of every level below 0, premultiplying as it shrinks. Each level
lands exactly on a full rebuild. This is directly testable (the ported photorust
test compares every level against a rebuild). `ViewPyramid` carries no lifetime
parameter: the borrow lives on the call, never in the struct.

### D9 — Premultiply only in the display conversion

The display conversion gets its own function; `buffer_to_rgba_bytes`,
`buffer_to_image`, and `rgba_image` keep emitting straight alpha for the export
encoder and the thumbnails. `image()` now also returns premultiplied pixels (it is
built from the sRGB level-0 frame through the display conversion); QPainter
consumes that format natively and the export/thumbnail helpers stay straight, so
the change is visible only to consumers that read `image()` pixels directly.
Rationale: `encode_image_rgba` and the thumbnail
sinks assume straight alpha; premultiplying the shared helpers would silently
corrupt Save-As/Export/Quick Export and thumbnail colours. A regression test
pins the export and thumbnail bytes.

### D10 — Rayon in a new module, row-variant blend functions

`composite.rs` is 1167/1200 LOC, so the row-splitting helpers go in a new
`composite_rows.rs`. `blend_into`/`blend_parts` keep their `&mut Canvas`
signatures (≈17 references); row-scoped variants delegate to them and are
reachable from `composite_native.rs` as well as `composite.rs`. Only per-pixel
content loops (`composite_pixels`, `composite_canvas`, the 8-bit adjustment loop)
and `gate_adjusted` (defined in `composite_native.rs:190`) parallelize; effect
kernels and the knockout `cover` canvas stay serial. `rayon` is a new direct
dependency, justified by the CPU fallback at 16000².

### D11 — Identity holds only at the same filter and at level 0

"Cached present matches the transform draw" means level 0 at zoom ≥ 1, both drawn
with the same sampling filter. It cannot mean byte-identity against the pre-change
nearest-filtered draw: smooth filtering below 200 % is an intended change. A
coarser level at a non-power-of-two zoom is a single reduction, compared against
a reference reduction, not against a full-resolution transform draw.

## Risks / Trade-offs

- **16000² memory** → the pyramid stores no level 0; the app caches one
  full-resolution sRGB `level0` frame for the pyramid and display, and only the
  halved levels are added on top. `ImageView::image_` is retained but no longer
  used by the repaint path. `image()` for panels is reviewed as a follow-up if it
  shows on the interactive path.
- **`changed == 0` invariants** → keep `region_blitted`/`changed` and the
  no-`changed`-on-region rule; migrate consumers before deleting anything.
- **Present-cache checks 76, 84, 345** → they assert the old cache's reuse/patch
  behavior; the tasks migrate them to assert level-crop reuse and byte identity
  instead of dropping them (codes are append-only).
- **Mid-stroke source** (`doc.composite` deliberately stale while painting) → the
  pyramid/damage is invalidated per dab from `stroke.document()`, never read from
  a stale `doc.composite`.
- **History/move-preview bypass** (`rust.image` set directly) → both must reset
  or rebuild the pyramid and the damage account.
- **Export/thumbnail corruption** → the display conversion is separate from the
  shared straight-alpha helpers (D9); a test pins export and thumbnail bytes.
- **Level-shrink rounding / premultiply** → port photorust's tests: odd-edge
  averaging and the soft-edge premultiplied test.
- **GPU `TooLarge` at 16000²** → CPU fallback is primary at that size, which is
  why row-parallelism (D10) is in scope rather than deferred.
- **`composite.rs` headroom** → new module for the row helpers.
- **Rayon nondeterminism** → rows are independent (no cross-row reads, dissolve
  is a pure `(x,y)` hash, `blend_if` reads the same pixel); a
  `parallel == sequential` test plus the existing region-equality tests pin it.

## Migration Plan

- One branch/PR, landed in one piece; each landed step runs
  `cargo nextest run --workspace`, `cargo test --workspace --doc`, and the
  headless self-test.
- No on-disk format change; rollback is reverting the branch.
- Kill switch is unnecessary: the crop path is always on; `PresentCache` is
  replaced only after checks 76, 84, and 345 are migrated to level-crop checks.
- Timing evidence: `#[ignore]`d print-only `scroll_zoom_pan_profile_4000` and
  `_16000`; the C++ self-test gains behavioral checks (crop reuse across pan,
  crop equals full per level, region path emits no `changed`), with any
  wall-clock only under `PICTURA_REFERENCE_RUN`.

## Open Questions

- Whether `image()` (full-resolution, used by histogram/control/self-test) should
  later be served from a pyramid level for 16000²; deferred, since it is not on
  the repaint path.
- Whether the navigator and histogram should share one level budget/LRU for huge
  documents; the initial change reuses whichever level fits.
