# Proposal

## Why

The canvas hands Qt the full-document composite and lets Qt rescale it on every
paint. On a 16000² document at high zoom the capped present cache is disabled and
a full-resolution source is redrawn, the navigator rescales the whole composite,
and the CPU composite fallback is single-threaded. Issue #126 ports photorust's
canvas, workspace, and composition optimizations: crop the viewport from a view
pyramid's levels (premultiplied crops) instead of scaling the document, keep
partial repaint correct with an explicit damage account, and make the GPU-absent
composite fallback row-parallel.

## What Changes

- **View pyramid.** Add a halved-level structure in `pictura-render`: level 0 is
  the document composite (straight alpha, not copied); each level below is half
  the last and premultiplied, down to a 256 px long side, updated from damage and
  cropped (premultiplied) to a viewport rect.
- **Crop-from-level present.** The canvas picks the level closest above screen
  resolution and blits a premultiplied crop; the single capped scaled cache and
  the full-resolution `drawImage(image_)` fallback are removed, including the
  64 MP cap that disabled caching at high zoom.
- **Premultiplied display conversion.** A display-only Rust→Qt conversion emits
  `Format_RGBA8888_Premultiplied`; the shared straight-alpha conversions used by
  export encoding and thumbnails are unchanged. Sampling is smooth below 200 %
  and nearest-neighbour at and above it.
- **Canvas damage.** Add `CanvasDamage` (`edited` / `mark` / `take`) with the
  invariant that any undescribed change forces a whole-canvas redraw and a mark
  can never mask an earlier undescribed edit, replacing the `display_dirty` bool.
  The existing `regionBlitted` / `changed` signals and their "no `changed` on a
  region refresh" contract are preserved.
- **Navigator from the pyramid.** The navigator renders from a pyramid level
  instead of scaling a second full-resolution copy.
- **Row-parallel CPU fallback.** Parallelise the per-pixel CPU compositor loops
  by row chunks with `rayon`. The GPU compositor stays the preferred backend;
  this only accelerates the fallback (no adapter, unsupported stack, or a
  document past the GPU buffer limits).
- **Docs.** Update the canvas dev notes; `docs/` changes carry
  `TASK-ALLOWS-DOCS`.
- **BREAKING**: none. `PictureView::image()`, `ImageView::image_`, `regionBlitted`,
  and `changed` keep their current behavior.

## Capabilities

### New Capabilities

- `compositing/view-pyramid`: the halved-level structure (level 0 uncopied,
  levels below 0 premultiplied), its damage-driven update, and its viewport crop.

### Modified Capabilities

- `ui/document-canvas`: crop-from-level present replaces the capped scaled cache;
  the `CanvasDamage` invariant governs partial repaint; the bridge emits
  premultiplied pixels and the canvas selects the sampling filter by zoom.
- `ui/navigator-panel`: the thumbnail is drawn from a view-pyramid level rather
  than a fresh full-composite reduction.
- `compositing/gpu-compositing`: the CPU fallback composites row-parallel.

## Impact

- `crates/pictura-render`: new `src/view_pyramid.rs` and `src/composite_rows.rs`;
  `Cargo.toml` adds `rayon`.
- `crates/pictura-app`: bridge in `src/cxxqt_object.rs` (`display_image`,
  `display_level_count`, `display_level_size`, `take_canvas_damage`); state and
  call sites in `state.rs`, `impl_core.rs`, `impl_history.rs`,
  `impl_transform/*`; `helpers_composite.rs` display conversion (shared
  straight-alpha helpers unchanged); `cpp/image_view.{h,cpp}`,
  `cpp/panels/navigator_panel.cpp`; a new
  `selftest_*.cpp`; `CMakeLists.txt` registration for any new `.cpp`.
- Docs: `docs/dev/STATE.md`, `docs/dev/canvas-compositing-plan.md`,
  `docs/dev/canvas-view-spec.md`.
- Dependency: `rayon` (new direct dependency; justified by the CPU fallback for
  documents past the GPU buffer limits and for unsupported stacks).
- Raster-import budget: `pictura_codec::ImageBudget`'s default allocation cap is
  raised from 512 MiB to 2 GiB (`crates/pictura-codec/src/probe.rs`), and Qt's
  decode edge raises `QImageReader`'s 256 MB allocation limit to 4 GB
  (`crates/pictura-app/cpp/decode_image.cpp`), so a 16000²-class RGBA raster
  (16507×16196×4 ≈ 1020 MiB) can be opened at all; the dimension cap stays
  30 000. This is the import-side prerequisite for the 16000² target.
- Memory at 16000²: the pyramid stores no level 0 (it borrows a per-call view) and
  the navigator adds no second full-resolution copy; only the halved levels are
  added (~⅓ of the document). The display path does cache one full-resolution sRGB
  `level0` frame for the pyramid and display, so the common RGBA path adds one
  full-resolution buffer over `doc.composite`.
