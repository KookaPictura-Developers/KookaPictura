# Design

## Context

See `proposal.md` - Why. The relevant current state, from the investigation:

- The canvas already presents from a view pyramid and has a described-damage
  account (`pictura-render::CanvasDamage`), a regional refresh
  (`PictureView::refresh_region`), and a C++ region blit. Paint also has an
  incremental regional path: `paint_dab` → `Stroke::take_dirty` →
  `refresh_region` patches the sRGB level-0 frame and repairs the pyramid every
  dab.
- `ImageView::presentCrop` returns no crop while `levelProvider_.isPainting()`
  is true (`cpp/image_view.cpp`), so mid-stroke `paintEvent` resamples the
  full-resolution `image_` even though the pyramid is already current.
- The app keeps two full-resolution display frames: `PictureViewRust::level0`
  (sRGB, straight alpha) and `PictureViewRust::image` (premultiplied `QImage`),
  plus the document's planar `composite`, plus the layer channels and the
  history snapshots.
- `PictureView::image()` rebuilds the full image whenever the damage account is
  non-default and clears it; `HistogramPanel::recompute` calls `view_->image()`
  on the 120 ms panel cycle, so a region refresh is undone by a full rebuild.
- 79 `.recomposite()` sites plus 3 direct `reset_pyramid()` sites; only paint
  dabs, visibility toggles, move-preview, move commit, and the healing family
  use the region path today. Regional composition
  (`composite_region_active`) is available and byte-equivalence is already
  covered by `tests/gpu_parity/region.rs` and `canvas_view_tests`.
- `pictura_codec::buffer_to_srgb` reparses the document profile and rebuilds an
  lcms transform on every call, and clones the buffer when there is no profile.
- `decode_image_rgba` copies the decoded image one byte at a time across the
  cxx bridge.

## Goals / Non-Goals

**Goals:**
- A paint stroke presents from the pyramid on every dab on a 16000²-class
  document.
- The display path holds one less full-resolution frame (`level0`); level-0
  crops come from the document composite.
- Ordinary bounded operations stop recompositing the whole document.
- No panel refresh forces a full-resolution rebuild.

**Non-Goals:**
- Tiled or copy-on-write history (separate change).
- Moving decode/open off the GUI thread.
- GPU layer residency; the GPU compositor keeps re-uploading as today.
- Making region composition correct for object-based effects, knockout, or
  pass-through groups; those keep the full recomposite.

## Decisions

### 1. Present mid-stroke from the pyramid, not from `image_`

Remove the `isPainting()` early return in `presentCrop` so the canvas presents
the zoom level crop during a stroke. Validate the premise the guard protected:
the pyramid is patched every dab by `refresh_region` → `refresh_level0_region`
+ `update_pyramid`, so it reflects the stroke's working document. `level0_buffer`
and `reset_pyramid` read `source.composite`, which is stale mid-stroke
(`Stroke::composite_pixel` writes the layer channels, not `working.composite`),
so the design MUST keep `reset_pyramid` off the mid-stroke path. `refresh_region`
already avoids it; the task is to audit the mid-stroke-reachable callers.

Alternative considered: keep the guard and patch `image_` from the region blit
(the status quo). Rejected: it still resamples full resolution and keeps the
full-res `image_` as a present source.

### 2. Drop the separate cached sRGB `level0` frame

Build the pyramid and serve level-0 crops from the document's planar `composite`
through the working-profile conversion, instead of caching a second
full-resolution sRGB frame in `PictureViewRust::level0`. `refresh_level0_region`
becomes a region conversion of `doc.composite` (or of the stroke's working
document mid-stroke) rather than a patch of a stored frame. This removes one
full-resolution buffer and the `into_rgba_frame`/`buffer_to_srgb` copy on the
full path is replaced by a region-sized conversion.

Alternative considered: keep `level0` and drop `image`. Rejected for this
change: `image()`'s cached-when-clean semantics are specified and used by the
self-tests; changing that is its own behavior change. Dropping `level0` is the
smaller, non-breaking half.

### 3. Regional operations by bounding the effect, with a conservative fallback

For each candidate operation, compute the dirty rectangle from the operation's
own parameters (layer/mask rect, pasted rect, stroke changed rect, move/transform
source ∪ destination) and call the existing `refresh_region`. Candidate set with
bounded rectangles: layer opacity/blend/fill-opacity on a raster layer, filter
and adjustment on a pixel layer, clipboard clear/paste, paint commit/cancel,
move and transform commit. Keep the full recomposite for document size changes,
flatten/merge/reorder, profile conversion, GPU toggle, unmasked adjustments,
groups/pass-through/knockout, and any layer with an effect block — the region
compositor falls back to a full composite for effects anyway, so claiming a win
there would be wrong.

Alternative considered: migrate every `recomposite()` site mechanically.
Rejected: several are not bounded (structure, groups, effects) and a wrong bound
is a correctness bug, not a perf bug.

### 4. Selection edits stop recompositing

Selection operations change no pixel the compositor reads; remove their
`recomposite` calls and their damage marks, and refresh only the overlays. This
is a behavior change that the new `compositing/described-damage` spec pins.

### 5. Histogram reads a pyramid level

`HistogramPanel::recompute` reads the coarsest pyramid level (≤512 px, as the
navigator already does) instead of `view_->image()`. This removes the full scan,
the full rebuild, and the damage drain in one move. The panel keeps its 256-bin
output; only the source changes.

### 6. Cache the working-profile transform

Keep the parsed `Profile` and the built `Transform` on the display side, keyed on
`document_icc` bytes, and invalidate on open, assign, and convert. Return the
buffer unchanged (no clone) when `document_icc` is `None`. The converted bytes
must stay identical to a fresh conversion.

### 7. Bulk decode copy

Replace the per-byte `push_back` loop in `decode_image_rgba` with a per-row or
whole-buffer `extend_from_slice` (or `reserve` + bulk append), and check the
budget against the probe's declared size before constructing engine structures.

## Risks / Trade-offs

- [Removing the mid-stroke present guard exposes a stale-pyramid path if any
  mid-stroke edit reaches `reset_pyramid`] → Audit and, if needed, make
  `level0_buffer`/`reset_pyramid` composite the stroke's working document so a
  stray rebuild is correct, not just avoided; add a mid-stroke crop-vs-patched
  equality test.
- [Dropping the stored `level0` re-converts pixels on every level-0 crop] →
  Level-0 crops are only requested at zoom > 0.5 and are bounded to the
  viewport; the working-profile transform is cached, so the per-crop cost is a
  region-sized conversion, not a profile parse.
- [A region bound that is wrong for an exotic stack silently diverges from a
  full composite] → Only migrate operations with a provable bound; treat
  effects, groups, pass-through, and knockout as unbounded; every migrated
  operation gets a region-vs-full equivalence test.
- [Removing selection recomposites changes observable behavior] → Pinned by
  scenarios in `compositing/described-damage`; verify no render path reads the
  selection before treating it as free.
- [`refresh_region` bumps `canvas_revision` but not `content_revision`] →
  Migrating preview paths could leave the move-preview cache stale; keep
  previews off the migrated set or bump the revision they rely on.
- [Dropping `level0` removes the buffer that `refresh_level0_region` patched] →
  Region refresh must convert the region from `doc.composite` (or the stroke's
  working document) each time; the equivalence tests already compare the
  updated level to a rebuild.

## Migration Plan

No data migration. The change is internal to the app and codec; the specs'
existing canvas-equality and region-parity tests are the rollback guard. Land in
phases (see `tasks.md`): cheap present/panel/ICC/decode wins first, then the
mid-stroke present, then the operation migrations, then the buffer reduction.
Each phase must keep `verify-fast.sh` green on its own.

## Open Questions

- Whether `image()`'s cached-when-clean semantics can be dropped to remove the
  last full-resolution `QImage` is deferred; the current change only removes the
  sRGB `level0` frame.
