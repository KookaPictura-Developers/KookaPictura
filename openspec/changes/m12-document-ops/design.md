## Context

M10 landed three buffer-level capabilities in `pictura-ops`: `resize` (Nearest /
Bilinear / Bicubic over a planar `PixelBuffer`), `resize_canvas` (nine-anchor
grow/shrink with a fill), and the orientation remaps (`rotate90_cw`,
`rotate90_ccw`, `rotate180`, `flip_horizontal`, `flip_vertical`, and
`rotate_arbitrary`). Those functions are pure and operate on a single plane set;
they do not know about `pictura_core::Document`, whose `layers` form a recursive
tree of pixel layers, adjustment layers, groups, decoded `LayerMask`s, and
document-level extra `channels`, and whose `composite` is a cached RGBA image.

`pictura-render` already owns that tree: `composite_rgba(doc)` walks the layer
stack (with group recursion, masks, opacity, and 27 blend modes) and produces the
4-channel straight-alpha composite. The M12 document operations described by
`docs/04-image-ops/image-size.md` (`IMG-001`), `canvas-size.md` (`IMG-002`), and
`image-rotation-and-flip.md` (`IMG-003`) are exactly this: apply the M10 buffer
math to every raster in the tree, move the bounds, update the document size, and
refresh the cached composite.

## Goals / Non-Goals

**Goals:**

- Add `resize_document`, `resize_canvas_document`, `rotate_document`, and
  `flip_document` to `pictura-render`, reusing the M10 kernels and remaps rather
  than reimplementing them.
- Define the recursive layer/group/mask/document-channel transformation, the
  rect mapping, the validation-and-error contract, and the composite
  recomputation as testable requirements.
- Prove the results with a psd-tools structural round-trip and a
  `composite == composite_rgba` check, both independent of the implementation.

**Non-Goals:**

- Arbitrary-angle document rotation (`rotate_arbitrary` stays a buffer op).
- Bicubic Smoother, Bicubic Sharper, and Bicubic Automatic kernels; the existing
  `Resample::{Nearest, Bilinear, Bicubic}` set is the whole surface.
- CMYK / Lab / Multichannel documents, and 16-/32-bit depths.
- The app `Image` menu, dialogs, quantity spin boxes, and anchor grid.
- Undo/history recording; the ops are destructive and the caller owns history.
- Smart Object source-pixel semantics and artboards.
- GPU acceleration; the CPU document path is the oracle.

## Decisions

**Document ops live in `pictura-render`, not `pictura-ops`.** `pictura-ops` is
the buffer math and depends only on `pictura-core`; it must not learn the layer
tree. `pictura-render` owns the tree traversal, masks, groups, and the composite
cache, so the document ops belong there as a new `src/document.rs`. Alternative
rejected: putting them in `pictura-ops` or `pictura-core`, which would duplicate
or invert the compositor dependency.

**`pictura-render` gains a `pictura-ops` dependency.** Every document op
delegates its per-plane work to the M10 functions: `resize` for resampling,
`resize_canvas` for the document-channel re-extension, and the exact remaps for
orientation. This keeps the kernels, the `Anchor` enum, the `Resample` enum, and
`OpsError` shared, and keeps the M10 ImageMagick oracle the single source of
truth for pixel behavior. The dependency is a normal (non-dev) one because the
shipped functions call into it. Alternative rejected: copying the kernels into
`pictura-render`, which would fork the oracle coverage.

**Recompute the cached composite after every op.** Each operation ends by
setting `doc.composite = composite_rgba(doc)`. The composite is derived, never
resampled or remapped independently, so it can never drift from the layer tree;
the requirement `doc.composite == composite_rgba(doc)` is the invariant the
oracle asserts. Alternative rejected: resampling `doc.composite` in parallel
with the layers, which duplicates work and can disagree with a fresh composite.

**In-place `&mut Document` with validate-first.** The functions take `&mut
Document` and return `Result<(), OpsError>`, matching `pictura_filters::apply`'s
in-place style. All validation (dimensions, `quarter_turns`, and malformed
channel/mask lengths) runs before the first mutation, so an error leaves the
document bit-identical. Alternative rejected: returning a fresh `Document`,
which would force a full tree clone per op for no gain in this destructive
model.

**One rect-mapping convention per operation.** Resize maps each rect edge with
`round(edge * scale)` so scaled edges stay adjacent and integer scale factors
are exact. Canvas translates both rects by the same `(dx, dy)` computed from the
anchor, so grow and shrink use one offset. Orientation remaps each rect with the
half-open rectangle that corresponds to the pixel remap (for 90° CW,
`new_left = H - bottom`, `new_right = H - top`, `new_top = left`,
`new_bottom = right`), which keeps a rotated layer buffer's swapped dimensions
equal to its rotated rect. Alternative rejected: scaling rect `width`/`height`
independently of `left`/`top`, which can produce inconsistent `right`/`bottom`
edges.

**Canvas growth is transparent; `background` is reserved.** The
`resize_canvas_document` signature carries `background: [u8; 4]` for parity with
`pictura_ops::resize_canvas`, but a document composite is rebuilt from layers
that only translate, so the newly exposed canvas is transparent and the
document-level channels are re-extended with a zero fill. Introducing the
extension color now would require the Background-layer gate that
`docs/04-image-ops/canvas-size.md` leaves open, so this change pins the added
region to transparent and records the parameter as reserved. Alternative
rejected: filling the added region with `background`, which contradicts the
transparent-background behavior of the CS6 command.

**Oracle: structural round-trip plus composite equality.** The document ops are
verified without ImageMagick (which has no layer-tree equivalent): build a
layered fixture, run the op, assert `doc.composite == composite_rgba(&doc)` in
`pictura-render`, then write the result with `pictura_codec::write_psd`, re-read
it, and open it with the independent `psd-tools` library to confirm the document
dimensions and layer bounds. This needs `pictura-codec` and `pictura-testkit` as
dev-dependencies (not normal ones) and reuses `scripts/validate_output.py`. The
oracle skips cleanly when `psd-tools` is missing and is never `#[ignore]`d.

## Risks / Trade-offs

- **Rect rounding at fractional scales.** Non-integer scale factors can move an
  edge by one pixel, so a resized layer may not map exactly onto its neighbor's
  original edge. Mitigation: the requirement fixes round-to-nearest and the
  tests pin integer scale factors where the mapping is exact; a later change can
  refine the edge rule without altering the signature.
- **Large memory copies.** A full-tree resample or remap clones every channel,
  which is costly for PSB-size documents. Mitigation: it matches the M10
  destructive model and the composite is recomputed once; tiled/cloned storage is
  a later optimization and not part of this contract.
- **Composite recomputation cost.** Every op recomputes the full composite even
  when only bounds moved. Mitigation: correctness first; the invariant
  `doc.composite == composite_rgba(doc)` is what the oracle checks, and an
  incremental composite cache can be added later without a contract change.
- **Reserved `background` parameter.** Carrying a parameter the change does not
  use can mislead callers. Mitigation: the spec states the added canvas is always
  transparent and `background` MUST NOT tint it; the design records it as
  reserved for the Background-layer extension color that `IMG-002` leaves open.
- **Orientation of non-square layer bounds.** The half-open rect remap is the
  part most likely to be off by one. Mitigation: the identity requirements (four
  quarter turns, doubled flip, CW then CCW) fail loudly on an off-by-one, and the
  psd-tools oracle checks the bounds independently.
- **psd-tools writes vs Pictura writes.** The oracle writes the transformed
  document with Pictura's own codec and only *reads* it back with psd-tools, so a
  codec defect could mask a document-op defect. Mitigation: the structural
  round-trip is paired with the render-level `composite == composite_rgba` check,
  which does not involve the codec at all.
- **Closed-kernel parity.** The Bicubic kernel is already marked behavioral
  parity only in M10; this change inherits that ceiling and does not claim CS6
  pixel parity for resampling.
