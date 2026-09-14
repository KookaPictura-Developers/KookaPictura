## Context

M6 created `crates/pictura-filters` as the CPU reference for the destructive
Filter menu: one `Filter` enum plus `apply`, in-place planar 8-bit writes, alpha
untouched, `FilterError` instead of panics, shared `kernel` / `luma` helpers, and
an ImageMagick differential oracle for the operators whose semantics match.
`docs/dev/m7-stylize-other.md` extends that contract with the **Other** and
**Stylize** families; `docs/06-filters/other-filters.md` (`FILT-070`) and
`docs/06-filters/stylize-filters.md` (`FILT-050`) describe the CS6 behavior and
parameter ranges.

The code does not exist yet: M7 is a forward-looking milestone. This change
records WHAT the family must do and HOW it fits the existing crate, so M7-A
through M7-E can be implemented against reviewable requirements. Adobe's
kernels are closed, so parity is behavioral only where an oracle exists and
every approximation is marked.

## Goals / Non-Goals

**Goals:**

- Add the eight M7 `Filter` variants to the existing enum and dispatch them
  through the existing `apply`, with no change to the M6 requirement surface.
- Ship Maximum, Minimum, Offset, High Pass, and Custom in
  `src/other.rs`, and Emboss, Find Edges, and Solarize in `src/stylize.rs`, each
  as a public pure function over `PixelBuffer`.
- Record the parameter ranges and validation, the algorithms, and the shared
  invariants: in-place application, alpha untouched, clamp-to-edge, tiny-image
  safety, deterministic output, errors instead of panics.
- Reuse the M6 Gaussian kernel for High Pass rather than adding a second blur.
- Record the ImageMagick oracle split: which variants are diffed, at what
  tolerance, and which are documented no-equivalent with property or
  known-value tests, one mapping-table row per variant.
- Wire the new kinds into the app filter menu with a unit test (M7-D).

**Non-Goals:**

- Stylize Wind, Tiles, Extrude, and Trace Contour; Oil Paint; Diffuse; Glowing
  Edges.
- Distort (geometric warps), Pixelate, Render, Liquify, Blur Gallery, Camera
  Raw, and Lens Correction.
- Smart Filters, the Filter Gallery, selection / mask composition, and render
  wiring.
- 16-bit and 32-bit math; M7 is 8-bit only.
- CMYK / Lab; M7 operates on the RGB(A) planes.
- The optional HSB/HSL plug-in (`FILT-070` gates it behind an optional install).
- GPU compute; the CPU result is the oracle for any later accelerator.

## Decisions

**Extend the M6 `Filter` enum and `apply` dispatch, no new crate.** The M7
functions live in two new modules of the existing `pictura-filters` crate, so
the error type, the buffer validation, the clamp helper, and the Gaussian
kernel are shared rather than duplicated. Alternative rejected: a second
filter crate, which would fork the invariant checks and the oracle harness.

**Per-variant public functions, dispatched by `apply`.** `other::maximum`,
`other::minimum`, `other::offset`, `other::high_pass`, `other::custom`,
`stylize::emboss`, `stylize::find_edges`, and `stylize::solarize` are public so
unit tests and a later tile pipeline can call them directly, matching the M6
`blur::gaussian` / `noise::median` shape. `apply` validates the buffer once and
dispatches.

**Maximum / Minimum as a direct square-window scan, reading an editable copy.**
Each output pixel is the per-channel max or min over the `(2r+1)²` footprint
with clamp-to-edge, computed into a destination plane so the source is not read
mid-write. Radius is clamped to 100 per `FILT-070`'s scriptable maximum and
`radius == 0` short-circuits to a no-op. The footprint is square because CS6
dropped the later CC `Preserve: Squareness/Roundness` control. A separable or
monotonic-deque upgrade is possible but unnecessary at radius ≤ 100; the direct
scan is the correctness baseline. Alternative rejected: a generic morphology
abstraction for two shapes (YAGNI).

**Offset as an index remap with two edge policies.** `wrap == true` indexes
`(x − h).rem_euclid(w)`, `(y − v).rem_euclid(h)`; `wrap == false` clamps an
out-of-range source to background and otherwise copies. Only the three color
planes move; alpha is left in place. `(0, 0)` short-circuits. The background
color is a parameter (`[u8; 3]`), not global state, so a re-apply is
reproducible and the undo record can store the color used.

**High Pass reuses the M6 Gaussian.** `out = clamp(orig − gaussian(orig, σ) +
128)` with `σ = sigma_from_radius(radius)`, calling the same
`kernel::sigma_from_radius` and Gaussian kernel as `blur::gaussian` and
Unsharp Mask. The radius must be finite and `> 0`; the dialog range is 0.1–250.0
but the function accepts any finite positive value. Adobe's exact blur kernel
and sigma mapping are closed, so this is behavioral parity only. Alternative
rejected: a bespoke high-pass frequency kernel, which would diverge from the
shared blur and add a second tuning surface.

**Custom as an explicit 5×5 f64 convolution.** `out = clamp(Σ kernel[i][j] ·
src / scale + offset)` per color channel with clamp-to-edge, traversed
left-to-right / top-to-bottom with `kernel[2][2]` at the evaluated pixel.
Accumulate in `f64` so extreme matrices do not overflow, matching `FILT-070`'s
note that the raw sum can exceed `i32`. `scale == 0` and any non-finite kernel
entry, scale, or offset are rejected with `InvalidParams` before any write. The
identity kernel with `scale 1`, `offset 0` is the pass-through test. The kernel
is `[[f64; 5]; 5]` (not `[[i32; 5]; 5]`) to match the frozen M7 signature and
to admit fractional edge kernels. Alternative rejected: an image- or 3×3-only
operator (the CS6 dialog is a fixed 5×5 grid).

**Emboss as a directional difference with a gray bias.** Compute the directional
luma difference along `angle`, scale by `height`, add 128, and scale the
deviation from mid-gray by `amount` percent; all three color channels receive
the same result. `angle` is finite in `-360.0..=360.0`; `height` and `amount`
are finite and `> 0`. The exact CS6 kernel is closed, so this is behavioral
parity only, tested by the gray output and the angle-sign highlight/shadow swap.
`Edit > Fade` (M6/`LAY-021`) is the supported way to keep color, so M7 does not
add a color-mixing option.

**Find Edges as Sobel magnitude inverted, Solarize as the fixed curve.** Find
Edges computes the per-channel Sobel gradient magnitude and renders
`255 − clamp(magnitude)` so edges are dark on white, matching Help. Solarize is
the pointwise curve `if v >= 128 { 255 − v } else { v }`, with no parameters.
Both are no-dialog filters, so they carry no parameter validation beyond the
shared buffer contract.

**Oracle split: diff where semantics match, document where they do not.**
ImageMagick is a sanity oracle, not a parity oracle, exactly as in M6.
Maximum / Minimum (`-morphology Dilate|Erode Square:N`), Offset wrap (`-roll`),
and Custom (`-convolve` with `convolve:scale` / `-bias`) are diffed; Solarize
(`-solarize 50%`) is exact at 8-bit. Offset background fill, High Pass (pending
a verified `-compose mathematics` recipe), Find Edges (differing detector and
opposite polarity), and Emboss (unless `-emboss` is verified faithful) are
no-equivalent with tolerance 0 and property or known-value tests. The mapping
table has one row per variant, so a new variant cannot be added without
classifying it.

**No committed fixtures; ImageMagick stays optional.** The differential tests
run `magick` at test time and print a skip when it is absent; no binary
references are checked in and nothing is marked `#[ignore]`.

## Risks / Trade-offs

- **Closed-kernel approximation drift.** The Emboss directional kernel, the
  High Pass sigma mapping, the Find Edges detector, and Custom versus Adobe's
  rounding may diverge from CS6. Mitigation: each is named as an approximation
  in the requirements and the code; the enum and parameter shapes stay stable so
  a later change can retune one function without touching the contract.
- **Maximum / Minimum cost.** The direct `(2r+1)²` scan is `O(area·r²)` and
  radius 100 is 201² taps per pixel. Mitigation: the square footprint is the
  correctness baseline; a separable or monotonic-deque upgrade can replace the
  inner loop without changing the public contract, and a 100-radius morphology
  on a large document is a progress/cancel concern owned by the tile pipeline,
  not this crate.
- **Offset wrap reference frame.** `FILT-070` is unsure whether CS6 wraps
  relative to the selection or the whole document. Mitigation: M7 wraps in
  document space and the divergence is recorded; selection-relative wrap is a
  later integration concern.
- **Custom edge policy and alpha.** Photoshop's out-of-bounds policy and
  whether alpha is convolved are undocumented. Mitigation: clamp-to-edge and
  color-channels-only are stated as deliberate, testable choices and the
  divergence is documented.
- **Emboss/Range ambiguity.** `FILT-050` marks the Emboss height range and
  defaults as inferred and the angle as `-360..=360`. Mitigation: the
  requirement fixes only the sourced angle bound plus finite/positive
  height/amount, so a CS6 measurement can tighten the dialog range without a
  contract change.
- **ImageMagick version and operator conventions.** The radius→window and
  convolution scale conventions can differ between builds and may not match our
  mapping. Mitigation: per-filter tolerances are explicit, the verified version
  is recorded, and the no-equivalent list bounds how much can regress silently.
- **8-bit only.** The contract assumes 8-bit planar planes. A 16/32-bit port
  needs wider samples and float accumulation; it is deferred, not accidental.
