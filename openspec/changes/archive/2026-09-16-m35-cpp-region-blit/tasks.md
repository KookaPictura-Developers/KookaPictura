## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m35-cpp-region-blit.md` milestone brief (the measured
  blit evidence, the planar-canvas design, the frozen signal/blit interfaces, the
  non-goals)
- [x] 1.2 Write `proposal.md`, `design.md`, and `tasks.md`
- [x] 1.3 Freeze in `design.md`: the `regionBlitted(QImage, i32 x, i32 y)` qsignal
  and its cxx-qt declaration; `ImageView::blitRegion(const QImage&, int, int)` with
  `CompositionMode_Source` and present-cache invalidation; the `display_dirty`
  lifecycle and every rebuild site; the `image()`/`sample_argb`/`move_preview_base`
  changes; removal of `REGION_REFRESH_BUDGET` and its fallback; the frame
  connection and panel-refresh throttling; explicit non-goals (resident sources,
  tiles, LoD, COW history, zero-copy present, no compositor/parity change)
- [x] 1.4 Update `docs/dev/STATE.md` "Next" and
  `docs/dev/canvas-compositing-plan.md` deferred list (M35–M38 staging)

## 2. Rust: composite owner + `display_dirty` + `image()`/`sample_argb`

- [x] 2.1 Add `display_dirty: bool` to `PictureViewRust` (default `false`) and
  clear it in `recomposite`, `open`, `new_document`, `undo`, and `redo`; leave
  `doc.composite` as the only authoritative full-document composite
- [x] 2.2 `image()`: return the cached `rust.image` when `!display_dirty`; otherwise
  rebuild from `doc.composite` (or the stroke's working document while painting)
  with `buffer_to_image`, clear `display_dirty`, and return it; change the bridge
  declaration to `fn image(self: Pin<&mut Self>) -> QImage`
- [x] 2.3 `sample_argb(x, y)`: read the planar `doc.composite` directly (1-plane
  grey/opaque, 2-plane grey+alpha, 3-plane RGB opaque, 4-plane RGBA), bounds-check
  against the composite dimensions, and never build a full image
- [x] 2.4 Unit tests: `image()` returns a freshly rebuilt image byte-identical to
  a full recomposite after a region patch; `sample_argb` matches
  `buffer_to_image(&doc.composite)` at sampled pixels for 1/2/3/4-plane composites
- [x] 2.5 `cargo test -p pictura_app`; `cargo fmt`/`clippy` clean

## 3. Rust: `regionBlitted` signal + `refresh_region` + budget removal

- [x] 3.1 Add the `#[qsignal] #[cxx_name = "regionBlitted"] fn region_blitted(
  self: Pin<&mut Self>, region: QImage, x: i32, y: i32)` declaration; if the
  by-value `QImage` parameter does not compile through cxx-qt, fall back to
  `&QImage` (`const QImage&` on the C++ side)
- [x] 3.2 `refresh_region`: keep clamp + `composite_region_active`; on the
  non-painting path patch `doc.composite` with `patch_composite_region`; convert
  only the region with `buffer_to_image`; set `display_dirty = true`; emit
  `region_blitted(region, x0, y0)`. Do NOT emit `changed` and do NOT touch
  `rust.image`
- [x] 3.3 Delete `REGION_REFRESH_BUDGET`, the "too big" full-recomposite fallback,
  the stroke `document_to_image` fallback, and `blit_image_region`; drop the budget
  comparison from `move_preview_region` (keep only the `cached_ok` guard, so a rect
  larger than the old 1 MP budget now takes the region path)
- [x] 3.4 `begin_move_preview`: build `move_base` from the authoritative
  `doc.composite` (clone the planar buffer, overwrite the hidden-layer region with
  a plane `copy_from_slice`, convert once with `buffer_to_image`), never from the
  stale `rust.image`
- [x] 3.5 Unit tests: a region-refreshed `doc.composite`/`image()` equals a full
  recomposite after a sequence of Move/paint/visibility refreshes; a region larger
  than the old 1 MP budget takes the region path (no full recomposite) and still
  equals a full recomposite; `begin_move_preview`'s base equals a full composite
  with the layer hidden (existing M32 test)
- [x] 3.6 `cargo test --workspace`; `cargo fmt`/`clippy` clean

## 4. C++: `ImageView::blitRegion` + frame connection

- [x] 4.1 Add `ImageView::blitRegion(const QImage& region, int x, int y)`:
  `QPainter` on `image_` with `setCompositionMode(CompositionMode_Source)`,
  `drawImage(QPoint(x, y), region)`, clipped to the document rect; set
  `presentCache_.valid = false`; `update()`
- [x] 4.2 In `frame.cpp::addDocument`, connect `PictureView::regionBlitted` to a
  lambda for that document's `canvas` that calls `canvas->blitRegion(...)`, then
  `panelRefreshTimer_->start()`, `updateTabTitle`, `updateWindowTitle`, and
  `registry_->refresh()` — and never calls `view->image()`, `replaceImage`, or
  `refreshPanels`
- [x] 4.3 `cmake --build build`; confirm no `set_pixel_color` per-pixel loop
  remains in `cxxqt_object.rs` and no `REGION_REFRESH_BUDGET` reference remains

## 5. Tests + self-test + evidence

- [x] 5.1 Add a `main.cpp` self-test step that drives a region refresh, observes
  `canvas->image()` equal to `view->image()` after a forced full recomposite, and
  observes the present cache invalidated then rebuilt on the next paint
- [x] 5.2 Re-point the M31 `m31_region_large` check at the region-blit path: it now
  asserts `regionBlitted` fired and `changed` did not (the region path ran) **and**
  the on-screen canvas equals a full recomposite — strictly stronger than the old
  fallback check, not weakened.
  `move_preview_region_guards_empty_and_missing_composite` now asserts a rect larger
  than the old 1 MP budget takes the region path. The M31/M32/M34 image-equality
  checks still pass.
- [x] 5.3 `#[ignore]` timing evidence: a 512² and a 1024² region refresh on a
  4000² document, comparing the old per-pixel blit (7.36/27.3 ms) with the C++
  blit; state the release/GPU conditions and skip with a printed note when no
  adapter exists (the `m35_region_refresh_profile_4000` test)
- [x] 5.4 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` on the
  GPU default and the CPU fallback (and the fixture variant)
- [x] 5.5 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D
  warnings`; `cargo test --workspace`

## 6. Close-out

- [x] 6.1 `openspec validate m35-cpp-region-blit --strict`;
  `openspec validate --all --strict`
- [x] 6.2 Update `docs/dev/STATE.md` with the M35 result
- [ ] 6.3 Archive the change (`openspec archive m35-cpp-region-blit`) and commit
  with the `TASK-ALLOWS-DOCS` marker where docs are touched — **deferred**: this
  close-out is docs-only and explicitly does not commit; archive/commit is left to
  a separate requested step.
