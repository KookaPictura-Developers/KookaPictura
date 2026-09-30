# Tasks

## 1. Present and rebuild the pyramid frame-bounded (B1)

- [x] 1.1 Present the frame-opening dab through `refresh_region`, which folds the
  presented region into the level-0 frame and the view pyramid once per frame;
  the frame's later dabs accumulate in `pending_present` until `flush_present`.
  (`gpu-stroke-rendering` later replaced the per-dab level-0 patch and the
  deferred-pyramid helper this task first added, so `pending_pyramid` /
  `defer_pyramid` / `flush_pyramid` are gone; the present is still
  frame-bounded.) **Done.**
- [x] 1.2 Return `-1` from `preview_present_level` for a live GPU stroke and map
  it to level 0 in `ImageView::presentCrop`, so the canvas never crops a stale
  stored level. Verify the C++ self-test PASS log shows the GPU path
  (`gpu=1`) with `updates=2`. **Done.**

## 2. Tile-align the pyramid update (B2)

- [x] 2.1 Add `TILE = 64` and round the damage rectangle outward in
  `ViewPyramid::update` before the per-level `expand`; keep level 0 the caller's
  buffer. Verify `cargo nextest run -p pictura-render view_pyramid` passes with
  the tile-rounding test and the existing rebuild-equality tests. **Done.**
- [x] 2.2 Mark deliberate shortcut: the alignment is unconditional, bounded by
  one tile per level. **Done** (documented on `update`).

## 3. Multi-rect dirty commit (C1)

- [x] 3.1 Add a document-space `TileSet` (row RLE plus greedy vertical merge)
  with a collapse heuristic to `helpers.rs`; track the stroke's dirty tiles from
  `take_dirty` on the exact path and from the preview/GPU replay. Verify the
  diagonal-decomposition and blob-collapse unit tests pass. **Done.**
- [x] 3.2 Composite the commit's rectangles through `refresh_region` in
  `end_paint`. Verify `a_multi_rect_stroke_commit_equals_a_full_recomposite`
  passes. **Done.**

## 4. Verification

- [x] 4.1 Add C++ self-test 547 (`pp_pyramid_per_frame`) with the
  `ST_BEGIN`/`ST_PASS`/`ST_FAIL` macros; runs the GPU path when an adapter is
  present, else the frame-bounded exact path. **Done.**
- [x] 4.2 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets --
  -D warnings`, `cargo nextest run --workspace`, `TASK_ALLOWS_DOCS=1 bash
  scripts/verify-fast.sh`, the CMake build plus both self-tests, and `openspec
  validate --all --strict`. **Done.**

## 5. Commit blit from the region buffer (C2)

- [x] 5.1 Measure the commit sub-steps with `PICTURA_PAINT_TIMING=1` on 4000²:
  `rr_composite_region`, `rr_buffer_to_srgb`, `rr_patch_composite`,
  `rr_level0_patch`, `rr_update_pyramid`, `rr_to_qimage`, `rr_blit_emit(C++)`,
  `commit_refresh_region`. **Done** (`commit_refresh_profile_4000`).
- [x] 5.2 A collapsed (single-rectangle) commit builds its emitted region image
  from the rectangle's sRGB buffer instead of a second `display_crop(0, union)`;
  several rectangles keep the union crop. Emit exactly one region blit. **Done.**
- [x] 5.3 Add `a_single_region_commit_blit_equals_the_level0_crop`, asserting the
  region-buffer image equals the level-0 crop pixel for pixel. **Done.**
- [x] 5.4 Confirm layer culling: `composite_pixels` (CPU) and `source_geom`
  (GPU) already clip each layer to its rect ∩ region and skip a non-intersecting
  layer, and `!layer.visible` skips hidden layers — no further culling is
  warranted; the region composite dominates for a large union and is inherent to
  the edited area. **Done.**
