## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m32-interactive-canvas.md` milestone brief (measured
  phase table, per-item design, frozen interfaces, explicit non-goals)
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze in `design.md`: the move-preview region base (no full composite,
  `doc.composite`/cached image untouched), `influence_rect` for the visibility
  toggle with its full-recomposite fallback, the zoom cache, and the byte-identity
  arguments
- [x] 1.4 Correct the M32 premise in `docs/dev/canvas-compositing-plan.md` (§2
  measured table, §3 M32–M35 ordering) and `docs/dev/STATE.md` (Next section)

## 2. App region completion (move-preview, visibility)

- [x] 2.1 `begin_move_preview`: build the base by region-compositing only the
  topmost pixel layer's clamped rect with the layer hidden into the cached canvas
  (do not write `doc.composite` or the cached image); restore `visible = true`
- [x] 2.2 `set_layer_visible`: flip the flag, record, then
  `refresh_region(influence_rect(layer))` (raster `rect`; masked adjustment
  `mask.rect`) when that is provably equivalent, otherwise `recomposite()`
- [x] 2.3 `cargo test --workspace`; `cmake --build build`; move and visibility
  still correct

## 3. Present zoom cache

- [x] 3.1 Add `presentCache_` + validity/zoom state to `ImageView`, invalidated by
  `setImage`, `replaceImage`, `setZoom`, `fitOnScreen`, `actualPixels`, and
  `applyInitialView`
- [x] 3.2 Build the cache lazily in `paintEvent` through the same transform and
  render hints as the direct draw; present it un-scaled at `offset_`
- [x] 3.3 Skip caching (direct transform draw, visually identical) when the scaled
  footprint exceeds the fixed bound
- [x] 3.4 `cmake --build build`; a pan repaint does not resample the document and
  the rendered image equals the transform draw

## 4. Self-test + evidence

- [x] 4.1 Self-test: after `begin_move_preview` the base equals a full composite
  with the layer hidden, and the document/canvas are unchanged
- [x] 4.2 Self-test: toggling a raster layer's visibility refreshes only its rect
  and equals a full recomposite; an unboundable toggle falls back and equals a
  full recomposite
- [x] 4.3 Self-test: a pan repaint uses the zoom cache and matches the transform
  draw
- [x] 4.4 Check the measured phases still hold (a region move is materially
  cheaper than a full composite; skip with a printed note when no adapter exists)
- [x] 4.5 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` passes
  on the default GPU or CPU fallback (and the fixture variant)

## 5. Close-out

- [x] 5.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D
  warnings`; `cargo test --workspace`
- [x] 5.2 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` (both
  variants)
- [x] 5.3 `openspec validate m32-interactive-canvas --strict`;
  `openspec validate --all --strict`
- [x] 5.4 `bash scripts/guard.sh` (docs carry the `TASK-ALLOWS-DOCS` marker)
- [x] 5.5 Update `docs/dev/STATE.md` with the M32 result
- [ ] 5.6 Archive the change (`openspec archive m32-interactive-canvas`) and
  commit — deferred: archive/commit is the post-close-out step; not committed here
