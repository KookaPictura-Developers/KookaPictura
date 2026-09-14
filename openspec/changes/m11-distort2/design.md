## Context

M9 created the `distort` module in `crates/pictura-filters` and filled five
geometric warps — Twirl, Pinch, Spherize, Ripple, Wave — behind the existing
`Filter` enum, with in-place planar 8-bit writes, untouched alpha,
`FilterError` instead of panics, a shared bilinear inverse-mapping sampler, and
a seeded `ChaCha8Rng` for Wave. `docs/dev/m11-distort2.md` extends that
contract with the four remaining Distort warps; `docs/06-filters/distort-
filters.md` (`FILT-040`) describes the CS6 behavior and parameter ranges.

The plumbing already exists. The M11 scaffold in `src/lib.rs` declares
`Filter::PolarCoordinates { kind }`, `Filter::Shear { curve, fill }`,
`Filter::ZigZag { amount, ridges, style }`, and
`Filter::OceanRipple { size, magnitude, seed }`, plus the `PolarKind`,
`ShearFill`, and `ZigZagStyle` enums, and `apply` already dispatches to
`distort::polar_coordinates` / `shear` / `zigzag` / `ocean_ripple`. The
`distort` module is split into `coord.rs` and `ripples.rs` with frozen
signatures that currently return `FilterError::Unsupported`. M11 fills those
bodies rather than adding plumbing.

Adobe's warp kernels, falloff curves, and random ripple model are closed, so
parity is behavioral only; every approximation is marked, and the geometric
Distort family has no faithful ImageMagick operator except where a measured
delta justifies a tolerance.

## Goals / Non-Goals

**Goals:**

- Implement the four `distort` functions behind the existing stubs, as public
  pure functions over `PixelBuffer`, with no change to the M9 requirement
  surface or the `Filter` enum shape.
- Record the parameter ranges and validation, the displacement algorithms, and
  the shared invariants: in-place application, alpha untouched, bilinear
  inverse-mapping, edge handling, tiny-image safety, deterministic output,
  errors instead of panics.
- Reuse the M9 bilinear inverse-mapping sampler and the seeded `ChaCha8Rng`
  already used by the Noise, Pixelate, and Wave families rather than adding a
  second resampler or RNG.
- Record the oracle classification: ZigZag and Ocean Ripple no-equivalent with
  tolerance 0; Polar Coordinates and Shear measured against their closest
  operators with a recorded delta and a justified tolerance or a no-equivalent
  note.
- Wire the four kinds into the app filter menu with a unit test (M11-C).

**Non-Goals:**

- Diffuse Glow, Displace, Glass, and Lens Correction; their auxiliary inputs
  (texture / displacement map / EXIF profile) are not in M11.
- Smart Filters, the Filter-Gallery stack, selection / mask composition, and
  render wiring.
- 16-bit and 32-bit math; M11 is 8-bit only.
- CMYK / Lab; M11 operates on the RGB(A) planes.
- GPU compute; the CPU result is the oracle for any later accelerator.

## Decisions

**Fill the existing stubs, no new crate or enum.** The four variants, the three
enums, the module split, and the `apply` dispatch already exist in the scaffold,
and `src/distort/{coord,ripples}.rs` hold the frozen signatures. M11 replaces
the `Unsupported` bodies with implementations, so the error type, buffer
validation, sampler, and RNG stay shared. The M11 scaffold landed the plumbing;
this change only adds behavior. Alternative rejected: a new distort crate or
re-declared enums, which would fork the invariant checks and the oracle
harness.

**One inverse-mapping warp, shared by the family.** Every filter maps each
destination pixel back to a source coordinate through its own displacement and
resamples there; they differ only in the field. The M9 private `warp` helper
takes the destination-to-source mapping and does the plane-by-plane bilinear
reads, so Polar Coordinates, Shear, ZigZag, and Ocean Ripple reuse it and the
resampling is written and tested once. Alternative rejected: a per-filter
resampler, which duplicates the interpolation math the oracle already covers.

**Coordinate transforms vs ripple warps split the module the same way M9
split radial vs undulate.** Polar Coordinates and Shear are coordinate-space
transforms that read the source grid directly, so they live in `coord.rs`.
ZigZag and Ocean Ripple are radial / random ripple displacements, so they live
in `ripples.rs`. The split matches the scaffold and keeps the two M11-A
workstreams disjoint.

**Polar Coordinates is a direct coordinate reinterpretation.** For
`RectangularToPolar`, the destination pixel's `(x, y)` normalized to the image
center supplies an angle and a radius that index the source grid; for
`PolarToRectangular` the roles swap. Both directions resample bilinearly, and
the pair round-trips to within resampling error. This is the standard
"unwraps / wraps an image about its center" behavior. Alternative rejected: a
forward scatter, which would leave holes and make inversion undefined.

**Shear interpolates the control curve piecewise-linearly.** The `curve` is a
list of `(x, y)` control points in normalized `-1..=1` space; a column's
normalized `x` position is interpolated to a `y` and used as the vertical
source offset, constant down the column. A flat curve is a no-op. Rows shifted
off-canvas follow `fill`: `WrapAround` samples the opposite edge,
`RepeatEdgePixels` repeats the nearest edge. The contract rejects a curve with
fewer than two points, a non-finite coordinate, or non-increasing `x` before
any write. Alternative rejected: a spline through the points, which Photoshop's
dialog does not promise and which would need its own monotonicity guarantees.

**ZigZag is a radial displacement with a ridge count.** `amount` scales the
magnitude, `ridges` sets the number of direction reversals from center to edge,
and `ZigZagStyle` selects the geometry (`AroundCenter` rotation,
`OutFromCenter` radial push, `PondRipples` diagonal bias). `amount` `0.0` and
`ridges` `0` are the degenerate cases: no displacement and a single direction.
The style enums are closed sets, so only the scalar and integer ranges are
validated. Alternative rejected: a fixed ridge model, which would ignore the
scriptable `ridges` and `style` parameters.

**Ocean Ripple reuses the seeded RNG and the M9 ripple shape.** It constructs
`ChaCha8Rng::seed_from_u64(seed)`, the same generator the Noise, Pixelate, and
Wave families use, places randomly spaced ripples whose frequency comes from
`size` and amplitude from `magnitude`, and sums them into the displacement
field. The same seed therefore reproduces the field bit-identically, which is
what makes Randomize reproducible in undo/redo. Alternative rejected: a bespoke
LCG, which would add a second generator to maintain and test.

**Validation up front, before any write.** `apply` validates the buffer once;
each function validates its own parameters before touching a sample, so a bad
argument leaves the buffer unchanged and cannot partially apply. Shear's curve
shape is validated structurally (count, finiteness, strictly increasing `x`),
and the integer ranges for ZigZag `ridges` and Ocean Ripple `size` / `magnitude`
are checked against the contract. Alternative rejected: clamping silently,
which would hide a bad UI value.

**Approximations marked behavioral parity only.** Adobe's exact Polar
pinch/phase origin, Shear curve treatment, ZigZag ridge geometry, and Ocean
Ripple noise distribution are undocumented. Each is named as an approximation
in the requirement and the code; the enum and parameter shapes stay stable so a
later change can retune one function without touching the contract.

**Oracle split: two no-equivalent, two measured.** ImageMagick is a sanity
oracle, not a parity oracle, exactly as in M6–M9. ZigZag and Ocean Ripple have
no faithful operator and are classified no-equivalent with tolerance 0 and
property tests. Polar Coordinates and Shear are diffed against
`-distort DePolar` / `-distort Polar` and `-shear`; their tolerance is justified
by the recorded maximum and mean per-sample delta, and a case that cannot be
aligned is classified no-equivalent with the delta recorded. The mapping table
gains one row per variant, so a new variant cannot be added without classifying
it. Alternative rejected: asserting equality against `-distort` or `-shear`,
which would encode one resampler as truth and fail for a faithful Photoshop
match.

**No committed fixtures; ImageMagick stays optional.** The differential tests
run `magick` at test time and print a skip when it is absent; no binary
references are checked in and nothing is marked `#[ignore]`.

## Risks / Trade-offs

- **Closed-kernel approximation drift.** The Polar coordinate origin and phase,
  the Shear curve interpolation, the ZigZag ridge geometry, and the Ocean
  Ripple noise distribution may diverge from CS6. Mitigation: each is named as
  an approximation in the requirements and the code; the enum and parameter
  shapes stay stable so a later change can retune one function without a
  contract change.
- **Polar coordinate origin and phase.** Whether the polar seam starts at the
  positive x-axis, the top, or an offset, and the radius normalization, are
  inferred. Mitigation: the requirement fixes the observable properties
  (the two directions differ and round-trip; the arrangement changes) and the
  origin can be shifted without a contract change if a reference render
  disagrees.
- **Shear partial-row sampling.** A non-integer vertical shift resamples whole
  columns bilinearly; the horizontal coordinate is unchanged, so there is no
  horizontal blur. Mitigation: the unit tests pin the no-op flat curve and the
  off-canvas fill behavior, and the shared sampler owns the interpolation.
- **ZigZag ridge cost and definition.** The number of reversals from center to
  edge and the exact modulation per style are inferred. Mitigation: the tests
  pin the amount-0 no-op and that larger `ridges` changes the field; the field
  can be retuned without a contract change.
- **Ocean Ripple determinism under tiling.** A tile-local RNG that reseeds per
  tile would break bit-identical output across tile splits. Mitigation: the
  contract fixes the seed as the single source of randomness; a
  coordinate-derived seed for huge PSB tiles is an integration concern.
- **Degenerate geometry on small images.** Polar Coordinates and ZigZag have no
  meaningful radius for a 1×1 image, and Shear has no neighbor to shift toward.
  Mitigation: the center and radius guards make a 1×1 a bit-exact no-op and
  keep 1-px-wide / 1-px-tall images panic-free.
- **Cross-platform float determinism.** The displacement math is `f64`; IEEE
  arithmetic is deterministic on a platform but not guaranteed bit-identical
  across architectures. Mitigation: the seeded RNG is the only
  nondeterministic-looking input; the contract requires same-input same-seed
  reproducibility and the oracle records an explicit tolerance.
- **8-bit only.** The contract assumes 8-bit planar planes. A 16/32-bit or
  CMYK/Lab port needs wider samples and non-RGB planes; it is deferred, not
  accidental.
