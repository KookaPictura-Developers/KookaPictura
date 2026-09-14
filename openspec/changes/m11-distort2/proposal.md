## Why

M9 shipped five geometric **Distort** warps, but four CS6 Distort filters are
still stubs that return `FilterError::Unsupported`: Polar Coordinates, Shear,
ZigZag, and Ocean Ripple. `docs/dev/m11-distort2.md` freezes the M11 contract —
the four remaining 8-bit inverse-mapping warps over the same planar
`PixelBuffer`, with bilinear resampling, untouched alpha, a per-column control-
point curve for Shear, a seeded RNG for Ocean Ripple, and structured
`InvalidParams` validation — so the dialogs, the Smart Filter stack, and the
oracle have a reviewable target.

## What Changes

- Fill the four remaining **Distort** stubs (`FILT-040`) in `pictura-filters`
  behind the existing `Filter` variants and the
  `src/distort/{coord,ripples}.rs` signatures.
- **Polar Coordinates**: a rectangular ⇄ polar coordinate transform
  (`PolarKind::{RectangularToPolar, PolarToRectangular}`), resampled
  bilinearly, an invertible pair up to resampling error.
- **Shear**: a piecewise-linear vertical displacement of each column from the
  control-point curve (`x`, `y` in `-1..=1`); `ShearFill::{WrapAround,
  RepeatEdgePixels}` selects the undefined-area policy for rows shifted
  off-canvas.
- **ZigZag**: a radial displacement whose magnitude scales with `amount` and
  whose reversal count from center to edge is set by `ridges`, with
  `ZigZagStyle::{AroundCenter, OutFromCenter, PondRipples}` selecting the
  direction.
- **Ocean Ripple**: a seeded random ripple displacement (`size`, `magnitude`,
  `seed`); the same seed is bit-identical.
- Shared: bilinear inverse-mapping resampling, alpha (channel 4) untouched,
  clamp-to-edge borders, 1×1 and 1-px images never panic, `FilterError` instead
  of panics for malformed buffers and bad parameters, and deterministic output
  except Ocean Ripple, which is seeded.
- Parameter validation: Shear `curve` SHALL have at least two finite points
  with strictly increasing `x`, each coordinate in `-1..=1`; ZigZag `amount`
  `-100..=100` and `ridges` `0..=20`; Ocean Ripple `size` `1..=15` and
  `magnitude` `0..=20`; out-of-range values are rejected with
  `FilterError::InvalidParams`.
- `crates/pictura-filters/tests/oracle.rs` and `tests/README.md` gain the four
  new Distort mapping rows: ZigZag and Ocean Ripple are classified
  no-equivalent with tolerance 0; Polar Coordinates and Shear are measured
  against `-distort DePolar` / `-distort Polar` and `-shear`, each with a
  recorded delta and a justified tolerance or a no-equivalent note.
- The app filter kinds and their unit test ship (M11-C).
- Out of scope (later): Diffuse Glow, Displace, Glass, Lens Correction,
  Filter-Gallery stacking, Smart Filters, 16/32-bit, and CMYK/Lab.

## Capabilities

### New Capabilities

### Modified Capabilities

- `distort-filters`: adds the Polar Coordinates, Shear, ZigZag, and Ocean
  Ripple requirements, and extends the shared application / error contract,
  alpha preservation, parameter validation, bilinear resampling,
  undefined-area edge handling, determinism, and oracle-classification
  requirements from five to nine Distort variants.

## Impact

- `crates/pictura-filters/src/distort/coord.rs`: `polar_coordinates`, `shear`.
- `crates/pictura-filters/src/distort/ripples.rs`: `zigzag`, `ocean_ripple`.
- `crates/pictura-filters/src/distort/mod.rs` and `src/lib.rs`: the `Filter`
  variants, the `PolarKind` / `ShearFill` / `ZigZagStyle` enums, the module
  split, and the `apply` dispatch already exist as the M11 scaffold; M11 fills
  the stub bodies, so the enum shape is unchanged.
- `crates/pictura-filters/tests/oracle.rs` and `tests/README.md`: the four new
  Distort mapping rows and the recorded observed deltas.
- `scripts/filter_oracle.py`: the Polar Coordinates, Shear, ZigZag, and Ocean
  Ripple oracle cases.
- `crates/pictura-app/**`: the four Distort filter kinds and their unit test
  (M11-C).
- Dependencies: none new. Reuses `pictura-core`, the seeded `ChaCha8Rng`
  already used by the Noise, Pixelate, and Wave families, and the bilinear
  inverse-mapping sampler shared with the M9 warp path.
- Follows `docs/dev/m11-distort2.md`, `docs/06-filters/distort-filters.md`
  (`FILT-040`), and `docs/06-filters/filters-overview.md`; those specs are not
  modified.
