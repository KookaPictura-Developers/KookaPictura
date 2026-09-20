## 1. Batch 1 — Layer reparent correctness

- [x] 1.1 Fixed `move_path_to_dest`/`move_path_to` in
  `crates/pictura-render/src/document_ops/layer_ops/properties.rs`: `dest_parent`
  stays pre-removal, is validated with a new pre-removal `container_of`
  (`paths.rs`), and is converted once to post-removal coordinates
  (`to_post_removal`) before `container_of_mut`. `container_exists_after_removal`
  deleted.
- [x] 1.2 Rust tests: `[A,G]` A-into-G; `[A,G,B]` A-into-G with B staying at
  root; sibling-container depth shift; out-of-group above/below siblings;
  same-parent reorder; every existing refusal unchanged.
- [x] 1.3 C++ self-tests `lpr_drag_into_above` (392) and `lpr_drag_out` (393)
  through the panel pipeline.

## 2. Batch 2 — Active-layer cursor/refusal

- [x] 2.1 `topmostPixelLocked` → `activePixelLocked` in
  `tools_marquee.cpp`; the brush branch and the `tool_brush.cpp` refusal message
  now resolve the active layer's lock/visibility. Checks 395/396/397.
- [x] 2.2 Self-tests: locked active non-topmost refuses with Forbidden; locked
  topmost + editable lower active paints with a blank cursor; panel
  multi-selection/no-selection refuses.

## 3. Batch 3 — Panel width/mode persistence

- [x] 3.1 `persistedWidth()` writes the remembered `normalWidthBeforeIconic_`
  while a normal flip is pending; the legacy rail width uses `persistedWidth()`
  (`panel_column.h`, `panel_column_iconic.cpp`, `panel_column.cpp`,
  `frame_session.cpp`).
- [x] 3.2 Regression `lpr_iconic_flip_width` (399): primary iconic→normal
  persists the remembered width; mixed modes round-trip.

## 4. Batch 4 — Paint responsiveness

- [x] 4.1 In-stroke backend keeps the faster path. The new 4000² profile
  measured GPU ~1.4 ms/dab vs CPU ~11 ms/dab, so forcing the CPU was a measured
  regression and is not done; recorded in the profile doc comment.
- [x] 4.2 Per-dab GUI fan-out coalesced: while a stroke is active the paint
  `regionBlitted` keeps the canvas blit, panel timer, and cursor, but skips
  `registry_->refresh()`, `updateTabTitle`, and `updateWindowTitle`; they run
  once on commit (`frame.cpp`). Check `pp_no_registry_refresh` (394).
- [x] 4.3 `blitRegion` (`image_view.cpp`) patches the full-resolution canvas with
  direct row writes (owning the buffer once) instead of a detaching `QPainter`;
  the present-cache patch and `valid=false` fallback are unchanged.
- [x] 4.4 `#[ignore]`d `paint_dab_profile_4000` prints per-dab CPU vs GPU
  composite + blit timings; the per-dab no-registry-refresh shape check passes.

## 5. Batch 5 — Lock coverage

- [x] 5.1 `llk_transparency_180` (398): with a bridge-set transparency lock, RGB
  changes while alpha stays exactly 180 and an `A=0` pixel is untouched.
  Content-move's transparency refusal is documented as a divergence.

## 6. Verification and gates

- [x] 6.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings` — clean.
- [x] 6.2 `cargo nextest run --workspace` — 1267 passed, 10 skipped; doctests 1
  skipped.
- [x] 6.3 `cmake --build build --parallel`; `./build/pictura --headless
  --self-test` — **334 passed, 0 failed, 0 skipped**; codes 392-399, none reused.
- [x] 6.4 `bash scripts/verify-full.sh` — **TOTAL 1637 passed, 11 skipped,
  0 failed**; `verify-full: OK`.
- [x] 6.5 `openspec validate app-ui-round3-fixes --strict` and
  `openspec validate --all --strict` — valid.
- [x] 6.6 No `docs/` change.

## 7. Explicitly not done / ceilings

- [x] 7.1 Artboard/frame nesting lock — no artboards/frames exist and the Move
  tool never reparents, so there is nothing to gate.
- [x] 7.2 No stroke-engine rewrite (tiles, threading, SIMD) or GPU-resident
  stroke buffers.
- [x] 7.3 No new dependency, no document-format change, no `docs/` edit.
