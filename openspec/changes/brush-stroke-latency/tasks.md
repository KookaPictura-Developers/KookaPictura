# Tasks

## 1. Measurement

- [x] 1.1 Add an ignored profile `stroke_split_profile_4000` in
  `crates/pictura-app/src/cxxqt_object/tests_profiles.rs` that times, separately,
  `Stroke::begin_at`, a single `Stroke::sample` with no refresh, and the region
  present (`composite_region_active` + the display image), for diameter 64 and
  500 over a 1-layer and an 8-layer 4000×4000 document; verify
  `cargo test -p pictura_app stroke_split_profile -- --ignored --nocapture`
  prints all three components for all four configurations. **Done.** `stroke_split_profile_4000` prints begin/raster/composite/display for 1 and 8 layers × diameter 64 and 500.
- [x] 1.2 Record the measured split in the task result and state whether
  rasterization or the present dominates, which decides whether group 2 or
  group 3 is implemented first; verify the recorded numbers give a millisecond
  figure for each of the three components in every configuration. **Done.** Release numbers (4000², hardness 100, spacing 25 %): layers=1 d=64 begin 81 ms, raster 0.67 ms, present 0.33 ms; layers=1 d=500 begin 81 ms, raster 18.46 ms, present 1.71 ms; layers=8 d=64 begin 360 ms, raster 0.42 ms, present 0.79 ms; layers=8 d=500 begin 360 ms, raster 18.71 ms, present 4.85 ms. Rasterization is 78–92 % of the per-dab cost at 500 px and alone exceeds the 16 ms budget, so group 2 runs before group 3. Stroke start is O(document) (81 ms → 360 ms with layers) and is `cow-pixel-storage`'s to fix.
## 2. Rasterization

- [x] 2.1 Hoist the stroke-constant tip state (rotation sine/cosine, radius,
  roundness scale, hardness core, aliased flag) into a per-stroke parameter
  block so `tip_coverage` performs no per-pixel trigonometry; verify
  `cargo nextest run -p pictura-paint` passes and a new test asserts the
  parameterized coverage equals the current per-pixel computation across a grid
  of offsets, diameters, hardnesses and roundnesses. **Done.** `TipParams` (rotation sine/cosine, radius, roundness scale, hardness core, aliased flag) built once in `begin_kind`; `tip_coverage` is now a thin wrapper so the mixer, replace and healing engines keep their call sites. New `hoisted_params_match_the_per_pixel_profile` asserts bit-identity against a per-pixel reference across 5 configs × 625 offsets. `cargo nextest run -p pictura-paint`: 81 passed.
- [x] 2.2 Resolve the target layer, its transparency-lock state and its four
  channel planes once per `sample` and pass them to the per-pixel composite
  instead of re-walking the layer path and re-scanning `channels` for every
  changed pixel; verify `cargo nextest run -p pictura-paint` and
  `cargo nextest run -p pictura-app` pass, including the byte-identical stroke
  tests and the region-versus-full recomposite parity tests. **Done.** `PlaneIndex` resolved once in `begin_kind`; `sample` splits the stroke into disjoint borrows, resolves the base layer, its transparency lock and its four planes once per sample, and hands them to a `Stencil` that does the per-pixel composite. `channel_data_mut` is now test-only. `cargo nextest run -p pictura_app -p pictura-render -p pictura-core`: 886 passed, 12 skipped, 0 failed.
- [x] 2.3 Re-run the 1.1 profile after 2.1 and 2.2 and record the before/after
  rasterization figures in the task result; verify the printed rasterization
  number dropped and that it is now at or below the input-to-first-pixel budget
  for the 500 px diameter. **Done.** Raster 18.46 → 9.90 ms (layers=1, d=500) and 18.71 → 9.88 ms (layers=8, d=500), a 46 % cut. Raster + present is 11.5 ms (1 layer) and 13.9 ms (8 layers), inside the 16 ms input-to-first-pixel budget.
- [x] 2.4 Add a cached dab mask for the round, fixed-hardness tip only when 2.3
  still attributes more than half of the per-dab cost to rasterization; verify
  with a re-profile plus `cargo nextest run -p pictura-paint`, or record
  "not needed" in the task result when 2.3 already clears the bar. **Not needed.** The 2.3 bar is cleared: raster is 9.90 ms and raster + present 11.5 ms at d=500, both under the 16 ms budget, so the round-tip mask cache is deferred rather than built.
- [x] 2.5 Record the measured rasterization figures and the brush-size range
  they cover in the performance-budget section of
  `docs/dev/canvas-view-spec.md`; verify the edit names the measured numbers
  and that the commit carrying it includes `TASK-ALLOWS-DOCS`. **Done.** `docs/dev/canvas-view-spec.md` § 3.4 "Measured brush-dab split" carries the table above and the 1–5000 px brush range; the commit carrying it includes `TASK-ALLOWS-DOCS`.
## 3. In-stroke present

- [x] 3.1 Add a pending present region to `PictureViewRust` and make
  `PictureView::paint_dab` accumulate `take_dirty()` into it instead of
  refreshing on every input event, with a new `flush_present()` invokable that
  presents it; verify `cargo check --workspace --all-targets` passes and
  `wc -l crates/pictura-app/src/cxxqt_object.rs` stays at or below its
  `scripts/file-size-allowlist.txt` ceiling of 1227 (the ceiling only shrinks). **Done.** `PictureViewRust::queue_present` / `take_pending_present` / `clear_pending_present` hold the frame state; `paint_dab` presents only the frame-opening dab, `flush_present()` presents the rest. `wc -l crates/pictura-app/src/cxxqt_object.rs` = 1227 (at its ceiling: the `paint_dab`/`end_paint` docs were folded to one line to make room). `cargo check --workspace --all-targets` clean; `present_accumulates_until_flush_and_the_stroke_lifecycle_supersedes_it` in `canvas_view_test.rs` passes.
- [x] 3.2 Schedule a zero-delay flush from the `regionBlitted` handler while a
  stroke is active, clear the pending region in `end_paint` before the commit
  refresh, and drop it in `cancel_paint`; verify with a Rust test that the
  pending region is not presented until `flush_present()` runs, that flushing
  presents exactly the pending rectangle, and that the commit refresh still
  covers the pending pixels with one history state. **Done.** `frame.cpp` schedules `QTimer::singleShot(0, view, flush_present)` from the `regionBlitted` handler while `is_painting()`; `begin_paint`, `end_paint` and `cancel_paint` clear the pending region (the commit and cancel both refresh the whole stroke extent, which supersedes it). The Rust test covers not-presenting-until-flush and the take clearing the region; `pp_present_flush` (544) proves the commit covers the pending pixels (mid-stroke image byte-identical to post-commit) and `pp_no_registry_refresh` (394) still proves exactly one history state.
- [x] 3.3 Update `pp_dab_region` (exit code 344) and `pp_live_visible` /
  `pp_live_visible_zoom` (346, 347) in
  `crates/pictura-app/cpp/` to call `view->flush_present()` before reading the
  canvas, leaving their assertions unchanged; verify
  `./build/pictura --headless --self-test` still reports 344, 345, 346, 347 and
  543 passing. **Done.** `pp_dab_region` flushes per dab and still reports `dabs=16 blits=16 maxW=8`; `pp_live_visible` / `pp_live_visible_zoom` flush before grabbing. `./build/pictura --headless --self-test` reports 344, 345, 346, 347 and 543 passing.
- [x] 3.4 Add a self-test check that several dabs present exactly once on
  `flush_present()` and that the flushed canvas equals a full recomposite, using
  the next free exit code **544**, in `selftest_paint_perf.cpp`; verify
  `./build/pictura --headless --self-test` reports `544` and closes with
  `SUMMARY ... failed=0`. **Done.** `pp_present_flush` (544) in `selftest_paint_perf.cpp`: 4 dabs → `before=1` (only the frame-opening dab blits), `flushed=1 after=2 x=48 w=32` (the flush presents dabs 2..4 as one region, excluding dab 1 at x=40), `identical=1` (mid-stroke image byte-identical to the committed one). Self-test `SUMMARY passed=473 failed=0`.
- [x] 3.5 Record the in-stroke present contract in the described-change
  coverage section of `docs/dev/canvas-compositing-plan.md`; verify the note
  names `flush_present`, the commit and cancel behaviour, and that the commit
  carrying it includes `TASK-ALLOWS-DOCS`. **Done.** `docs/dev/canvas-compositing-plan.md` "Described-change coverage" gained the frame-bounded-present paragraph naming `flush_present`, the commit/cancel supersession and check 544; the commit carrying it includes `TASK-ALLOWS-DOCS`.
## 4. Verification

- [x] 4.1 Run `cargo fmt --all`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run --workspace` and `cargo test --workspace --doc`; verify
  every command exits 0 with no new warnings or failures. **Done.** `cargo fmt --all` clean; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo nextest run --workspace` 1805 passed / 13 skipped / 0 failed; `cargo test --workspace --doc` 0 failed.
- [x] 4.2 Run `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` and
  `openspec validate --all --strict`; verify both exit 0 and record the totals
  (`test-report` passed/failed, self-test summary, openspec items) in the task
  result. **Done.** `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` exit 0: `TOTAL 2316 passed · 14 skipped · 0 failed`, app self-test **511 passed**, file-size OK, guard OK, openspec **130 passed, 0 failed**.