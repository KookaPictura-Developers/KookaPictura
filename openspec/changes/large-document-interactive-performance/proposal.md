# Proposal

## Why

The canvas-view work from #126 (PR #129) landed the view pyramid, the damage
account, and the regional refresh, but large documents are still slow to edit:
a paint stroke repaints the full-resolution frame on every dab, the 120 ms panel
timer calls `PictureView::image()` and rebuilds the whole display image, every
operation that is not a dab or a visibility toggle recomposites the entire
document, and the decode edge copies the decoded buffer byte-by-byte. On the
16000² class (267 Mpx) the per-operation cost stays in the seconds and the
resident image buffers total several gigabytes. Issue #130 tracks the remaining
large-document work; this change covers the interactive present, operation,
panel, ICC, and decode paths. Replacing the full-document history snapshots is
staged as its own change (`tiled-layer-history`), because it needs a tile and
content-versioning design the other items do not.

## What Changes

- **Present from the pyramid during a stroke.** `presentCrop` currently returns
  no crop while `isPainting()` is true, so every dab repaints by resampling the
  full-resolution `image_`. The regional path already patches the pyramid from
  the stroke's working document on every dab, so the canvas SHALL present the
  zoom level crop mid-stroke, exactly as it does when idle.
- **One full-resolution frame.** The app SHALL hold at most one full-resolution
  frame (the document's planar `composite`). The cached full-resolution `image`
  becomes a derived, on-demand product for `image()` consumers, and the cached
  full-resolution sRGB `level0` frame is no longer held alongside it; the display
  serves level-0 crops from the document composite through the cached ICC
  transform.
- **A single damage consumer.** The damage account is drained by one explicit
  take, not implicitly by `image()`; a regional refresh MUST NOT be followed by a
  full-document rebuild because a panel read `image()`.
- **Histogram from a pyramid level.** The Histogram panel computes its 256-bin
  distribution from a view-pyramid level (cost independent of document size)
  instead of cloning and scanning a full-resolution image.
- **Regional operations.** Pixel-changing operations that can bound their effect
  — filter or adjustment on a bounded layer, blend/opacity/fill on a bounded
  layer, clipboard clear/paste, paint commit and cancel, move and transform
  commit — SHALL refresh only their dirty rectangle through the existing region
  path, byte-identical to a full recomposite. Operations that cannot be bounded
  keep the full recomposite. Selection-only edits stop recompositing outright.
- **Decode without a per-byte copy.** The Qt decode edge SHALL copy the decoded
  RGBA rows in bulk instead of one byte at a time across the bridge, and SHALL
  enforce the allocation budget before any engine structure is allocated.
- **ICC transform reuse.** Repeated conversion of a document's pixels to its
  working/display space SHALL reuse the parsed profile and its transform, keyed
  on the document's working profile.
- **Out of scope.** Full-document history snapshots are unchanged here (the
  tiled/COW history is a separate change), and moving the decode and document
  construction off the GUI thread is deferred: it needs an asynchronous open
  path of its own.
- **BREAKING**: none. `PictureView::image()` still returns a full image on
  demand for the panels and self-tests that consume it, and the `regionBlitted` /
  `changed` signal contracts are preserved.

## Capabilities

### New Capabilities

- `compositing/described-damage`: the operation-side contract that a
  pixel-changing operation which can bound its effect refreshes only that
  rectangle, equal to a full recomposite, and that an unbounded operation keeps
  the full recomposite.

### Modified Capabilities

- `ui/document-canvas`: the canvas presents a level crop while a stroke is in
  progress; the app holds one full-resolution frame and derives `image()`
  on demand; the damage account is drained by one explicit take and a regional
  refresh never triggers a full-document rebuild.
- `ui/info-histogram-panel`: the histogram is computed from a view-pyramid level
  so its cost is independent of document size, and it does not read
  `PictureView::image()` on refresh.
- `interop/image-import`: the decode edge copies decoded rows in bulk and
  enforces the allocation budget before any engine structure is allocated.
- `color/color-profile-assignment`: repeated working-space conversion of a
  document's pixels reuses the parsed working profile and its transform.
- `tools/paint-engine`: releasing a stroke refreshes only the stroke's changed
  rectangle (matching a full recomposite) rather than recompositing the whole
  document.

## Impact

- `crates/pictura-app`: `cpp/image_view.cpp` (`presentCrop` painting guard),
  `cpp/panels/histogram_panel.cpp`, `cpp/decode_image.cpp`, `cpp/frame.cpp`;
  `src/state.rs` and `src/cxxqt_object/impl_core.rs` (`reset_pyramid`, `image`,
  `recompute`/`refresh_region`, damage take), `impl_filters.rs`,
  `impl_layers.rs`, `impl_paint.rs`, `impl_transform/*`, `impl_selection.rs`,
  `clipboard.rs`; `src/helpers_composite.rs`, `src/helpers.rs`.
- `crates/pictura-render`: `composite.rs` / `gpu/mod.rs` region compositing
  (already present) and the region bounds helpers; no new format.
- `crates/pictura-codec`: `src/icc.rs` (`buffer_to_srgb` transform reuse); the
  decode edge's bulk copy.
- `crates/pictura-color`: caches a parsed working profile / transform for
  repeated conversion where the app currently rebuilds it (`Profile`,
  `Transform`).
- Tests: region-vs-full equivalence for each newly bounded operation; a
  mid-stroke crop-vs-patched equality test; a histogram-cost check; a decode
  copy/budget check; a self-test check that a stroke on a large document leaves
  the canvas correct and that no per-dab path rebuilds the full image.
- Docs: `docs/dev/STATE.md` and `docs/dev/canvas-view-spec.md` if the milestone
  close-out requires it; `docs/` changes carry `TASK-ALLOWS-DOCS`.
- No new dependencies: the work uses `rayon`, `lcms2`, and Qt already present.
