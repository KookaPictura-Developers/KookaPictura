# Design

## Context

See proposal.md — Why. The frame-bounded present (`queue_present` /
`take_pending_present`) already coalesces the *regions* shown per frame; the
remaining per-dab work is the CPU view-pyramid walk and the commit's
bounding-box composite.

## Goals / Non-Goals

**Goals:**

- Charge the view-pyramid rebuild to the frame, not the dab.
- Keep every committed pixel byte-identical (CPU is the oracle).
- Decompose a stroke's dirty area so a sparse stroke composites its tiles.

**Non-Goals:**

- Changing the compositor, the region-blit protocol, or the GPU dab shader.
- Exact visible parity of the *mid-stroke* canvas; it stays a frame-bounded
  approximation, exact at commit.

## Decisions

**D1 — Present frame-bounded; fold the pyramid once per present.** The dab that
opens a frame presents immediately and the frame's later dabs accumulate in
`pending_present`, shown at `flush_present`. Each present goes through
`refresh_region`, which composites the presented region from the active working
document and folds it into the level-0 frame and the view pyramid, so
`canvas_revision` advances once per present, not once per dab. (This replaced
the per-dab level-0 patch plus deferred pyramid from the original sketch;
`gpu-stroke-rendering` presents each frame from the exact working document.)

**D2 — The canvas crops level 0 while a GPU stroke presents.** `preview_present_level`
returns `-1` for a live GPU stroke. `ImageView::presentCrop` maps `-1` to
level 0, whose region blits keep it current. This reuses the existing provider
callback, so no new invokable and no growth of the ceiling-bound
`cxxqt_object.rs`; the alternative — cropping a stored level — would need the
present to keep every level current.

**D3 — Tile-align the pyramid update.** `ViewPyramid::update` rounds the level-0
dirty rect out to `TILE = 64` before `expand`ing per level. The alignment is
transparent to correctness: pixels recomputed outside the precise dirty rect are
re-derived from the unchanged level-0 buffer and equal a rebuild. Sharing the
grid with the commit's tile decomposition keeps one granule across the change.

**D4 — The stroke tracks a document-space tile bitmap.** `TileSet` marks the
tiles of every changed dab rectangle (from `take_dirty` on the exact path and
from each GPU dab's returned rectangle). `end_paint` turns it into disjoint
rectangles with a per-row RLE plus a greedy vertical merge, then composites each
through the existing `refresh_region`. This is the smallest change that stops a
diagonal from paying its bounding box: it adds no paint-engine work and leaves
`Stroke` untouched.

**D5 — Collapse when decomposition does not pay.** More than `REGION_CAP = 64`
rectangles, or rectangles that fill at least three quarters of their own
bounding box, collapse back to that bounding box. A filled blob is one rect; a
diagonal stays many small ones.

**D6 — The collapsed commit blits its own region buffer.** When the dirty tiles
collapse to one rectangle, `PictureViewRust::refresh_regions` builds the single
emitted region image from that rectangle's sRGB buffer
(`premultiplied_display_image`) instead of `display_crop(0, union)`, dropping a
second full-union crop from the dense-stroke commit. The two are byte-identical
because both premultiply the same pixels (`round(c * a / 255)`), so no committed
or displayed pixel changes; several rectangles keep the union crop, because the
gaps between them carry level-0 pixels the per-rect buffers do not hold. The
one-region-blit invariant is unchanged: the shell still refreshes the command
registry once per commit, and the `commit_refresh_region` timing plus its
`rr_to_qimage` / `rr_blit_emit(C++)` sub-steps now cover the commit image.

## Risks / Trade-offs

- [A GPU stroke crops level 0 at reduced zoom] → D2; level 0 is the region
  blit's source and is kept current, and the commit rebuilds the zoom-selected
  level.
- [Tile alignment enlarges a small region refresh] → The extra work is bounded
  by one 64×64 tile per level and is dwarfed by the per-dab walk it replaces.
- [Per-rect commit emits several `regionBlitted` signals] → The canvas blit is
  idempotent and the rects are disjoint; the commit already emitted one region
  per stroke.
- [A commit rect can extend past the document] → `refresh_region` clamps it,
  exactly as it clamps the union today.
- [The region-buffer blit could diverge from the level-0 crop] → Both
  premultiply the same region pixels; the equivalence is asserted pixel for
  pixel by `a_single_region_commit_blit_equals_the_level0_crop`.

## Open Questions

None that change the spec or the task breakdown.
