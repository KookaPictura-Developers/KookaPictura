## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m34-composite-coherence.md` milestone brief (the two
  problems, the four requirements, the channel-consistent
  `store_composite`/region-patch design, the `record`-ordering change and its
  history-semantics argument, the non-goals)
- [x] 1.2 Write `proposal.md`, `design.md`, and `tasks.md`
- [x] 1.3 Freeze in `design.md`: `store_composite(doc, &PixelBuffer)` — RGB
  stores the rendered 4-plane RGBA frame, a non-RGB mode preserves its composite
  colour-plane count, a dimension change replaces; `patch_composite_region` made
  channel-consistent; `recomposite` render → store → `QImage`; `refresh_region`
  stores on the region path (not mid-stroke); `save` unchanged; the
  unconditional `undo`/`redo` fast path; the 18 `record` sites that move after
  their composite step plus `Open`/`New`; explicit non-goals (COW/tile-diff
  history, resident GPU sources, zero-copy present, 256² tiles, no `write_psd`
  change)
- [x] 1.4 Update `docs/dev/STATE.md` "Next" and
  `docs/dev/canvas-compositing-plan.md` deferred list (M34 re-scoped to
  composite coherence + cheap undo/redo; resident sources / zero-copy present /
  tiles / COW history each need their own design)

## 2. `store_composite` + channel-consistent region patch + `recomposite`

- [x] 2.1 Add `store_composite(doc: &mut Document, rendered: &PixelBuffer)`:
  return on a render/document size mismatch; normalise the render to a 4-plane
  RGBA frame; RGB (or a dimension mismatch) replaces `doc.composite` with that
  frame; otherwise preserve `doc.composite.channels` and copy
  `min(rendered, composite)` planes
- [x] 2.2 Make `patch_composite_region` channel-consistent: require a 4-plane
  region, write `min(4, doc.composite.channels)` planes at the composite's own
  count, one `copy_from_slice` per row per plane; keep the size/bounds guards
  and the outside-region-leaves-previous-values behaviour
- [x] 2.3 `recomposite`: compute `current_buffer(doc, gpu_compute)` once,
  `store_composite(doc, &rendered)`, build `rust.image =
  buffer_to_image(&rendered)`, emit `changed`; a layerless RGB document
  normalises its embedded composite to RGBA
- [x] 2.4 `refresh_region`: on the non-painting region path, after the display
  blit call the channel-consistent `patch_composite_region`; keep the
  over-budget `recomposite` fallback and the mid-stroke no-patch behaviour
- [x] 2.5 Unit tests: `store_composite` stores 4 planes for RGB and keeps a
  1-plane Grayscale composite; a region patch preserves the count and leaves
  outside pixels untouched; the undo/redo display path equals a full recomposite
  after a sequence of ops including a `refresh_region`-based one
- [x] 2.6 `cargo test -p pictura_app`; `cargo fmt`/`clippy` clean

## 3. Save

- [x] 3.1 Confirm `save` needs no mechanism change (it serializes
  `doc.composite`, kept current by section 2) and that `write_psd`'s bytes and
  channel-count rule are untouched
- [x] 3.2 Test: apply an edit, `save` to a temp path, `read_psd` back, and
  assert the composite equals the edited render (not the pre-edit one), at RGB

## 4. `undo`/`redo`

- [x] 4.1 `undo`/`redo`: restore `doc`/`selection` from the snapshot; restore
  `image = buffer_to_image(&snapshot.doc.composite)` unconditionally (no
  channel- or layer-count fallback); emit `changed`
- [x] 4.2 Test: after a sequence of ops (including a region refresh) each
  undo/redo display equals `document_to_image` of the restored document; at the
  stack ends undo/redo are no-ops
- [x] 4.3 Confirm `history_jump`/`history_restore_snapshot` remain correct on
  `recomposite`

## 5. `record` ordering + Open/New

- [x] 5.1 Move `record(...)` to after its `recomposite`/`refresh_region` at all
  18 sites (the table in `docs/dev/m34-composite-coherence.md`); leave
  `select_all`/`deselect`/`magic_wand` unchanged
- [x] 5.2 `open` and `new_document`: render `current_buffer(doc, gpu_compute)`
  and `store_composite` before `history.capture`, so the first snapshot is
  current
- [x] 5.3 Update `record`'s doc comment to state it expects a current composite;
  confirm labels, stack order, depth bound, and redo truncation are unchanged
- [x] 5.4 Test: after each mutating command the captured snapshot's composite
  equals a full recomposite of the captured document

## 6. Tests + self-test + evidence

- [x] 6.1 Extend `crates/pictura-app/cpp/main.cpp` with a composite-coherence
  probe, an undo/redo round-trip, and a save-after-edit pixel round-trip (new
  exit codes 79/80/81)
- [x] 6.2 Confirm the existing M14/M17 self-test checks (undo/redo image
  equality, open resets history, save round-trip) still pass; name any assertion
  corrected rather than weakened
- [x] 6.3 `cargo test --workspace`; `cargo fmt --all`; `cargo clippy
  --workspace --all-targets -- -D warnings`
- [x] 6.4 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` on
  the GPU default and the CPU fallback
- [x] 6.5 `#[ignore]` timing evidence: undo on a 4000² 4-plane edit is
  materially cheaper than a full composite. Measured (release, GPU):
  `document_to_image(gpu=true)` **163.16 ms** vs
  `buffer_to_image(snapshot.composite)` **40.94 ms** (~4×), with the
  `History::capture` whole-document clone at **59.49 ms** (remaining, deferred);
  test `m34_undo_profile_4000`

## 7. Close-out

- [x] 7.1 `openspec validate m34-composite-coherence --strict`;
  `openspec validate --all --strict`
- [x] 7.2 Update `docs/dev/STATE.md` with the M34 result
- [ ] 7.3 Archive the change (`openspec archive m34-composite-coherence`) and
  commit with the `TASK-ALLOWS-DOCS` marker where docs are touched — deferred to
  the orchestrator on request
