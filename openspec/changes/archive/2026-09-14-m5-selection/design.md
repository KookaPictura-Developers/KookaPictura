## Context

M5 adds the first selection primitive to Kooka Pictura. The long-form contract
is `docs/08-selection/*.md` (`SEL-001`, `SEL-002`, `SEL-005`, `SEL-010`,
`SEL-012`), plus the M5 brief `docs/dev/m5-selection.md`. Adobe publishes the
observable semantics but not the kernels, the colour-distance metric, or the
morphology structuring element, so several choices below are inferred and
marked as such.

The crate must serve masked rendering (adjustment layers, fills, layer masks)
without touching pixel colour, and must round-trip through the 8-bit grayscale
channels the PSD codec already stores (`pictura_core::Channel`).

## Goals / Non-Goals

**Goals:**

- A single, deterministic representation of a selection as an 8-bit coverage
  mask that survives save/load through an alpha channel byte for byte.
- The four Photoshop combine operations with well-defined integer semantics.
- Modify operations (feather, expand, contract, border, smooth) that preserve
  partial coverage.
- Image-derived tools (magic wand, grow, similar, colour range) that agree with
  one another because they share one colour-distance metric.
- A runnable ImageMagick sanity oracle for the operations that have a faithful
  mathematical equivalent.

**Non-Goals:**

- Refine Edge (smart radius, decontaminate colours), quick-selection brush
  heuristics, vector-mask boolean ops, and per-channel 16-bit masks.
- Exact Adobe parity for the closed algorithms; these are behavioural
  approximations and are documented as such.
- The Qt/UI wiring; that is `m5-selection-integration`.

## Decisions

### Mask as an 8-bit coverage value, not a boolean region or an `f32` field

A selection is `{ width, height, data: Vec<u8> }` with 0 = outside and 255 =
inside. `docs/` proposes a `f32` `CoverageMask`, but M5's storage contract is
the 8-bit grayscale alpha channel; keeping the selection in `u8` makes
`to_channel`/`from_channel` a memcpy and makes quick masks, layer masks, and
`SEL-012` saves use one representation. Soft coverage is retained rather than
thresholded; only the marching-ants contour (a later concern) would binarise at
128. Alternative rejected: `f32` — no consumer needs sub-8-bit coverage yet,
and `16/32-bit masks` are explicitly out of scope.

### Integer combine arithmetic

Combine is per-pixel and branch-free: Replace copies `b`, Add is `max(a, b)`,
Subtract is `a.saturating_sub(b)`, and Intersect is `((a as u16 * b as u16 +
127) / 255) as u8` (round-half-up). `docs/` writes Subtract as `min(E, 255-N)`
and Intersect as `E*N/255`; saturating subtraction is the same operation
clamped at 0 and avoids `u8` underflow. All arithmetic stays integral so results
are deterministic and match stored channel bytes. Alternative rejected: float
combine then round — extra casts, no benefit at 8-bit, and it risks
platform-dependent rounding.

### Separable square structuring element for expand/contract

`expand`/`contract` run a separable `(2r+1)`-wide min/max pass with clamped
edges, i.e. O(r) per pixel rather than O(r²). The spec proposes a
threshold-independent Euclidean distance transform (a round disk). The two
differ at the diagonal corners: on the 8×8 oracle mask exactly 4 corner pixels
diverge by a whole step. We keep the square element because it reuses the
separable shape of the feather pass, is cheap on large documents, and the
canvas-edge clamp matches Photoshop's documented edge exemption. The divergence
is recorded in the oracle mapping and `tests/README.md`; upgrading to a distance
transform is the named follow-up if a parity test demands a disk.

### Inferred feather mapping and smooth filter

Feather is a separable Gaussian with `sigma = radius / 2`, support
`ceil(3 * sigma)`, and `f64` accumulation clamped to `[0, 255]`; radius 0 is the
identity. Adobe does not publish the radius-to-sigma mapping, so it is inferred
and calibrated against `-gaussian-blur 0x(radius/2)`. Smooth is a majority vote
(the median) over a `(2r+1)²` square window computed from a 256-bin histogram
per pixel; the spec states the majority rule but not the window shape, so the
square window is inferred.

### One Chebyshev colour metric, shared by wand/grow/similar/colour-range

The image-derived tools compare colours by the maximum per-channel absolute
difference. This is Adobe-unspecified; choosing one metric and reusing it means
the wand, Grow, and Similar cannot disagree (the docs require them to share the
wand's tolerance). `color_range` turns the same distance into a soft ramp.
Magic wand and Grow are four-connected flood fills; Similar and colour-range are
global passes. These have no faithful ImageMagick equivalent, so they are
covered by known-value and property tests rather than a differential oracle.

### Errors, not panics; clamped parameters

`SelectError::{SizeMismatch, InvalidParams}` (via `thiserror`) is returned for
mismatched dimensions, wrong-length channels, and out-of-bounds seeds. Radii are
clamped to the documented ceilings (feather 250, morphology 100, border 200,
smooth 100) so a dialog value can never panic the core.

## Risks / Trade-offs

- **Structuring-element divergence** (square vs. Euclidean disk) → documented in
  the oracle mapping and README; the oracle passes `Square:r` explicitly and
  exposes `Disk:r` for inspection. Upgrade path is a distance transform.
- **Chebyshev colour distance is unproven against CS6** → all image-derived
  tools are behavioural approximations; tests assert documented properties
  (monotonicity, adjacency vs. global) rather than parity.
- **Smooth is a median, IM `Smooth` is a mean** → the oracle row carries a
  tolerance and a note, and Smooth is not treated as a parity claim.
- **Feather radius-to-sigma is inferred** → calibrated to `-gaussian-blur`; the
  mapping is called out as inferred, not verified.
- **u8 coverage cannot represent 16/32-bit precision** → accepted; per-channel
  high-bit-depth masks are out of scope and the representation can grow later.
- **O(width·height·radius) morphology and blur on PSB-size documents** → a
  named ceiling; tiled/distance-transform implementations are the upgrade path
  when full-document modify shows up in profiles.
