# Tasks

## 1. Stage A — premultiplied display conversion and zoom filter

- [x] 1.1 Add a dedicated premultiplied display conversion in `crates/pictura-app/src/cxxqt_object/helpers_composite.rs`, leaving `buffer_to_rgba_bytes`, `buffer_to_image`, and `rgba_image` straight-alpha. Verify with a unit test that a display pixel with alpha `a < 255` becomes `round(c * a / 255)` with unchanged alpha, an alpha-0 pixel has zero colour, and the export/thumbnail helpers are byte-identical to before.
- [x] 1.2 Select the canvas sampling filter by zoom: below 200 % smooth, at or above 200 % nearest, in `crates/pictura-app/cpp/image_view.cpp`. Verify with a self-test check that asserts the boundary choice at 200 %.
- [x] 1.3 Run `cargo nextest run -p pictura-app` and `cargo test --workspace --doc`; verify no image or export test regressed and the export encoder's straight-alpha bytes are unchanged.

## 2. Stage A — canvas damage account

- [x] 2.1 Add a damage account with `edited`, `mark(rect)`, and `take(canvas) -> rect` that clips to the canvas and clears on take, plus unit tests: nothing changed is empty; described changes union; an undescribed edit yields the whole canvas; a mark cannot mask an earlier undescribed edit; taking starts afresh.
- [x] 2.2 Replace the `display_dirty` boolean with the account across `state.rs`, `impl_core.rs`, `impl_history.rs`, and `impl_transform/dispatch.rs`, keeping `region_blitted`/`changed` semantics and the no-`changed`-on-region rule. Verify exit codes 75, 76, 82, 83, 84, 85, 344, 345, and 363 still pass and region paths emit `changed == 0`.

## 3. Stage A — crop-from-source present at zoom ≥ 100 %

- [x] 3.1 In `image_view.cpp` `paintEvent`, at zoom ≥ 1 crop the visible document rect from `image_` and draw only that, extending the cache key with the document rect. Verify with a new self-test check in `selftest_canvas_view.cpp` that the cropped present is pixel-identical to the transform draw at the same filter and that a pan does not change the source. (Subsumed by 6.1: the level-0 crop is the zoom ≥ 1 present, checked byte-identical by exit 76.)
- [x] 3.2 Add a self-test check that a high-zoom document past the former 64 MP bound presents through the crop and does not fall back to drawing the full-resolution source. (Subsumed by 6.1: the 64 MP cap and full-resolution fallback were removed.)

## 4. Stage B — view pyramid in `pictura-render`

- [x] 4.1 Add `crates/pictura-render/src/view_pyramid.rs`: a `Planes<'_>` borrowed straight-alpha level-0 view supplied per call (not stored), whole-level premultiplied levels below 0 halving to a 256 px long side, and `rebuild`, damage-driven rect `update` reading the borrowed level 0, and a premultiplied `crop`. Verify with tests: levels halve to the smallest side; a small composite is its only level; level 0 is supplied, not copied; an update equals a rebuild at every level; odd-edge averaging keeps full coverage; a crop returns the level's pixels premultiplied; a level-0 crop is premultiplied; a crop past the level is transparent.
- [x] 4.2 Export the type from `crates/pictura-render/src/lib.rs`. Verify `cargo nextest run -p pictura-render` passes.

## 5. Stage B — `PictureView` owns the pyramid and exposes crops

- [x] 5.1 Add the pyramid and damage account to `PictureViewRust`, update them in `recomposite` and `refresh_region`, and reset/rebuild them in `undo`, `redo`, `history_jump`, `history_restore_snapshot`, and the move-preview paths. Verify with unit tests that a region update equals a full rebuild and that each history/move path resets the pyramid and damage.
- [x] 5.2 Add bridge invokables `display_image(level, x, y, w, h) -> QImage`, `display_level_count() -> i32`, `display_level_size(level) -> QString`, and `take_canvas_damage() -> QString` using the existing `QString`/`i32` marshalling. Verify the generated header compiles and a Rust unit test covers the level queries and that an out-of-range level, an empty rectangle, and a rectangle past the level return an empty image without panicking.
- [x] 5.3 Pass the borrowed planar level-0 view (after the app has applied the damage to its own composite) to the pyramid update, which premultiplies as it shrinks. Verify with a unit test that a borrowed view gives the same levels as a rebuild.

## 6. Stage B — canvas presents from a level; navigator from the pyramid

- [x] 6.1 In `image_view.cpp`, choose the level per the design formula (level 0 above 50 %, coarser below), fetch the crop with `display_image`, blit it at the level origin, and remove the 64 MP cap and the full-resolution `drawImage(image_)` fallback from the repaint path; keep the document clip and checkerboard from full document dimensions. Verify present is pixel-identical to the transform draw at level 0 and is a single reduction for a coarse level, and that a pan does not resample the document.
- [x] 6.2 Keep `ImageView::image_` as a compatibility full-resolution buffer built on demand for `image()`'s consumers (`navigator_panel.cpp:259`, `histogram_panel.cpp:96`, `control_server.cpp:794`, `main.cpp:198`, `frame.cpp:380,851`, self-tests), and stop using it on the repaint path. Verify the scrollbar range checks and the `image()` consumer checks still pass.
- [x] 6.3 Draw the navigator thumbnail from a pyramid level instead of scaling a full copy. Verify with a check that the thumbnail updates on a content change and that no second full-resolution buffer is allocated.
- [x] 6.4 Migrate the present-cache checks 76 (`selftest.cpp`), 84, and 345 (`selftest_paint_perf.cpp`) to assert level-crop reuse and byte identity, and verify exit codes 75, 76, 82, 83, 84, 85, 344, 345, and 363 all pass after the present path moves to level crops.

## 7. Stage C — row-parallel CPU fallback

- [x] 7.1 Add `rayon` to `crates/pictura-render/Cargo.toml` and a `crates/pictura-render/src/composite_rows.rs` with a `Canvas` row accessor and row-scoped wrappers delegating to `blend_into`/`blend_parts`, reachable from both `composite.rs` and `composite_native.rs`. Verify the workspace builds and a unit test shows a parallel row equals the sequential row.
- [x] 7.2 Parallelize `composite_pixels`, `composite_canvas`, the 8-bit adjustment loop, and `gate_adjusted` (defined in `composite_native.rs`), keeping effect kernels and the knockout `cover` canvas sequential. Verify a `parallel == sequential` test on representative stacks including a knockout layer and a layer effect, and that `cpu_region_matches_full_slice` and the region-equality suite stay green.

## 8. Integration verification and dev notes

- [x] 8.1 Add ignored print-only `scroll_zoom_pan_profile_4000` and `scroll_zoom_pan_profile_16000` that time a `ViewPyramid` level crop against a local full-resolution rescale (a Rust proxy for the old present, not the canvas present path). Verify they run under `cargo test -p pictura-app -- --ignored --nocapture` and print the comparison.
- [x] 8.2 Extend the existing `selftest_canvas_view.cpp` with the C++ self-test behavioral checks (crop reused across pan, crop equals full at level 0, region refresh emits no `changed`) and gate any wall-clock under `PICTURA_REFERENCE_RUN`; the level-provider wiring lives in a new `frame_canvas.cpp`, registered in `CMakeLists.txt`. Verify `./build/pictura --headless --self-test` exits 0 and the SUMMARY counts match.
- [x] 8.3 Update `docs/dev/STATE.md`, `docs/dev/canvas-compositing-plan.md` (name this display pyramid distinctly from the M38 GPU-tiles/proxy-compositing track), and `docs/dev/canvas-view-spec.md`. Verify `TASK_ALLOWS_DOCS=1 bash scripts/guard.sh` passes and commit the docs separately with `TASK-ALLOWS-DOCS`.
- [x] 8.4 Run `bash scripts/verify-fast.sh` and `openspec validate --all --strict`. Verify both are green.
- [x] 8.5 Raise the raster-import default allocation cap from 512 MiB to 2 GiB in `crates/pictura-codec/src/probe.rs` and Qt's `QImageReader` allocation limit from 256 MB to 4 GB in `crates/pictura-app/cpp/decode_image.cpp`, so a 16000²-class RGBA raster (16507×16196×4 ≈ 1020 MiB) imports. Verify with probe tests that the 16000²-class header probes `Ok` and a 30 000² header still names the allocation limit, and by opening the World Map PNG through the control server.
