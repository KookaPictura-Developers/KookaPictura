## Context

M0–M9 built `pictura-core`'s planar 8-bit `PixelBuffer` and the filter/adjust
crates, but nothing operates on a buffer's *dimensions*: `docs/04-image-ops/`
specifies Image Size (`IMG-001`), Canvas Size (`IMG-002`), and Image Rotation
(`IMG-003`), and each future dialog or document command would otherwise
re-implement resampling, canvas extension, and index remaps. There is no
`pictura-ops` crate. `docs/dev/m10-image-ops.md` freezes the M10 contract; M10
implements only the primitive layer, with no document, layer, undo, or UI
integration.

The specs describe the full Photoshop behavior (dialogs, physical units, styles,
color modes); M10 deliberately implements the shared pixel-geometry core those
features need, so a later milestone can compose them without re-deriving the
math. Adobe's resize kernels are closed, so parity is behavioral only and every
approximation is marked.

## Goals / Non-Goals

**Goals:**

- Add `crates/pictura-ops` as a pure buffer-geometry crate over
  `pictura_core::PixelBuffer`, with no document/layer/undo/UI state.
- Provide the three primitive families with the frozen signatures:
  `resize` (per-channel kernel resampling with a new-buffer contract and
  `InvalidParams` validation), `resize_canvas` (nine-anchor grow/shrink with
  background fill and alpha), and the orientation functions (`rotate90_cw`,
  `rotate90_ccw`, `rotate180`, `flip_horizontal`, `flip_vertical`, and
  `rotate_arbitrary`).
- Keep input buffers immutable and return new buffers, so a caller can hold the
  before-image for undo without a copy.
- Record the resample kernels and clamp-to-edge behavior, the anchor placement
  math, and the exact remap index formulas.
- Reuse the ImageMagick oracle pattern from M6–M9: measured resize filter
  mapping, `-extent` with gravity, exact right-angle/flip checks, and a measured
  arbitrary-rotation delta.

**Non-Goals:**

- Bicubic Smoother, Bicubic Sharper, and Bicubic Automatic; only the three
  frozen `Resample` methods.
- Document and layer-tree integration, the Image Size / Canvas Size / Rotation
  dialogs, and app wiring.
- DPI / resolution math and physical units; CMYK / Lab; 16- and 32-bit samples.
- Smart Objects and the Crop-tool / Ruler straighten tools.
- GPU acceleration; the CPU result is the oracle.

## Decisions

**New `pictura-ops` crate, not additions to `pictura-filters` or
`pictura-core`.** Resize, canvas, and orientation are document-level geometry,
not filters, and they return new buffers rather than mutating in place; putting
them in `pictura-filters` would drag in the `FilterError`/in-place contract, and
putting them in `pictura-core` would grow the frozen document types. A small
crate keeps `PixelBuffer` as the only shared type. Alternative rejected: adding
methods to `PixelBuffer`, which couples geometry to the core type.

**A dedicated `OpsError` with `InvalidParams`.** The primitive layer has one
failure class — bad dimensions, malformed buffers, or out-of-range angles — so
`OpsError::InvalidParams` is the whole error surface, mirroring
`FilterError::InvalidParams` and the project's "errors instead of panics" rule.
No `Unsupported` variant is needed because the crate implements everything it
declares. Alternative rejected: `panic!` on bad input, forbidden by the shared
invariants.

**Return new buffers; never mutate the input.** `resize` and `resize_canvas`
take `&PixelBuffer` and return `Result<PixelBuffer, OpsError>`; the exact remaps
return `PixelBuffer` (they cannot fail on a valid buffer) and `rotate_arbitrary`
returns `Result<PixelBuffer, OpsError>`. This gives undo a free before-image and
keeps the functions pure and testable. Alternative rejected: in-place mutation,
which would force callers to clone first and complicate redo.

**Resample kernels: point, 2×2 tent, 4×4 Keys/Catmull-Rom, clamp-to-edge.**
`Nearest` selects the nearest sample; `Bilinear` blends the 2×2 neighbors;
`Bicubic` convolves the 4×4 neighborhood with a Keys cubic (`a = -0.5`,
Catmull-Rom). All source coordinates clamp to the image edge, so 1-pixel
dimensions and the 4×4 support near the border never read out of bounds. The
kernels are standard references, not Adobe's closed coefficients; the oracle
records the measured divergence. Alternative rejected: a single box filter,
which cannot meet the nearest-vs-bicubic distinction the specs require.

**Canvas as a signed-offset blit; one anchor math for grow and shrink.** The
anchor maps to a source offset in the destination (or a crop origin in the
source) computed from the same fractional position, so grow and shrink place the
image consistently. Growth allocates the destination filled with `background`
and copies the source rect; shrink copies only the in-bounds intersection,
discarding the rest. No interpolation is involved. Alternative rejected:
separate grow/shrink code paths, which would let the anchor math drift.

**Exact remaps as index formulas; arbitrary rotation as a bilinear inverse map.**
The right-angle rotations and flips are branch-free index remaps over the planar
channels, giving bit-exact reversible results. `rotate_arbitrary` computes the
bounding box, walks destination pixels, inverse-rotates each to a fractional
source coordinate, bilinearly samples with clamp-to-edge, and writes the
background where the coordinate maps outside the source rectangle. Alternative
rejected: forward-mapping the source (holes or overdraw). The angle is validated
to finite `-359.99..=359.99` before any work.

**No third-party dependencies.** The crate depends only on `pictura-core`. The
oracle uses the optional ImageMagick `magick` binary at test time, printing a
skip when absent, exactly as M6–M9; no fixtures are committed. Alternative
rejected: a pure-`std` rounding helper, unnecessary for the few arithmetic ops.

## Risks / Trade-offs

- **Closed-kernel divergence.** Adobe's Bicubic coefficients and edge handling
  are unpublished. Mitigation: the kernel is named as a standard Keys/Catmull-Rom
  approximation in the requirement and code, and the oracle records the measured
  ImageMagick delta so a later change can retune the kernel without changing the
  signature.
- **ImageMagick phase/rounding mismatch.** Point/triangle/Catmull-Rom resize and
  `-rotate` may differ by a sample or two from the implementation. Mitigation:
  the differential uses the tolerance justified by the measured delta, and an
  unmatchable case is classified no-equivalent with the observed delta recorded,
  not hidden behind a fixed tolerance.
- **Bounding-box rounding for arbitrary rotation.** `ceil` vs `round` changes
  the output by one pixel per dimension. Mitigation: the requirement fixes the
  formula and the rounding; the exact value is tested on known angles.
- **Background semantics for 3- versus 4-channel buffers.** The `background` is
  always RGBA; a 3-channel input ignores alpha. Mitigation: the requirement
  states the behavior per channel count so it is testable.
- **Large buffers and memory.** Returning new buffers doubles peak memory on a
  resize. Mitigation: accepted for the primitive layer; a later tile pipeline or
  in-place path is a document-integration concern, not M10.
- **No Unicode/document/doc UI integration.** M10 lands nothing the user can
  click. Mitigation: that is the explicit scope; the integration milestone
  composes these primitives and owns the dialogs and history records.

## Open Questions

- Whether a later milestone should expose the Bicubic Smoother / Sharper /
  Automatic kernels here or in the integration layer (deferred; the `Resample`
  enum is closed for M10).
- The exact ImageMagick tolerances for each resize filter and for
  `rotate_arbitrary`, to be filled from the measured differential at
  implementation time.
