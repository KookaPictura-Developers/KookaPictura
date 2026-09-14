## Why

The Filter menu needs working destructive Blur / Sharpen / Noise math before the
filter dialogs, Smart Filters, and the Filter Gallery can be wired to it, and
before a later M6-C integration wave. `docs/dev/m6-filters.md` freezes the M6
contract (pure functions over a planar 8-bit `PixelBuffer`, with an independent
ImageMagick oracle where semantics actually match), but nothing in OpenSpec pins
it. This change records that contract as reviewable requirements.

## What Changes

- New `pictura-filters` crate exposes
  `apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError>` over
  a planar 8-bit `PixelBuffer` with 3 or 4 channels, mirroring `pictura-adjust`
  (one enum plus `apply`, `FilterError` instead of panics).
- Blur family ships: Gaussian, Box, Motion, Radial (Spin / Zoom), Average,
  Blur / Blur More, and Surface (bilateral).
- Sharpen family ships: Sharpen / Sharpen More / Sharpen Edges (fixed 3×3
  high-pass) and Unsharp Mask (amount / radius / threshold, blurred-difference,
  reusing the Gaussian kernel).
- Noise family ships: Add Noise (Uniform / Gaussian, monochromatic, seeded),
  Median (rank filter), and Despeckle (edge-gated smoothing).
- Shared invariants: planar in-place application, alpha (channel 4) untouched,
  clamp-to-edge border sampling, 1×1 and 1-px images never panic, and
  `FilterError` instead of panics for malformed buffers and bad parameters.
- Add Noise is the only random filter. A seeded RNG makes the same seed
  bit-identical and a re-apply reproducible; no other filter uses randomness.
- `scripts/filter_oracle.py` plus `crates/pictura-filters/tests/oracle.rs` diff
  the filters that have a faithful ImageMagick operator (Gaussian, Box, Motion,
  Median, Unsharp Mask). Average, Radial, Surface, Despeckle, Add Noise, the
  fixed Sharpen variants, and Blur / Blur More have no faithful equivalent and
  are covered by property / known-value tests, with the divergence documented.
- Out of scope (later): Lens / Shape / Smart Blur and the Blur Gallery; Smart
  Sharpen `Remove = Lens|Motion` and Advanced Shadow/Highlight; Dust &
  Scratches; Reduce Noise; 16/32-bit math; GPU; selection/mask and render/app
  wiring; CMYK/Lab.

## Capabilities

### New Capabilities

- `blur-filters`: the destructive Blur family in `pictura-filters` — Gaussian,
  Box, Motion, Radial (Spin / Zoom), Average, Blur / Blur More, and Surface
  (bilateral) — including their parameters and ranges, the separable / kernel
  algorithms, clamp-to-edge borders, deterministic output, and the ImageMagick
  correspondence or documented no-equivalent classification.
- `sharpen-filters`: the destructive Sharpen family — Sharpen / Sharpen More /
  Sharpen Edges (fixed 3×3 high-pass) and Unsharp Mask (amount / radius /
  threshold blurred-difference reusing the Gaussian kernel) — including
  parameter ranges, the USM `-unsharp` oracle, and the known-value tests for the
  fixed kernels.
- `noise-filters`: the destructive Noise family — Add Noise (Uniform / Gaussian,
  monochromatic, seeded), Median (rank filter), and Despeckle (edge-gated
  smoothing) — including the seeded-RNG determinism contract, the `-median`
  oracle, statistical / same-seed tests for Add Noise, and the documented
  no-equivalent for Despeckle.

### Modified Capabilities

None. No existing capability has requirements yet.

## Impact

- `crates/pictura-filters` (new workspace member): `Cargo.toml`, `src/lib.rs`,
  `src/kernel.rs`, `src/luma.rs`, `src/blur.rs`, `src/sharpen.rs`, `src/noise.rs`,
  and `tests/oracle.rs`; the workspace `Cargo.toml` gains the member.
- `scripts/filter_oracle.py`, a test-time helper that drives ImageMagick.
- Dependencies: `pictura-core` and `thiserror` (workspace) for the library;
  `pictura-testkit` as a dev-dependency. Add Noise's seeded RNG adds
  `rand_chacha` (with its `rand_core`), the only new external crate, per the
  determinism requirement in the M6 brief.
- ImageMagick is optional. Differential tests skip with a message when `magick`
  is not on `PATH`; nothing is marked `#[ignore]`.
- Follows `docs/dev/m6-filters.md` and `docs/06-filters/{filters-overview,
  blur-filters,sharpen-filters,noise-filters}.md`; those specs are not modified.
