## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m31-region-compositing.md` milestone brief
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze in `design.md`: `composite_region_active(doc, rect: PsdRect,
  gpu_enabled) -> (PixelBuffer, Backend)` byte-identical to the slice of
  `composite_active`; region GPU dispatch (region origin/width in the uniform,
  region-sized source/mask, region-only readback); CPU loop restricted to the
  region; `PictureView` cached canvas with `refresh_region`; Move `old ∪ new`,
  paint dab bbox; unchanged oracles and ±1 LSB parity; explicit non-goals
  (LoD/proxies, GPU-resident present, 256² tiles, off-GUI-thread compute)
- [x] 1.4 Commit brief + OpenSpec artifacts with a `TASK-ALLOWS-DOCS` message

## 2. Render region compositing API + GPU region dispatch + parity

- [x] 2.1 Add `composite_region_active(doc, rect, gpu_enabled)` in
  `pictura-render`: clamp the rect to the document, composite only it, return a
  rect-sized `PixelBuffer` with the active `Backend`; an empty intersection
  returns a zero-dimension buffer without panicking
- [x] 2.2 CPU region loop: run the per-pixel loop over the clamped region
  (document coordinates preserved), shared across pixel, adjustment, and group
  passes; keep `composite_rgba`/`composite_active` as the oracle
- [x] 2.3 `gpu.rs`: region canvas (`region_w * region_h` packed words),
  `region_x0`/`region_y0`/`region_w` in the uniform, `sample_src` maps
  region-local to document coordinates; `build_source`/`build_mask` clamp the
  layer/group rect to the region and size planes to the region; group inner
  canvases are region-sized
- [x] 2.4 `gpu.rs`: dispatch `count = region_w * region_h` (2-D grid), read back
  only the region, de-interleave to a region-sized buffer
- [x] 2.5 Byte-identity tests: a full-rect region equals `composite_active`; a
  sub-rect equals the corresponding slice for separable, non-separable,
  adjustment, group and masked scenes; out-of-bounds/empty regions clamp without
  panicking
- [x] 2.6 `cargo test -p pictura-render`; clippy clean

## 3. App cached canvas + dirty-rect move/paint

- [x] 3.1 Add `refresh_region(rect)` on `PictureView`: clamp, composite the
  region through the active backend, patch the source `composite` and the cached
  `image` at the rect origin, emit `changed`; empty rect is a no-op
- [x] 3.2 `commit_move`: capture the topmost pixel layer rect before/after the
  shift, then `refresh_region(old ∪ new)` instead of the full composite; keep
  `translate_layer`/`translate_layer_active` as the oracle; history unchanged
- [x] 3.3 `paint_dab`: on a changed sample, `refresh_region` the dab's dirty
  rectangle against the stroke's working document; only the dab's bounding box
  updates
- [x] 3.4 Expose the last sample's dirty document rect from `Stroke`
  (`pictura-paint`) so a dab reports its own box
- [x] 3.5 Confirm all non-Move/non-paint mutations still full-recomposite and no
  other app behaviour changes
- [x] 3.6 `cargo test --workspace`; `cmake --build build`; painting and moving
  still correct

## 4. Self-test + timing evidence

- [x] 4.1 Add a self-test that moves and paints, region-refreshes, and asserts the
  cached canvas equals a full recomposite
- [x] 4.2 Add a timing check that a small dirty rect on a large document is
  materially cheaper than the full composite/readback; skip with a printed note
  when no adapter exists
- [x] 4.3 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` passes
  on default GPU or CPU fallback

## 5. Close-out

- [x] 5.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D
  warnings`; `cargo test --workspace`
- [x] 5.2 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` (and
  the fixture variant)
- [x] 5.3 `openspec validate m31-region-compositing --strict`;
  `openspec validate --all --strict`
- [x] 5.4 `bash scripts/guard.sh` (docs carry the `TASK-ALLOWS-DOCS` marker)
- [x] 5.5 Update `docs/dev/STATE.md` with the M31 result
- [x] 5.6 Archive the change (`openspec archive m31-region-compositing`) and
  commit
