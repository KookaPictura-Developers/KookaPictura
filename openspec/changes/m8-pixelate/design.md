## Context

M6 created `crates/pictura-filters` as the CPU reference for the destructive
Filter menu: one `Filter` enum plus `apply`, in-place planar 8-bit writes, alpha
untouched, `FilterError` instead of panics, shared `kernel` / `luma` helpers, a
seeded `ChaCha8Rng` used by the Noise family, and an ImageMagick differential
oracle for the operators whose semantics match. `docs/dev/m8-pixelate.md`
extends that contract with the **Pixelate** family; `docs/06-filters/`
`pixelate-filters.md` (`FILT-084`) and `filters-overview.md` describe the CS6
behavior and parameter ranges.

The code does not exist yet: `src/pixelate.rs` holds seven
`FilterError::Unsupported` stubs and the `Filter` variants already dispatch to
them, so M8 fills the bodies rather than adding plumbing. Adobe's kernels are
closed, so parity is behavioral only; every approximation is marked, and only
Mosaic has a faithful oracle.

## Goals / Non-Goals

**Goals:**

- Implement the seven `src/pixelate.rs` functions behind the existing stubs, as
  public pure functions over `PixelBuffer`, with no change to the M6/M7
  requirement surface.
- Record the parameter ranges and validation, the algorithms, and the shared
  invariants: in-place application, alpha untouched, clamp-to-edge, tiny-image
  safety, deterministic output, errors instead of panics.
- Reuse the seeded `ChaCha8Rng` already used by the Noise family for
  Crystallize, Mezzotint, and Pointillize rather than adding a second RNG.
- Reuse the shared block-mean and clamp helpers where they fit.
- Record the ImageMagick oracle split: Mosaic diffed, the other six documented
  no-equivalent with the observed delta, one mapping-table row per variant.
- Wire the new kinds into the app filter menu with a unit test (M8-C).

**Non-Goals:**

- Distort, Render, Liquify, Blur Gallery, Camera Raw, and Lens Correction.
- Smart Filters, the Filter-Gallery stack (Pixelate is not a gallery category
  in CS6), selection / mask composition, and render wiring.
- 16-bit and 32-bit math; M8 is 8-bit only.
- CMYK / Lab; M8 operates on the RGB(A) planes.
- GPU compute; the CPU result is the oracle for any later accelerator.

## Decisions

**Fill the existing stubs, no new crate or enum.** The seven variants and the
`MezzotintType` enum already exist in `src/lib.rs` with their `apply` dispatch,
and `src/pixelate.rs` holds the frozen signatures. M8 replaces the
`Unsupported` bodies with implementations, so the error type, buffer
validation, clamp helper, and RNG stay shared. Alternative rejected: a new
pixelate crate or re-declared enum, which would fork the invariant checks and
the oracle harness.

**Per-filter public functions, dispatched by `apply`.** `pixelate::mosaic`,
`crystallize`, `facet`, `fragment`, `mezzotint`, `pointillize`, and
`color_halftone` are public so unit tests and a later tile pipeline can call
them directly, matching the M6 `blur::gaussian` / M7 `other::maximum` shape.
`apply` validates the buffer once and dispatches.

**Seeded RNG reused from the Noise module.** Crystallize, Mezzotint, and
Pointillize construct `ChaCha8Rng::seed_from_u64(seed)`, the same generator
`noise::add_noise` uses, so there is one seeded RNG convention in the crate and
redo reproduces bit-identically. The seed is part of the `Filter` variant and
is stored in the history record. Alternative rejected: a bespoke LCG, which
would add a second generator to maintain and test.

**Mosaic as block mean; the reusable path is the shared block helper.** Each
`cell_size × cell_size` block accumulates its color sums in `u64` and writes
`sum / count` to every member, averaging over the available pixels when a block
straddles the edge. This is the one Pixelate filter with a faithful ImageMagick
oracle. Alternative rejected: a separable box pass, unnecessary for a single
non-overlapping partition.

**Crystallize as a jittered Voronoi tessellation.** One seed point per grid
cell, jittered by the RNG within the cell, then every pixel is assigned to its
nearest point by squared distance and filled with the mean color of its
members. A brute-force nearest-seed scan is the correctness baseline;
`cell_size` 3..=300 and a large document make this the most expensive member.
Mitigation: a grid-bucketed nearest-neighbor search can replace the inner loop
without changing the signature. Alternative rejected: a Delaunay
triangulation, which is far more machinery than the online assignment needs.

**Facet as a fixed-pass similar-neighbor average.** A fixed number of passes
replaces each pixel with the local average of neighbors whose value is within a
similarity threshold, flattening gradients while preserving strong edges. The
pass count and threshold are constants, not UI parameters, per the brief. The
exact Adobe kernel is closed, so this is behavioral parity only, tested by the
flatten-a-gradient and preserve-a-step-edge properties.

**Fragment as a four-tap offset average.** Four copies at a small fixed
`(dx, dy)` are summed and divided by four into a destination plane, so the
output is a deterministic ghost and a flat region is a bit-exact no-op. The
offset is a named constant. Alternative rejected: a general convolution, which
would obscure that this is exactly the documented four-copy average.

**Mezzotint as a per-kind seeded procedural pattern.** The output value at each
pixel is selected by `kind` (dot / line / stroke families) from a thresholded
noise field generated by the RNG; a grayscale buffer uses the luma pattern and
a color buffer keeps saturated color. The pattern families are independent
functions, so adding a `MezzotintType` is a localized change. Adobe's exact
patterns are closed, so this is behavioral parity only.

**Pointillize as seeded dot scatter over a background.** The RNG scatters dot
centers on a jittered grid, each dot's radius scales with `cell_size`, and each
inside pixel takes the local source color while everything else takes the
`background` parameter. The background is a parameter, not global state, so a
re-apply is reproducible and the undo record can store it.

**Color Halftone as per-channel rotated screening.** Each color channel is
sampled on a grid rotated by `angles[channel]`, the cell luminance sets the dot
radius, and the channel's dot is composited. Grayscale uses `angles[0]`; a
3-channel buffer uses the first three; other channel counts reuse an available
angle so no index reads out of bounds. Adobe's exact screen kernel is closed,
so this is behavioral parity only.

**Oracle split: diff Mosaic, document the rest.** ImageMagick is a sanity
oracle, not a parity oracle, exactly as in M6/M7. Mosaic is diffed against a
box downsample followed by a point upsample (the block average) within a stated
tolerance. Crystallize (Voronoi with a jittered seed set), Facet (closed pass
count), Fragment (fixed offset and 4-tap average), Mezzotint (procedural
patterns), Pointillize (dot scatter), and Color Halftone (closed screen kernel)
have no faithful operator; each is classified no-equivalent with tolerance 0, a
property or known-value test, a non-empty note, and the observed delta
recorded. The mapping table has one row per variant, so a new variant cannot be
added without classifying it.

**No committed fixtures; ImageMagick stays optional.** The differential tests
run `magick` at test time and print a skip when it is absent; no binary
references are checked in and nothing is marked `#[ignore]`.

## Risks / Trade-offs

- **Closed-kernel approximation drift.** The Facet pass count, the Mezzotint
  pattern families, the Pointillize dot geometry, and the Color Halftone screen
  kernel may diverge from CS6. Mitigation: each is named as an approximation in
  the requirements and the code; the enum and parameter shapes stay stable so a
  later change can retune one function without touching the contract.
- **Crystallize cost.** Brute-force nearest-seed assignment is
  `O(pixels · cells)`. Mitigation: the grid-bucketed search is the documented
  upgrade path behind the same signature; progress and cancellation stay a
  tile-pipeline concern, not this crate.
- **Deterministic randomness under tiling.** A tile-local RNG that reseeds per
  tile would break bit-identical output across tile splits. Mitigation: the
  contract fixes the seed as the single source of randomness; a coordinate-
  derived seed for huge PSB tiles is an M8 integration concern, recorded here.
- **Mezzotint color semantics.** "Fully saturated color patterns" is
  underspecified in the brief. Mitigation: the requirement fixes the observable
  properties (distinct per kind, seed-reproducible, color keeps saturation) so
  the exact mapping can be tuned without a contract change.
- **Color Halftone channel-count degradation.** Grayscale and RGB are pinned;
  other channel counts are allowed to reuse an angle. Mitigation: the
  out-of-bounds-read is stated as forbidden and tested on a grayscale buffer.
- **Seed determinism across platforms.** Float accumulation in Crystallize and
  Pointillize could vary. Mitigation: means accumulate in integer sums and the
  seeded RNG is the only nondeterministic-looking input, keeping results
  bit-identical.
- **8-bit only.** The contract assumes 8-bit planar planes. A 16/32-bit or
  CMYK/Lab port needs wider samples and non-RGB planes; it is deferred, not
  accidental.
