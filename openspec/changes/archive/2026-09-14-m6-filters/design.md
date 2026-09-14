## Context

M4 built `crates/pictura-adjust` as the CPU reference for the destructive
`Image > Adjustments` math: one enum plus `apply`, in-place planar 8-bit writes,
alpha untouched, `AdjustError` instead of panics, and an ImageMagick differential
oracle for the handful of operators whose semantics match. M6 repeats that shape
for the Filter menu. `docs/dev/m6-filters.md` freezes the contract (the `Filter`
enum, the per-family function signatures, the kernel and luma helpers, the oracle
split, and the seed rule) and `docs/06-filters/filters-overview.md` plus the
per-family specs describe the CS6 behavior and parameter ranges.

The code does not exist yet: M6 is a forward-looking milestone. This change
records WHAT the crate must do and HOW it is structured, so M6-A through M6-F can
be implemented against reviewable requirements. Adobe's kernels are closed, so
parity is behavioral only where an oracle exists and approximations are marked
as such.

## Goals / Non-Goals

**Goals:**

- Freeze the `apply` signature, the `Filter` enum, the support enums
  (`RadialMethod`, `Quality`, `NoiseDistribution`), and `FilterError`.
- Provide the Blur, Sharpen, and Noise families as pure functions over a planar
  8-bit `PixelBuffer`, with the shared helpers in `kernel.rs` and `luma.rs`.
- Record the parameter ranges, defaults, and algorithm for each filter, and the
  shared invariants: in-place application, alpha untouched, clamp-to-edge,
  tiny-image safety, deterministic output, errors instead of panics.
- Record the ImageMagick oracle split: which filters are diffed, at what
  tolerance, and which are documented as no-equivalent with property or
  known-value tests.
- Make Add Noise reproducible: the seed is part of the filter, so the same seed
  is bit-identical and a re-apply is reproducible.

**Non-Goals:**

- Lens Blur, Shape Blur, Smart Blur, and the entire Blur Gallery.
- Smart Sharpen (`Remove = Lens|Motion`, `More Accurate`) and its Advanced
  Shadow / Highlight tabs. M6 ships the fixed kernels and Unsharp Mask only.
- Dust & Scratches and Reduce Noise.
- 16-bit and 32-bit math; M6 is 8-bit only.
- GPU compute. `pictura-filters` is the CPU reference; the CPU result is the
  oracle for any later accelerator.
- Selection / mask composition, render or app wiring, Smart Filters, and the
  Filter Gallery. Those are a later M6-C integration wave.
- CMYK / Lab. M6 operates on the RGB(A) planes.

## Decisions

**New `pictura-filters` crate mirroring `pictura-adjust`.** The library depends
on `pictura-core` for `PixelBuffer` and `thiserror` for the error enum;
`pictura-testkit` is a dev-dependency for byte comparison. Add Noise adds
`rand_chacha` as the seeded RNG, the only new external crate. Keeping the crate
shape identical to `pictura-adjust` (one enum, one `apply`, in-place planar
writes, typed errors) means the two destructive-command paths read the same and
a later integration can dispatch either behind one command handler. Alternative
rejected: folding the filters into `pictura-adjust`, because a filter is a
neighborhood operator over tiles, not a per-pixel adjustment, and the oracle and
kernel helpers are shared across blur and sharpen rather than across
adjustments.

**One `Filter` enum plus `apply` dispatch.** `apply(&Filter, &mut PixelBuffer)`
validates the buffer and channel count once, then dispatches to the family
function. Family functions (`blur::gaussian`, `sharpen::unsharp_mask`,
`noise::median`, ...) are public so tests and a later tile pipeline can call
them directly. `FilterError` carries `Unsupported` for channel counts and
`InvalidParams` for values and buffer shapes. This is the same error-not-panic
contract as M4 and is what keeps a malformed buffer from taking down the host.

**Separable FIR Gaussian as the shared kernel.** Gaussian blur is implemented as
two 1-D passes (horizontal then vertical) with a normalized kernel of support
`⌈3σ⌉` samples each side, where the UI radius is the 3σ support
(`sigma_from_radius(r) = max(r / 3, 0.1)`). Unsharp Mask reuses the same
`gaussian_kernel`, so the sharpen family has no second blur implementation. The
separable FIR is the correctness baseline; a box-blur approximation or an IIR
recursion for very large radii is a later optimization. Alternative rejected:
three box blurs by default, because the FIR result is what the oracle can pin.

**Clamp-to-edge border policy.** `clamp_index(i, n)` maps any out-of-range
sample index to the nearest edge sample. This is the border policy for every
neighborhood filter, so `radius` larger than the image degrades gracefully and
the 1×1 / 1-px cases cannot index out of bounds. The docs mark Photoshop's exact
border behavior unverified; clamp-to-edge is the proposed policy and is stated
as a deliberate, testable choice.

**Seeded RNG for Add Noise only.** Add Noise carries `seed: u64` in the filter
and drives a `rand_chacha` generator from it. The same seed produces
bit-identical output; a different seed produces different speckle. No other
filter touches the RNG, so every other output is a pure function of the input.
This is a design choice: Adobe does not document its redo behavior, and storing
the seed is what makes a re-apply reproducible. Alternative rejected: seeding
from a global counter, which would make output depend on call order.

**Oracle split: diff where semantics match, document where they do not.**
ImageMagick is a sanity oracle, not a parity oracle, exactly as in M4. Gaussian
(`-gaussian-blur 0xσ`), Box (`-statistic mean NxN`), Motion
(`-motion-blur 0xN+angle`), Median (`-median R`), and Unsharp Mask
(`-unsharp 0xRxA+T`) are diffed with a small per-sample tolerance. Average,
Radial, Surface, Despeckle, Add Noise, Sharpen / Sharpen More / Sharpen Edges,
and Blur / Blur More have no faithful operator (region mean; differing radial
semantics; closed bilateral / detector / fixed kernels; differing RNG streams)
and use tolerance 0 with property or known-value tests. The mapping table has one
row per `Filter` variant, so a new variant cannot be added without classifying it.

**No committed fixtures; ImageMagick is optional.** The differential tests run
`magick` at test time and print a skip when it is not on `PATH`; no binary
references are checked in and nothing is marked `#[ignore]`.

## Risks / Trade-offs

- **Closed-kernel approximation drift.** The bilateral Surface kernel, the fixed
  Sharpen gains, Despeckle's detector, and the radial samplers may diverge from
  CS6. Mitigation: each is named as an approximation in the requirements and the
  code, and the enum / parameter shapes stay stable so a later change can retune
  one function without touching the contract.
- **ImageMagick version and kernel sensitivity.** The operator set and its
  radius→sigma / window conventions can change between builds and may not match
  our radius mapping. Mitigation: tolerances are small but explicit per filter,
  the verified version is recorded, and the no-equivalent list bounds how much
  can regress silently.
- **Radius↔σ divergence.** Adobe's mapping from the UI radius to the Gaussian σ
  is undocumented; `radius / 3` is a proposal. Mitigation: it lives in one
  function (`sigma_from_radius`) and is floored at 0.1, so a measurement can
  retune it in one place.
- **Large-radius cost.** A direct separable FIR is `O(N·r)`; very large radii on
  huge documents will be slow. Mitigation: the FIR is the correctness baseline;
  the box-approx / IIR upgrade path is named and can replace the inner loop
  without changing the public contract.
- **Add Noise touches every pixel.** A full-layer Add Noise is a large history
  state and a per-call RNG draw. Mitigation: the seed makes it reproducible and
  the later undo/tile-diff budget owns the cost, not this crate.
- **8-bit only.** The contract assumes 8-bit planar planes. A 16/32-bit port
  needs wider samples and float accumulation; it is deferred, not accidental.
