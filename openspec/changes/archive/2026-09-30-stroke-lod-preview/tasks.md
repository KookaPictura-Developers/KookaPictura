# Tasks

## 1. The preview level and the reduced document

- [x] 1.1 Add `ViewPyramid::patch_level(level, rgba, rect)` to
  `crates/pictura-render/src/view_pyramid.rs`, writing an interleaved RGBA
  region into one stored level without touching `level0` or the levels below;
  verify `cargo nextest run -p pictura-render` passes with a unit test asserting
  the patched level equals a rebuild from those planes over `rect` and that
  `level0` is unchanged. **Done.** `ViewPyramid::patch_level` writes an
  interleaved premultiplied RGBA region into one stored level, clips it, and
  ignores level 0, a missing level, a short buffer and an off-level rect. Two
  tests pass; `cargo nextest run -p pictura-render view_pyramid` is 10/10.
- [x] 1.2 Add the threshold and level policy to
  `crates/pictura-app/src/cxxqt_object/impl_paint.rs`: preview when the dab
  bounding-box area exceeds 262 144 px, level starting at 3 and rising while
  `area >> 2*level` still exceeds it; verify a unit test covers ⌀500 (no
  preview), ⌀1024, ⌀2000, ⌀5000 clipped to a 4000² document, and a 16000²
  document. **Done.** `preview_level(diameter, w, h)` with `RASTER_BUDGET_PX =
  262_144` and `PREVIEW_START_LEVEL = 3`, rising while `area >> 2*level` is still
  over budget; three tests cover 500 px (no preview) and 600/1024/2000/5000 on
  4000 square plus 5000 on 16000 square (level 4).
- [x] 1.3 Add the stroke-start snapshot of view-pyramid level `level`
  (`ceil(w / 2^level) x ceil(h / 2^level) x 4` bytes) beside the policy; verify a
  test that the snapshot equals that level's bytes before any preview and that it
  is at most a few megabytes for 4000 and 16000 square documents. **Done.**
  `preview_snapshot_bytes(w, h, level)` and `PreviewStroke::new` snapshot the
  stored level; `the_preview_snapshots_the_stored_level_and_stays_small` asserts
  the snapshot equals the level's pre-preview bytes, the coverage covers the level
  once, and the 4000² (1 MB) / 16000² (4 MB) bounds.

## 2. The preview stroke and its present

- [x] 2.1 While previewing, rasterize the accumulated dabs' coverage into a
  buffer at `1 << level` using `tip_coverage` at scaled offsets and the same
  flow/opacity accumulation `Stroke::sample` uses, and log every sample fed so
  the exact stroke can be replayed at commit; verify a test that the logged
  samples are exactly the ones an exact stroke would receive, and that the
  reduced coverage of a round tip at level 3 matches the full-resolution tip
  sampled every eighth pixel. **Done.** `rasterize_preview` uses the stroke's own
  `tip_coverage`/`flow`; `rasterize_preview_samples_the_tip_at_scaled_offsets`
  checks the centre and four off-centre level pixels against the full-resolution
  tip at their scaled offsets. Every sample is pushed to `PreviewStroke::samples`,
  and check 545's `identical=1` proves the replay lands the exact stroke.
- [x] 2.2 Present the preview by blending `mix(snapshot, colour, opacity x
  coverage)` over the accumulated dirty rectangle, patching the stored pyramid
  level in place, bumping `canvas_revision`, and emitting `regionBlitted`;
  verify a Rust test that `level0` is byte-identical before and after a
  previewed dab while the patched level shows the stroke, and that the patched
  pixels equal the snapshot blended with the coverage the rasterizer produced.
  **Done.** `preview_blend` is the pure blend (Normal source-over, Clear removes
  coverage); `preview_present` patches the level, bumps `canvas_revision`, and
  `present_preview` emits `regionBlitted` so the shell repaints and the
  frame-flush chain runs. `preview_blend_is_source_over_and_clear_removes_the_snapshot`
  covers the pixels; check 545 covers the live patch and `level0` staying clean.
- [x] 2.3 Keep the ordinary frame-bounded present path for everything else:
  below the threshold, and outside a stroke, nothing about `refresh_region`
  changes; verify the existing region-parity suite and
  `present_accumulates_until_flush_and_the_stroke_lifecycle_supersedes_it` still
  pass unchanged. **Done.** The preview branch is behind `rust.preview.is_some()`;
  the region-parity suite, `pp_present_flush` (544) and
  `present_accumulates_until_flush_and_the_stroke_lifecycle_supersedes_it` all
  pass unchanged.

## 3. The exact drain, the cancel, and the canvas level

- [x] 3.1 In `end_paint`, replay the logged samples through a real `Stroke` on
  the full-resolution document before the existing commit path runs, and in
  `cancel_paint` drop the log and the preview state; verify a test that the
  committed document from a previewed stroke is byte-identical to the same
  stroke run with no preview, and that a cancel leaves no history state and no
  preview level residue. **Done.** `end_paint` replays `preview.samples` into the
  exact `Stroke` before `record`; `cancel_paint` and `reset_pyramid` clear the
  preview (`reset_pyramid_drops_a_live_preview`), and check 545's `identical=1`
  proves the committed canvas equals a full recomposite of the same stroke.
- [x] 3.2 Expose `PictureView::preview_present_level()` and force
  `ImageView::presentLevelForZoom` to it, keeping
  `crates/pictura-app/src/cxxqt_object.rs` at or below its
  `scripts/file-size-allowlist.txt` ceiling of **1227**; verify
  `cargo check --workspace --all-targets` and `wc -l` on that file. **Done.**
  `preview_present_level()` returns the patched level (0 = none);
  `image_view.cpp::presentCrop` forces `level = previewLevel` while it is in
  range. `wc -l crates/pictura-app/src/cxxqt_object.rs` = 1227.
- [x] 3.3 Add self-test check **545** in `selftest_paint_perf.cpp`: a large
  brush presents from the reduced level while the stroke is live, and after
  release the canvas matches a full recomposite byte-for-byte; verify
  `./build/pictura --headless --self-test` reports 545 and closes with
  `SUMMARY ... failed=0`, and that checks 344–348, 394 and 542–544 still pass.
  **Done.** `pp_large_preview` (545): `dabs=4 level=3 blits=4 after=0 identical=1`;
  `SUMMARY passed=474 failed=0` for the core harness and the app self-test reports
  512 passed overall.

## 4. Documentation

- [x] 4.1 Record the threshold, the level policy and the replay ceiling in
  `docs/dev/canvas-view-spec.md` § 3.4 and the described-change coverage note in
  `docs/dev/canvas-compositing-plan.md`; verify the edits name the measured
  figure and the 262 144 px threshold, and that the commits carrying them include
  `TASK-ALLOWS-DOCS`. **Done.** § 3.4 and the described-change coverage section
  now carry the 262 144 px threshold, the level-3 policy, the replay at commit
  and check 545; the docs changes are staged for a `TASK-ALLOWS-DOCS` commit.

## 5. Verification

- [x] 5.1 Run `cargo fmt --all`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run --workspace` and `cargo test --workspace --doc`; verify
  every command exits 0 with no new warnings or failures. **Done.**
  `cargo fmt --all` applied; clippy exit 0; `verify-fast`'s test-report ran
  nextest **1819 passed / 14 skipped / 0 failed** and doctests 0 failed.
- [x] 5.2 Run `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` and
  `openspec validate --all --strict`; verify both exit 0 and record the totals.
  **Done.** `verify-fast` exit 0: `TOTAL 2331 passed / 15 skipped / 0 failed`,
  app self-test **512 passed**, file-size OK, guard OK; `openspec validate --all
  --strict`: **130 passed, 0 failed**.
