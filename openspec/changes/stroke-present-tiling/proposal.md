# Proposal

## Why

A large-brush stroke on a 4000² document still costs **13–43 ms per dab** in the
GPU present path (`gpu_patch_level0_pyramid`): `gpu_dab` patches level 0 and
then rebuilds the CPU view pyramid for the dab rectangle on **every** dab, and
the exact path pays the same rebuild on each present through
`refresh_region`'s `rr_update_pyramid`. The frame-bounded present already
coalesces the *pixels*; what remains per-dab is the pyramid walk and a
bounding-box composite at commit.

## What Changes

- **The pyramid is rebuilt once per frame, not once per dab.** The present is
  frame-bounded: the dab that opens a frame presents through `refresh_region`
  (which folds its region into the pyramid) and the frame's later dabs accumulate
  until the flush. `canvas_revision` is bumped once per presented frame, not per
  dab. (The GPU-stroke deferral this originally sketched was superseded by
  `gpu-stroke-rendering`, which presents each frame from the exact working
  document rather than a level-0 patch.)
- **The canvas shows level 0 while a GPU stroke presents.** `preview_present_level`
  returns `-1` during a GPU stroke; the canvas then crops level 0 — whose region
  blits keep it current.
- **The pyramid update is tile-aligned.** `ViewPyramid::update` rounds the
  damaged rectangle out to a 64×64 grid before walking the levels, so a frame's
  dabs share one stable grid and the per-level filter footprint (a level-`N`
  texel reads a 2×2 block at `N-1`) is applied on top. Level 0 stays the
  caller's single buffer; no full-frame detach.
- **The commit composites dirty tiles, not the stroke's bounding box.** The
  stroke's dirty area is tracked as a document-space tile bitmap; `end_paint`
  decomposes it into disjoint rectangles (row RLE plus a greedy vertical merge)
  and composites each, collapsing back to the union when the tiles fill most of
  their bounding box or exceed a cap. A short diagonal stops recompositing its
  full bounding box.
- **A collapsed commit blits the region buffer it already composited.** When the
  dirty tiles collapse to one rectangle, the emitted region image is the
  rectangle's sRGB buffer premultiplied, not a second crop of level 0; the commit
  still emits exactly one region blit, so the shell refreshes the command
  registry once.
- **One new self-test (547)** proves the frame pyramid is rebuilt once per frame
  with the same committed pixels.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `compositing/view-pyramid`: the damage update rounds out to a tile grid.
- `compositing/described-damage`: a paint commit decomposes its dirty area into
  tiles.
- `tools/paint-engine`: the live stroke presents and rebuilds the pyramid
  frame-bounded.
- `ui/document-canvas`: the canvas crops level 0 while a GPU stroke presents.
- `verification/verification-harness`: self-test 547.

## Impact

- `crates/pictura-render/src/view_pyramid.rs` — the tile constant, the outward
  alignment in `update`, and one unit test.
- `crates/pictura-app/src/cxxqt_object/state.rs` — `pending_pyramid` and
  `stroke_tiles`.
- `crates/pictura-app/src/cxxqt_object/helpers.rs` — the `TileSet` bitmap, its
  rectangle decomposition and collapse heuristic.
- `crates/pictura-app/src/cxxqt_object/impl_paint.rs` — deferred/flushed
  pyramid, per-dab tile marking, multi-rect commit, the `-1` present sentinel.
- `crates/pictura-app/src/cxxqt_object/impl_core.rs` — `reset_pyramid` clears
  the deferral state.
- `crates/pictura-app/cpp/image_view.cpp` / `.h` — the canvas honours the `-1`
  present sentinel.
- No new dependencies; committed pixels are unchanged.
