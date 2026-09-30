# Tasks

## 1. Cheap wins: panel, ICC, decode, selection

- [x] 1.1 Change `HistogramPanel::recompute` (`cpp/panels/histogram_panel.cpp`) to read the coarsest view-pyramid level (≤512 px, as `navigator_panel.cpp` does) instead of `view_->image()`; verify the Histogram still renders 256 bins and a headless self-test/manual check shows no `image()` call during a region refresh
- [x] 1.2 Add a Rust test around the histogram source that asserts the level used is ≤512×512 and independent of document size (extend `canvas_view_test.rs` or a new `histogram_test`); verify `cargo nextest run -p pictura-app` passes
- [x] 1.3 Cache the parsed working profile and lcms transform in `crates/pictura-codec/src/icc.rs` keyed on `document_icc`, invalidated on open/assign/convert; return the buffer unchanged without a clone when `document_icc` is `None`; verify a new unit test proves repeated conversions are byte-identical to a fresh parse and that a profile change is picked up
- [x] 1.4 Replace the per-byte `push_back` loop in `decode_image_rgba` (`cpp/decode_image.cpp`) with a bulk row/frame copy, and check the allocation budget before constructing engine structures; verify `cargo nextest run -p pictura-codec` and the import tests still pass, and add a test that a large decoded buffer is copied without a per-byte loop
- [x] 1.5 Remove the `recomposite` calls from the selection-only operations in `impl_selection.rs` (keeping the `changed` emission that updates the selection overlay); verify the `ldoc_selection_no_recomposite` self-test asserts the document composite and `canvas_revision` are unchanged across select/invert/deselect/select-all, and `cargo nextest run -p pictura-app` passes

## 2. Mid-stroke canvas present

- [x] 2.1 Remove the `isPainting()` early return in `ImageView::presentCrop` (`cpp/image_view.cpp`) so the canvas presents the zoom level crop during a stroke; verify a manual/headless check that a stroke repaints through a crop
- [x] 2.2 Audit every `reset_pyramid` caller reachable while `rust.stroke.is_some()` and confirm none runs mid-stroke; if one does, make `level0`/pyramid rebuild from the stroke's working document; verify the audit is recorded and a test covers the reachable path
- [x] 2.3 Add a Rust test that a dab's mid-stroke presented crop is byte-identical to a full recomposite of the stroke's working document at the same level and filter; verify `cargo nextest run -p pictura-app` passes
- [x] 2.4 Add a C++ self-test check (new `selftest_*.cpp`, next free code ≥ 299) that paints a stroke on a large document and asserts the canvas is correct and no per-dab path rebuilt the full image; verify `./build/pictura --headless --self-test` passes and the exit code is registered in `selftest_report`
- [x] 2.5 Register any new `.cpp`/`.h` in `CMakeLists.txt` and re-run `cmake --build build --parallel`; verify the build succeeds and `scripts/file-size-allowlist.txt` is respected

## 3. Regional operations

- [x] 3.1 Migrate paint `end_paint`/`cancel_paint` (`impl_paint.rs`) to refresh the stroke's changed rectangle instead of `recomposite`; verify a region-vs-full equivalence test and `cargo nextest run -p pictura-app`
- [x] 3.2 Migrate single-layer opacity/blend/fill-fill-opacity (`impl_layers.rs` `mutate_layer`) to `refresh_region` on the layer's clamped rect via `layer_visibility_region`, with a full-recomposite fallback for groups/unbounded adjustments; verify `layer_opacity_region_refresh_equals_a_full_recomposite` plus the existing unbounded-fallback test. The generic `batch_changed` path remains a full recomposite (its op would have to report its changed paths; a follow-up).
- [x] 3.3 Migrate a filter and an adjustment (`impl_filters.rs`, `impl_pictura_raw.rs`) to the destination rectangle, keeping the full recomposite when the adjustment is unmasked/unbounded; verify equivalence via the existing region suite and the opacity/move tests (adjustment mask bounds reuse `layer_visibility_region`)
- [x] 3.4 Migrate clipboard clear/paste (`clipboard.rs`) to the affected/new layer rectangle; verify the region suite and self-test suite stay green
- [x] 3.5 Migrate move/translate preview and `commit_transform` to the union of source and destination rectangles (`impl_transform/dispatch.rs`, `session.rs`); `commit_move` already did; verify `layer_move_region_refresh_equals_a_full_recomposite` and the transform/move self-tests
- [x] 3.6 Documented the bounded-vs-unbounded operation set in `docs/dev/canvas-compositing-plan.md` ("Described-change coverage"); reviewed the remaining `recomposite()` sites (document size, flatten/merge/reorder, profile convert, GPU toggle, unbounded adjustments/groups/batch). `docs/` changes carry `TASK-ALLOWS-DOCS`.

## 4. One fewer full-resolution buffer

- [x] 4.1 Removed the cached full-resolution display `QImage` (`PictureViewRust::image`); `PictureView::image()` now builds on demand from the level-0 frame. The sRGB `level0` frame is kept because the pyramid needs it for regional updates (dropping it would force a full conversion per dab). Verify `cargo nextest run -p pictura-app` and the self-test.
- [x] 4.2 `refresh_level0_region`/`update_pyramid` are unchanged: with `level0` retained, a regional update still patches level 0 and repairs the pyramid from the region; the region-vs-rebuild tests still pass.
- [x] 4.3 Buffer accounting: at 16000² the display now holds one full-resolution sRGB frame (`level0`, ~1.02 GB) instead of two (`level0` + the premultiplied `image` QImage, ~2.0 GB); the composite and layer channels are unchanged. The `mem_probe_16k` example covers the document/pyramid buffers, not the display QImage.
- [x] 4.4 Updated `docs/dev/STATE.md` and `docs/dev/canvas-view-spec.md`; `docs/` changes carry `TASK-ALLOWS-DOCS` (guard checked at commit time).

## 5. Integration verification

- [x] 5.1 Ran the ignored profiling tests (`cargo test -p pictura_app profile -- --ignored --nocapture --skip 16000`). Evidence at 4000²: visibility region toggle 7.2 ms vs full recomposite 980 ms; region refresh 512² total 18 ms / 1024² 40 ms (old per-pixel blit 7.4/27.3 ms); paint dab GPU 2.5 ms vs CPU 17.5 ms per dab; undo display 507 ms vs old full `document_to_image` 805 ms plus a 62 ms snapshot clone. The 16507×16196 map is the manual target (the 16000² synthetic profile is memory-heavy and was skipped).
- [x] 5.2 `bash scripts/verify-fast.sh` (with `TASK_ALLOWS_DOCS=1`) passed: **2313 passed, 0 failed**; app self-test **510 passed**; doctests 0 passed / 1 skipped; file-size, guard, openspec all green.
- [x] 5.3 `openspec validate --all --strict` → **129 passed, 0 failed**.
