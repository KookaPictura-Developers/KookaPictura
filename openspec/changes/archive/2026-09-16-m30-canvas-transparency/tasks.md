## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m30-canvas-transparency.md` milestone brief
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze in `design.md`: a screen-space 8 px two-tone (`#FFFFFF`/`#CCCCCC`)
  checkerboard anchored to the document origin via a cached 2×2 tile and a brush
  origin, clipped to the document rect; document-rect clipping of the composited
  image, the Move-tool preview, and the overlay; unchanged canvas colour outside;
  explicit non-goals (Transparency & Gamut preferences, `View > Show >
  Transparency Grid`, gamut warning, GPU/RHI canvas)
- [x] 1.4 Commit brief + OpenSpec artifacts with a `TASK-ALLOWS-DOCS` message

## 2. ImageView checkerboard + clipping

- [x] 2.1 Add `static int transparencyCellSize()` (8), `transparencyColorA()`
  (`#FFFFFF`), `transparencyColorB()` (`#CCCCCC`) to `ImageView`
- [x] 2.2 Add the lazily-built `2 * cell` square `transparencyTile()` with the
  opposite two cells in the second tone
- [x] 2.3 `paintEvent`: fill the document rect ∩ widget with the tile at
  `setBrushOrigin(docRect.topLeft())`, so the checkerboard is clipped to the
  document rect and anchored to the document origin
- [x] 2.4 `paintEvent`: after `translate`/`scale`, `setClipRect(QRectF(0, 0,
  image_.width(), image_.height()))` so the image, preview, and overlay are
  cropped to the document
- [x] 2.5 `cmake --build build`; the canvas still pans/zooms and the preview
  still tracks the drag

## 3. Self-test

- [x] 3.1 Add a `--self-test` block that opens a 64×64 transparent document and
  renders the canvas
- [x] 3.2 Assert both checker tones appear inside the document rect, the canvas
  colour does not, and the pixels outside the document rect are the canvas colour
- [x] 3.3 Start a Move preview with a red layer at `(-32, -32)`; assert the
  covered quadrant is red and the pixels outside the document rect stay the
  canvas colour
- [x] 3.4 Exit 74 with `FAIL: M30 transparency/clipping wrong` on failure;
  report `m30_canvas checker=1 clipped=1` on success
- [x] 3.5 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` passes

## 4. Close-out

- [x] 4.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo test --workspace`
- [x] 4.2 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` (and the
  fixture variant)
- [x] 4.3 `openspec validate m30-canvas-transparency --strict`;
  `openspec validate --all --strict`
- [x] 4.4 `bash scripts/guard.sh`
- [x] 4.5 Update `docs/dev/STATE.md` with the M30 result
- [x] 4.6 Archive the change (`openspec archive m30-canvas-transparency`) and commit
