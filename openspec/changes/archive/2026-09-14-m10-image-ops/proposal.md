## Why

`docs/04-image-ops/image-size.md` (`IMG-001`), `canvas-size.md` (`IMG-002`), and
`image-rotation-and-flip.md` (`IMG-003`) specify Image Size, Canvas Size, and
Image Rotation, but no code sits below them: there is no crate that resamples,
extends, or reorients a `PixelBuffer`. M10 introduces `pictura-ops`, a pure,
document-free buffer-geometry crate, so the later Image Size / Canvas Size /
Rotation integration has one reviewed resize, canvas, and orientation primitive
instead of every caller re-deriving the pixel math. `docs/dev/m10-image-ops.md`
freezes the M10 contract.

## What Changes

- Add a workspace crate `pictura-ops` for pure buffer geometry over
  `pictura-core::PixelBuffer`; it owns no document, layer, undo, or UI state.
- `image-resize`: `pictura_ops::resize(buf, width, height, Resample)` resamples
  every channel of the input to the target size with `Resample::Nearest`,
  `Bilinear`, or `Bicubic`, returning a new `PixelBuffer` and leaving the input
  untouched. `width`/`height` below 1 and malformed buffers return
  `OpsError::InvalidParams`; sampling clamps to the edge and 1×1 / 1-px inputs
  never panic.
- `canvas-operations`: `pictura_ops::resize_canvas(buf, width, height, Anchor,
  background)` grows the canvas around one of nine anchors, filling the new area
  with `background` (the 4th component is alpha), or shrinks it to the
  anchor-aligned rectangle, returning a new buffer.
- `image-orientation`: exact index remaps `rotate90_cw`, `rotate90_ccw`,
  `rotate180`, `flip_horizontal`, and `flip_vertical` that are pixel-exact and
  reversible; plus `rotate_arbitrary(buf, angle_deg, background)`, a bilinear
  inverse map onto an expanded bounding box with background corners, where a
  finite `angle_deg` in `-359.99..=359.99` is required or `OpsError::InvalidParams`
  is returned.
- Add a differential oracle (`scripts/ops_oracle.py` plus
  `crates/pictura-ops/tests/oracle.rs`): resize filters vs ImageMagick with the
  measured mapping and tolerance recorded; canvas growth/crop vs `-extent` with
  gravity; orientation exact for right angles/flips and measured for arbitrary
  angles.
- Out of scope (later): Bicubic Smoother / Sharper / Automatic; document and
  layer-tree integration and the Image Size / Canvas Size / Rotation dialogs;
  DPI / resolution math and physical units; CMYK / Lab; 16- and 32-bit; Smart
  Objects; Crop-tool / Ruler straighten tools.

## Capabilities

### New Capabilities

- `image-resize`: the pure `resize` resampler in `pictura-ops` — the
  `Resample::{Nearest, Bilinear, Bicubic}` kernels, per-channel resampling,
  clamp-to-edge sampling, the new-buffer / untouched-input contract, the
  `InvalidParams` size and buffer validation, tiny-image safety, and the
  measured ImageMagick differential classification.
- `canvas-operations`: the pure `resize_canvas` in `pictura-ops` — the nine
  `Anchor` placements, grow (background fill with alpha) and shrink
  (anchor-aligned crop) semantics, the new-buffer contract, dimension and buffer
  validation, and the ImageMagick `-extent` / gravity oracle.
- `image-orientation`: the pure orientation primitives in `pictura-ops` — the
  pixel-exact and reversible `rotate90_cw` / `rotate90_ccw` / `rotate180` /
  `flip_horizontal` / `flip_vertical` index remaps, and the bilinear-inverse-map
  `rotate_arbitrary` with an expanded bounding box, background corners, and the
  `-359.99..=359.99` finite-angle validation, plus the exact-for-right-angles and
  measured-for-arbitrary oracle classification.

### Modified Capabilities

None. The M6–M9 capabilities (`blur-filters`, `sharpen-filters`,
`noise-filters`, `other-filters`, `stylize-filters`, `pixelate-filters`,
`distort-filters`) and the `pictura-core` document types are untouched; this
change adds a new crate without changing their requirements.

## Impact

- New crate `crates/pictura-ops/` with `src/lib.rs` (the `Resample`, `Anchor`,
  and `OpsError` types plus the public re-exports) and the resize / canvas /
  orientation modules.
- `Cargo.toml` workspace members and `crates/pictura-ops/Cargo.toml`; the crate
  depends on `pictura-core` for `PixelBuffer`.
- `scripts/ops_oracle.py` and `crates/pictura-ops/tests/oracle.rs` plus
  `tests/README.md`: the resize filter-mapping row, the canvas gravity row, and
  the orientation exact/measured rows.
- Existing crates, `docs/`, and the M6–M9 changes are not modified. No app,
  document-model, or dialogs work is included (M10 is the primitive layer only).
- Dependencies: no new third-party crates; the oracle script uses the
  already-optional ImageMagick `magick` binary at test time.
- Follows `docs/04-image-ops/image-size.md` (`IMG-001`), `canvas-size.md`
  (`IMG-002`), `image-rotation-and-flip.md` (`IMG-003`), and
  `docs/dev/m10-image-ops.md`; those specs are not modified.
