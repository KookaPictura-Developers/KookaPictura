# Design

## Context

See proposal.md — Why.

The present facts this design rests on, all measured or read from the current
code:

- `Stroke::begin_kind` deep-clones the document twice and allocates two
  layer-sized coverage buffers; one snapshot clone measures 62 ms at 4000².
  That cost is a storage problem and belongs to `cow-pixel-storage`, not here.
- `Stroke::sample` walks the dab's full bounding box and calls `tip_coverage`
  per pixel, which recomputes `theta.sin_cos()`, the radius, the roundness scale
  and the hardness core — all constants for the life of the stroke. A 500 px dab
  is ~250 000 pixels.
- `composite_pixel` resolves the target layer twice and runs four
  `channels.iter().find()` scans per plane access, so roughly a dozen iterator
  scans per changed pixel.
- `PictureView::paint_dab` composites, converts, patches the level-0 frame,
  repairs the pyramid and emits `regionBlitted` on **every** input event that
  placed a dab. The GPU region composite measures 2.5 ms and the CPU fallback
  17.5 ms per 512² dab; the Qt `paintEvent` that consumes `regionBlitted` is
  already coalesced by the event loop, but the Rust composite is not.
- The only profile today, `paint_dab_profile_4000`, times the region composite
  and the buffer-to-image copy. Rasterization and stroke start are unmeasured.

## Goals / Non-Goals

**Goals:**

- Attribute the per-dab cost to stroke start, rasterization and present before
  changing anything, so the task order is decided by numbers.
- Remove all stroke-constant work from the pixel loop.
- Bound the in-stroke composite and present to the event loop turn rather than
  the input event rate.
- Leave every existing pixel result byte-identical.

**Non-Goals:**

- Stroke-start and history memory — `cow-pixel-storage`.
- Tiled history, GPU stroke rendering, reduced-resolution stroke preview —
  separate later changes.
- Changing the composite, pyramid, damage or history contracts beyond the
  `regionBlitted` timing stated in the spec delta.

## Decisions

**D1 — Measure first, then order the rest.** The change opens with an ignored
profile that times `Stroke::begin_at`, a single `Stroke::sample` with no
refresh, and the `refresh_region` composite + present, for diameter 64 and 500
over a 1- and an 8-layer 4000² document. Rasterization and stroke start have
never been measured; every ordering claim below is a hypothesis until this
prints numbers. *Alternative considered:* optimize the obvious hot loop first.
Rejected — the previous change shipped three optimizations against an
unmeasured rasterizer.

**D2 — Hoist tip constants into a per-stroke parameter block, not a dab-mask
cache.** `tip_coverage` becomes a function of a `TipParams` (rotation sin/cos,
radius, roundness scale, hardness core, aliased flag) computed once in
`begin_kind`. A pre-rendered mask cache (Krita's `KisDabCache`, MyPaint's
proposed stamp cache) would cut the remaining profile arithmetic too, but it
needs an invalidation key across size/roundness/hardness/angle and costs memory
per distinct tip. Ship the parameter block first; add the cache only if task 0
still attributes most of the raster cost to the profile after D2 and D3.

**D3 — Resolve the layer and its planes once per `sample`, pass them down.**
`composite_pixel` takes the pre-resolved base and working layer references plus
the four channel slices instead of re-walking the layer path and re-scanning
`channels` per pixel. *Alternative considered:* caching the four slices on the
`Stroke` — rejected, the borrow checker fights `&mut self` for no gain over a
per-call local.

**D4 — Coalesce by accumulating a pending region in `PictureView`, presented
only by `flush_present()`.** `paint_dab` merges `take_dirty()` into
`pending_present` and emits a new `presentPending` signal the first time the
region goes empty to non-empty; `frame.cpp` connects it to
`QTimer::singleShot(0, view, flush_present)`. A zero-delay single-shot runs on
the next event-loop turn, so every input event the loop drained in that turn
coalesces into one composite and one present, and an idle pointer adds no
latency. *Alternatives considered:* a 16 ms frame timer — adds up to one frame
of input-to-first-pixel latency against a 16 ms budget, rejected; coalescing
inside `tool_brush.cpp` — the other paint tools call `paint_dab` too, rejected;
coalescing the Qt paint only — `update()` is already coalesced by the event
loop, so it buys nothing, rejected.

**D5 — Commit supersedes the pending region; cancel drops it.** `end_paint`
already refreshes the whole stroke rectangle against the committed document
before `record()`, which strictly covers the pending region, so the pending
region is cleared rather than flushed. `cancel_paint` discards it: its pixels
are in the working document that is about to be thrown away.

**D6 — Consumers that need a mid-stroke present call `flush_present()`.** Four
self-test sites paint and then read the canvas without releasing the stroke
(`pp_dab_region` 344, `pp_present_cache` 345, `pp_live_visible` 346/347,
`ldoc_paint_presents_from_crop` 543). Each gains an explicit
`view->flush_present()`; their assertions are unchanged. The commit-path tests
need no edit because D5 covers them.

## Risks / Trade-offs

- [The 16 ms input-to-first-pixel budget is missed for a 500 px dab even after
  D2/D3] → Task 0 prints the raster number before D2 lands; if it is still over
  budget afterwards, add the dab-mask cache from D2's alternative and re-profile
  rather than weakening the budget scenario.
- [`pending_present` makes the displayed canvas lag the document by up to one
  event-loop turn] → The lag is bounded by the loop turn, `flush_present()` is
  public for consumers that need it now, and `end_paint` supersedes the region
  so no released stroke can be stale.
- [A new `presentPending` signal widens the bridge] → One signal, one invokable,
  both trivially greppable; the alternative (a C++-owned timer inside the
  cxx-qt object) is not expressible without extending codegen.
- [The dirty-rectangle rewording in `tools/paint-engine` could be read as
  weakening `lpe_dirty_per_dab`] → The scenario now says "since the previous
  present" and adds `lpe_dirty_since_last_present`, which asserts the rect is
  no larger than the union of those dabs. It still cannot grow with the stroke.

## Open Questions

None. The task-0 gate orders tasks 2 and 3; it does not change the specs, the
approach or the task breakdown.
