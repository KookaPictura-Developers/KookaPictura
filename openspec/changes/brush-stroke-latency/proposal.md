# Proposal

## Why

A 500 px brush on a 4000×4000 document starts slowly and lags under the
pointer. Profiling the paint path shows two causes that the canvas-view budget
(`docs/dev/canvas-view-spec.md`: input-to-first-pixel ≤ 16 ms, ≥ 60 FPS) already
forbids: the per-dab rasterizer recomputes stroke-constant tip state and
re-resolves the target layer and its four channels for *every changed pixel*, and
every input event that places a dab runs a full region composite and present with
no frame bound. The existing profile (`paint_dab_profile_4000`) times only the
region composite, so the raster half of the budget is currently unmeasured.

## What Changes

- **Measure the split first.** An ignored profile separates stroke start,
  rasterization, and the region present for two brush diameters and two layer
  counts, so the rest of the change is ordered by numbers rather than by
  inspection.
- **Stroke-constant tip state.** The tip's angle rotation, radius, roundness
  scale, hardness core and aliased flag are computed once per stroke; a dab's
  coverage becomes a pure function of the offset, with no trigonometry in the
  pixel loop.
- **Resolve the layer and its planes once per sample.** The target layer, its
  transparency-lock state, and its four channel planes are looked up when a
  sample starts rather than per changed pixel.
- **Frame-bounded in-stroke present.** Dabs accumulate into one pending region
  that is presented at most once per frame interval; `PictureView` exposes an
  explicit `flush_present()`, and the stroke commit flushes before recording
  history. The reported dirty rectangle covers the dabs since the last present,
  never the cumulative stroke union.
- **Out of scope.** Stroke-start document cloning is a storage change
  (`cow-pixel-storage`), and tiled history, GPU stroke rendering and a
  reduced-resolution stroke preview are separate later changes.
- **BREAKING**: during a stroke, `regionBlitted` fires once per presented frame
  instead of once per input event. A consumer that needs per-dab regions calls
  `PictureView::flush_present()`; the commit path flushes automatically.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tools/paint-engine`: the per-dab incremental dirty rectangle is redefined as
  the footprints of the dabs since the last present (still never the cumulative
  stroke union), and the existing dab-latency budget gains a large-brush
  rasterization scenario.
- `ui/document-canvas`: a new requirement bounds the in-stroke present rate to a
  frame interval with an explicit flush, coexisting with the region-blit and
  present-cache coherence contract.

## Impact

- `crates/pictura-paint/src/{tip.rs,stroke.rs}` — tip parameters, the sample
  loop, `composite_pixel`, `take_dirty`.
- `crates/pictura-app/src/cxxqt_object/impl_paint.rs` — pending-present
  accumulation and `flush_present`; `end_paint` / `cancel_paint` flush or drop it.
- `crates/pictura-app/cpp/frame.cpp`, `tool_brush.cpp` and the other paint-tool
  handlers — a single-shot frame timer driving `flush_present`.
- `crates/pictura-app/cpp/selftest_paint_perf.cpp` — the per-dab region check
  (exit code 344) calls `flush_present()` between dabs.
- `crates/pictura-app/src/cxxqt_object/tests_impl.rs` — the measurement profile.
- No new dependencies; no change to the composite, pyramid, or history
  contracts beyond the signal timing above.
