# Tasks

## 1. The preview level and the reduced document

- [ ] 1.1 Add `ViewPyramid::patch_level(level, planes, rect)` to
  `crates/pictura-render/src/view_pyramid.rs`, writing an interleaved RGBA
  region into one stored level without touching `level0` or the levels below;
  verify `cargo nextest run -p pictura-render` passes with a unit test asserting
  the patched level equals a rebuild from those planes over `rect` and that
  `level0` is unchanged.
- [ ] 1.2 Add the threshold and level policy to
  `crates/pictura-app/src/cxxqt_object/impl_paint.rs`: preview when the dab
  bounding-box area exceeds 262 144 px, level starting at 3 and rising while
  `area >> 2*level` still exceeds it; verify a unit test covers ⌀500 (no
  preview), ⌀1024, ⌀2000, ⌀5000 clipped to a 4000² document, and a 16000²
  document.
- [ ] 1.3 Add the reduced-document builder (same layer tree, every plane
  box-filtered by `1 << level`) beside the policy; verify a test that each
  reduced plane has the expected dimensions and that its corner and centre
  samples equal the box mean of the source region.

## 2. The preview stroke and its present

- [ ] 2.1 In `begin_paint`, above the threshold, build the reduced document and
  start a second `Stroke` on it with the diameter, spacing and coordinates
  scaled by `1 << level`, logging every sample fed to it; verify a test that a
  previewed stroke's log replays to the same samples an exact stroke receives.
- [ ] 2.2 Present the preview by compositing the reduced document over its
  dirty rectangle, patching the stored pyramid level in place, bumping
  `canvas_revision`, and emitting `regionBlitted`; verify a Rust test that
  `level0` is byte-identical before and after a previewed dab while the patched
  level shows the stroke, and that the patched level equals a rebuild from the
  reduced document.
- [ ] 2.3 Keep the ordinary frame-bounded present path for everything else:
  below the threshold, and outside a stroke, nothing about `refresh_region`
  changes; verify the existing region-parity suite and
  `present_accumulates_until_flush_and_the_stroke_lifecycle_supersedes_it` still
  pass unchanged.

## 3. The exact drain, the cancel, and the canvas level

- [ ] 3.1 In `end_paint`, replay the logged samples through a real `Stroke` on
  the full-resolution document before the existing commit path runs, and in
  `cancel_paint` drop the log and the preview state; verify a test that the
  committed document from a previewed stroke is byte-identical to the same
  stroke run with no preview, and that a cancel leaves no history state and no
  preview level residue.
- [ ] 3.2 Expose `PictureView::is_previewing()` and clamp
  `ImageView::presentLevelForZoom` to it, keeping
  `crates/pictura-app/src/cxxqt_object.rs` at or below its
  `scripts/file-size-allowlist.txt` ceiling of **1227**; verify
  `cargo check --workspace --all-targets` and `wc -l` on that file.
- [ ] 3.3 Add self-test check **545** in `selftest_paint_perf.cpp`: a large
  brush presents from the reduced level while the stroke is live, and after
  release the canvas matches a full recomposite byte-for-byte; verify
  `./build/pictura --headless --self-test` reports 545 and closes with
  `SUMMARY ... failed=0`, and that checks 344–348, 394 and 542–544 still pass.

## 4. Documentation

- [ ] 4.1 Record the threshold, the level policy and the replay ceiling in
  `docs/dev/canvas-view-spec.md` § 3.4 and the described-change coverage note in
  `docs/dev/canvas-compositing-plan.md`; verify the edits name the measured
  58 ns/px figure and the 262 144 px threshold, and that the commits carrying
  them include `TASK-ALLOWS-DOCS`.

## 5. Verification

- [ ] 5.1 Run `cargo fmt --all`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run --workspace` and `cargo test --workspace --doc`; verify
  every command exits 0 with no new warnings or failures.
- [ ] 5.2 Run `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` and
  `openspec validate --all --strict`; verify both exit 0 and record the totals
  (`test-report` passed/failed, self-test summary, openspec items) in the task
  result.
