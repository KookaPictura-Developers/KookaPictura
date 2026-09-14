## Context

M6 created `crates/pictura-filters` as the CPU reference for the destructive
Filter menu: one `Filter` enum plus `apply`, in-place planar 8-bit writes, alpha
untouched, `FilterError` instead of panics, shared `kernel` / `luma` helpers, a
seeded `ChaCha8Rng` used by the Noise family, and an ImageMagick differential
oracle for the operators whose semantics match. M7 and M8 added the Other /
Stylize and Pixelate families the same way. `docs/dev/m9-distort.md` extends that
contract with the **Distort** family; `docs/06-filters/distort-filters.md`
(`FILT-040`) and `filters-overview.md` describe the CS6 behavior and parameter
ranges.

The plumbing already exists. `src/lib.rs` declares `Filter::Twirl { angle }`,
`Filter::Pinch { amount }`, `Filter::Spherize { amount, mode }`,
`Filter::Ripple { amount, size }`, and `Filter::Wave { generators, wavelength,
amplitude, kind, scale, seed, repeat_edge }`, plus the `SpherizeMode`,
`RippleSize`, and `WaveType` enums, and `apply` already dispatches to
`distort::twirl` / `pinch` / `spherize` / `ripple` / `wave`. The `distort`
module is split into `radial.rs` and `undulate.rs` with frozen signatures that
return `FilterError::Unsupported`. M9 fills those bodies rather than adding
plumbing.

Adobe's warp kernels, falloff curves, and Wave generator model are closed, so
parity is behavioral only; every approximation is marked, and none of the five
has a faithful ImageMagick operator.

## Goals / Non-Goals

**Goals:**

- Implement the five `distort` functions behind the existing stubs, as public
  pure functions over `PixelBuffer`, with no change to the M6/M7/M8 requirement
  surface or the `Filter` enum shape.
- Record the parameter ranges and validation, the displacement algorithms, and
  the shared invariants: in-place application, alpha untouched, bilinear
  inverse-mapping, edge handling, tiny-image safety, deterministic output,
  errors instead of panics.
- Share one bilinear inverse-mapping sampler across the family so the five
  filters differ only in how their displacement field is generated.
- Reuse the seeded `ChaCha8Rng` already used by the Noise and Pixelate families
  for Wave rather than adding a second RNG.
- Record the oracle classification: all five no-equivalent with tolerance 0, a
  property or known-value test, a non-empty note, and the observed delta.
- Wire the new kinds into the app filter menu with a unit test (M9-C).

**Non-Goals:**

- Diffuse Glow, Displace, Glass, Ocean Ripple, Polar Coordinates, Shear, and
  ZigZag; their auxiliary inputs (texture / displacement map) are not in M9.
- Lens Correction and the Auto Correction workspace.
- Smart Filters, the Filter-Gallery stack, selection / mask composition, and
  render wiring.
- 16-bit and 32-bit math; M9 is 8-bit only.
- CMYK / Lab; M9 operates on the RGB(A) planes.
- GPU compute; the CPU result is the oracle for any later accelerator.

## Decisions

**Fill the existing stubs, no new crate or enum.** The five variants, the three
enums, and the `apply` dispatch already exist in `src/lib.rs`, and
`src/distort/{radial,undulate}.rs` hold the frozen signatures. M9 replaces the
`Unsupported` bodies with implementations, so the error type, buffer
validation, and RNG stay shared. Alternative rejected: a new distort crate or
re-declared enums, which would fork the invariant checks and the oracle
harness.

**One inverse-mapping warp, shared by the family.** Every filter maps each
destination pixel back to a source coordinate through its own displacement and
resamples there; they differ only in the field. A private `warp` helper takes
the destination-to-source mapping and does the plane-by-plane bilinear reads,
so the resampling is written and tested once. This is the shape the docs'
pipeline note asks for without paying for a trait. Alternative rejected: the
docs' aspirational `Warp` trait plus `DisplacementField` struct, which would be
a trait with one implementation and a data structure for a per-pixel value the
filters can compute inline (`AGENTS.md` rule 3).

**Bilinear sampling, reused not re-implemented.** `blur.rs` already has a
`bilinear(plane, w, h, x, y)` clamp-to-edge sampler. M9 promotes that sampler to
`pub(crate)` and adds a single wrap branch for Wave's `repeat_edge = false`,
rather than adding a second resampler to the crate. Bilinear is the frozen
choice for M9; bicubic is deferred. Alternative rejected: copying the sampler
into `distort`, which duplicates the interpolation math the oracle already
covers.

**Displacement fields differ per filter, the sampler does not.** Twirl rotates
the source coordinate about the center by an angle that decays from the center
to the edge. Pinch remaps the normalized radius through a monotone curve that
contracts (positive) or expands (negative) toward the center. Spherize applies
the same radial remap but as a 3D sphere wrap and masks the axes by
`SpherizeMode`. Ripple adds a sinusoid along one axis whose frequency comes from
`RippleSize`. Wave sums `N` seeded generators. All five express their field as
an `(x, y)` source lookup and hand it to the shared sampler.

**Seeded RNG reused from the Noise module.** Wave constructs
`ChaCha8Rng::seed_from_u64(seed)`, the same generator `noise::add_noise` and the
Pixelate family use, so there is one seeded RNG convention in the crate and redo
reproduces bit-identically. Each generator's wavelength and amplitude are drawn
once from its range, so the same seed yields the same field. Alternative
rejected: a bespoke LCG, which would add a second generator to maintain and
test.

**Undefined areas via `repeat_edge`.** Wave carries `repeat_edge: bool`: `true`
samples the nearest edge sample when the displacement falls outside the image
(clamp / repeat edge), `false` wraps to the opposite edge. The radial and
sinusoidal filters keep their lookups inside the image with clamp-to-edge
sampling. This matches the CS6 "defining undistorted areas" rule for the one
Distort filter in scope that can push pixels off-canvas.

**Validation up front, before any write.** `apply` validates the buffer once;
each function validates its own parameters before touching a sample, so a bad
argument leaves the buffer unchanged and cannot partially apply. The ranges and
the `min`/`max` ordering rules for Wave come from the brief and `FILT-040`.
Alternative rejected: clamping silently, which would hide a bad UI value.

**Approximations marked behavioral parity only.** Adobe's exact falloff
functions, the radial remap curve, the ripple phase, and the Wave generator
model are undocumented. Each is named as an approximation in the requirement
and the code; the enum and parameter shapes stay stable so a later change can
retune one function without touching the contract.

**Oracle split: all five no-equivalent.** ImageMagick is a sanity oracle, not a
parity oracle, exactly as in M6–M8. `-swirl`, `-implode` / `-explode`, and
`-wave` are the closest operators, but their falloff and generator models differ
from Adobe's, so Twirl, Pinch, Spherize, Ripple, and Wave are each classified
no-equivalent with tolerance 0, a property or known-value test, a non-empty
note, and the observed delta recorded. The mapping table gains one row per
variant, so a new variant cannot be added without classifying it. Alternative
rejected: asserting equality against `-swirl` or `-wave`, which would encode
one closed kernel as truth and fail for a faithful Photoshop match.

**No committed fixtures; ImageMagick stays optional.** The differential tests
run `magick` at test time and print a skip when it is absent; no binary
references are checked in and nothing is marked `#[ignore]`.

## Risks / Trade-offs

- **Closed-kernel approximation drift.** The Twirl falloff, the Pinch /
  Spherize radial curve, the Ripple phase and frequency mapping, and the Wave
  generator model may diverge from CS6. Mitigation: each is named as an
  approximation in the requirements and the code; the enum and parameter shapes
  stay stable so a later change can retune one function without a contract
  change.
- **Rotation direction sign convention.** Whether a positive angle turns
  clockwise or counter-clockwise is inferred. Mitigation: the requirement fixes
  the observable property (opposite signs rotate in opposite directions, the
  center rotates more than the edge) and the sign can be flipped without a
  contract change if a reference render disagrees.
- **Wave cost with many generators.** The field is `O(pixels × generators)`;
  999 generators at a large amplitude is the worst case. Mitigation: the
  per-pixel generator sum is the correctness baseline; progress and
  cancellation stay a tile-pipeline concern, and a separable or cached-field
  upgrade keeps the same signature.
- **Deterministic randomness under tiling.** A tile-local RNG that reseeds per
  tile would break bit-identical output across tile splits. Mitigation: the
  contract fixes the seed as the single source of randomness; a
  coordinate-derived seed for huge PSB tiles is an M9 integration concern.
- **Degenerate geometry on small images.** A 1×1 or 1-px image has no
  meaningful radius or neighbor. Mitigation: the center and radius guards make
  a 1×1 a bit-exact no-op and keep 1-px-wide / 1-px-tall images panic-free.
- **Cross-platform float determinism.** The displacement math is `f64`; IEEE
  arithmetic is deterministic on a platform but not guaranteed bit-identical
  across architectures. Mitigation: the seeded RNG is the only
  nondeterministic-looking input; the contract requires same-input same-seed
  reproducibility and the oracle records an explicit tolerance.
- **8-bit only.** The contract assumes 8-bit planar planes. A 16/32-bit or
  CMYK/Lab port needs wider samples and non-RGB planes; it is deferred, not
  accidental.
