# Design

## Context

See proposal.md — Why.

The measurement that drives every choice below is `large_brush_profile_4000`
(release, 4000², one layer): a dab's rasterization runs at **58 ns per pixel of
its bounding box**, and its region present at roughly the same rate over the
region it damages. The 16 ms Target therefore allows 262 144 px. A ⌀500 dab is
250 000 px and fits; a ⌀5000 dab clipped to the document is 16 MP and does not.

Everything that does not change the *number of pixels* has already been spent:
`TipParams` removed the per-pixel trigonometry, the `Stencil` removed the
per-pixel layer and channel lookups, `Plane` made the document clones refcount
bumps, and the frame-bounded present coalesces the compositing. Halving the
constant again would take 933 ms to ~450 ms, not to 16 ms.

## Goals / Non-Goals

**Goals:**

- Input-to-first-pixel stays inside the budget for every brush size the options
  bar allows, by rasterizing fewer pixels while the button is down.
- Every pixel the document keeps after `end_paint` is byte-identical to what the
  exact stroke would have produced — no tolerance, no re-derivation at undo.
- The preview reuses the `ViewPyramid` levels, the `tip_coverage` profile and
  the cover~flow accumulation rather than introducing a second stroke engine;
  it rasterizes coverage into one stored level over a snapshot of that level.

**Non-Goals:**

- A background thread. The exact stroke still runs on the GUI thread; see the
  trade-off below.
- Previewing below the threshold, where the exact raster already fits.
- Exact reduced-scale resampling of the layer stack: the preview blends the
  paint colour over a snapshot of the composited level, so a masked, grouped or
  non-Normal target is approximate (Clear is handled; Dissolve and Behind
  present as Normal). Only the committed full-resolution pixels must be exact.

## Decisions

**D1 — One constant for the threshold and the level.** Preview begins when
`dab bounding box area > 262 144` px (⌀≈512), the area 16 ms buys at the
measured 58 ns/px. The level starts at 3 (1/8) and rises while
`area >> 2*level > 262 144`, so the preview's own rasterization and its present
fit the same budget. A fixed starting level keeps the reduced document's
construction at `total plane bytes / 64` — 8 MB for an eight-layer 4000²
document, about 8 ms once per stroke — which is what makes the preview cheaper
than the dab it replaces even for a single click. *Alternatives considered:*
a level derived only from the diameter — it forgets that a dab clipped to the
document is cheaper than its nominal box; a time-based adaptive threshold — it
needs a clock in the hot path and produces a non-deterministic stroke.

**D2 — The preview renders the tip over a snapshot of the reduced composite,
not a second stroke on a reduced document.** While previewing, the app keeps a
snapshot of view-pyramid level `level` taken at stroke start — at most a few
megabytes, `(w >> level) × (h >> level) × 4` — and per frame rasterizes the
accumulated dabs' coverage into a reduced buffer using the same
`tip_coverage` profile at `1 << level` scaled offsets, then blends
`mix(snapshot, paint colour, opacity × coverage)` and patches it into the stored
level. Nothing in `pictura-paint` changes, and no second `Stroke` exists. The
exact `Stroke` is not started until the drain runs.

*Alternatives considered:* building a reduced copy of the document and running a
second `Stroke` on it — it composites the layer stack faithfully, but reading
every plane to box-filter it costs O(document) per stroke start, about 50 ms at
4000² over eight layers and roughly half a second at 16000², which is exactly
the cost this change exists to remove; subsampling the exact stroke — it would
leave full-resolution pixels in `level0` and could not be undone. The snapshot
blend is an approximation of the layer stack rather than a resampling of it:
for the ordinary case of an opaque target layer in Normal mode it is identical,
and for a masked, grouped or non-Normal layer it is a plausible picture of a
stroke whose exact pixels land at commit.

**D3 — The preview writes one stored pyramid level, not `level0`.** The preview
present blends against the snapshot over the accumulated dirty rectangle and
patches `ViewPyramid`'s level `level` in place, then bumps `canvas_revision`. It
deliberately does **not** call `update`, which would recompute that level from
`level0` and wipe the preview; `level0` is left exactly as it was, so a cancel
or a commit repairs everything with the ordinary region path. Cost is
`area >> 2*level` pixels — at most 262 144 — instead of the `area` pixels a
`level0` patch would write.

**D4 — The canvas clamps its present level while a preview stroke runs.**
`ImageView::presentLevelForZoom` returns `max(zoomLevel, previewLevel)` while
`PictureView::is_previewing()`, so the crop comes from the level the preview
patched. The whole in-progress image drops to preview resolution, which is the
point; the commit's `refresh_region` patches `level0` and runs `update`, which
rebuilds the level the preview touched and hands the zoom-selected level back.

**D5 — The exact stroke is replayed before the history state is recorded.**
`paint_dab` logs each sample while previewing; `end_paint` replays the log
through a real `Stroke` on the full-resolution document, then takes the
existing commit path unchanged. `cancel_paint` drops the log and runs the
existing restore, which patches `level0` and rebuilds the pyramid — repairing
the preview level as a side effect.

**D6 — The replay is synchronous, and that is a recorded ceiling.** At ⌀5000 the
replay is the same ~933 ms the raster costs today, moved from "before the first
pixel appears" to "after the button is released". That is the right trade for a
single dab (the preview costs ~10 ms against 933 ms) and it is a poor one for a
long stroke of many large dabs, which would freeze for the sum at release.
Moving it to a worker thread — or draining it a sample per frame tick so the
freeze is interleaved with input — is the follow-up, and needs a threading story
this codebase does not have yet. Recorded here rather than guessed at.

## Risks / Trade-offs

- [The replay at commit can freeze for the whole stroke's exact work] → D6,
  with the drain-per-tick variant named as the upgrade; the threshold keeps the
  freeze off every brush except the ones that are already unusable today.
- [`level0` and the preview level disagree between the first dab and the
  commit] → Only the preview level is written, and only `level0` is authoritative
  afterwards; a reader of `PictureView::image()` mid-stroke sees the pre-stroke
  document, which is what panels already see mid-stroke.
- [`blitRegion` writes a preview-sized region into the full-resolution
  `image_` at preview coordinates] → Harmless while the level crop is forced
  (the canvas never draws `image_` then) and repaired by the commit's blit over
  the same bounding box; the self-test that compares the canvas mid-stroke
  against the commit covers it.
- [The snapshot blend approximates the layer stack instead of resampling it] →
  The preview is presentational and is thrown away, so only the exact stroke
  needs to match the oracle; the preview's job is to be plausible, not exact,
  and the ordinary opaque-Normal case is identical.
- [The snapshot goes stale if the document changes mid-stroke] → Only a stroke
  writes pixels mid-stroke, and it writes the preview level, never `level0`, so
  the snapshot's source is unchanged until the commit; a cancel or a commit
  rebuilds the level from `level0` and drops the snapshot.
- [The `is_previewing` invokable pushes `cxxqt_object.rs` past 1227 lines] →
  Free the lines by folding a multi-line doc comment in the same block, exactly
  as `flush_present` did; the file-size gate fails the build if it is missed.

## Open Questions

None that would change the spec, the approach or the task breakdown. Whether the
replay runs on a worker thread is a recorded follow-up (D6), not a decision this
change waits on: the contract — approximate while down, exact before the history
state — is the same either way.
