## Why

M6–M8 shipped Blur, Sharpen, Noise, Other, Stylize, and Pixelate, but the
Filter menu's geometric **Distort** warps are still stubs that return
`FilterError::Unsupported`, described only in `docs/`. `docs/dev/m9-distort.md`
freezes the M9 contract — the five 8-bit geometric warps as pure functions over
the same planar `PixelBuffer`, with bilinear inverse-mapping, untouched alpha, a
seeded RNG for Wave, and repeat-edge vs wrap undefined-area handling — so the
dialogs, the Smart Filter stack, and a later integration wave have a reviewable
target.

## What Changes

- Fill the five **Distort** stubs (`FILT-040`) in `pictura-filters`: Twirl,
  Pinch, Spherize, Ripple, and Wave, behind the existing `Filter` variants and
  the `src/distort/{radial,undulate}.rs` signatures.
- Twirl: angular inverse-mapping warp; the rotation falls off from center to
  edge and the sign of `angle` sets direction; `angle` is within `-999..=999`.
- Pinch: radial inverse-mapping warp that squeezes toward (positive) or expands
  away from (negative) the selection center; `amount` is within `-100..=100`.
- Spherize: radial inverse-mapping warp that 3D-wraps around a sphere, with
  `SpherizeMode::Normal / HorizontalOnly / VerticalOnly`; `amount` is within
  `-100..=100`.
- Ripple: sinusoidal inverse-mapping displacement; `amount` sets magnitude
  within `-999..=999` and `RippleSize` sets spatial frequency.
- Wave: a sum of `N` seeded wave generators, each with a wavelength and an
  amplitude drawn from its range and a `WaveType` (Sine / Triangle / Square);
  `scale` applies axis-wise and `repeat_edge` selects the undefined-area mode.
- Shared: bilinear inverse-mapping resampling, alpha (channel 4) untouched,
  clamp-to-edge borders, 1×1 and 1-px images never panic, `FilterError` instead
  of panics for malformed buffers and bad parameters, and bit-identical output
  for a repeated apply with the same seed.
- Parameter validation per the brief: Twirl `angle` `-999..=999`; Pinch and
  Spherize `amount` `-100..=100`; Ripple `amount` `-999..=999`; Wave
  `generators` `1..=999`, `wavelength` and `amplitude` each with min `1..=998`
  and `max >= min + 1`, and each `scale` `1..=100`; out-of-range or non-finite
  values are rejected with `FilterError::InvalidParams`.
- `crates/pictura-filters/tests/oracle.rs` and `tests/README.md` gain the
  Distort mapping rows: all five are classified no-equivalent (Adobe's warp
  kernels and falloff curves are closed) with tolerance 0, a property or
  known-value test, a non-empty note, and the observed delta against the
  closest ImageMagick operator recorded.
- The app filter kinds and their unit test ship (M9-C).
- Out of scope (later): Diffuse Glow, Displace, Glass, Ocean Ripple, Polar
  Coordinates, Shear, ZigZag, Lens Correction, Filter-Gallery stacking, Smart
  Filters, 16/32-bit, and CMYK/Lab.

## Capabilities

### New Capabilities

- `distort-filters`: the destructive geometric Distort family in
  `pictura-filters` — Twirl, Pinch, Spherize, Ripple, and Wave — including their
  parameter ranges and validation, the shared bilinear inverse-mapping warp,
  the angular / radial / spherical / sinusoidal / multi-generator displacement
  fields, repeat-edge vs wrap undefined-area handling, alpha preservation,
  seeded determinism for Wave, and the ImageMagick correspondence or documented
  no-equivalent classification.

### Modified Capabilities

None. The M6/M7/M8 filter capabilities (`blur-filters`, `sharpen-filters`,
`noise-filters`, `other-filters`, `stylize-filters`, `pixelate-filters`) are
untouched; this change adds bodies to the existing `Filter::Twirl` / `Pinch` /
`Spherize` / `Ripple` / `Wave` variants without changing their requirements.

## Impact

- `crates/pictura-filters/src/distort/radial.rs`: `twirl`, `pinch`, `spherize`.
- `crates/pictura-filters/src/distort/undulate.rs`: `ripple`, `wave`.
- `crates/pictura-filters/src/distort/mod.rs`: the shared bilinear inverse-mapping
  helper and the existing `radial` / `undulate` re-exports.
- `crates/pictura-filters/src/lib.rs`: the `Filter` variants, the `SpherizeMode`
  / `RippleSize` / `WaveType` enums, and the `apply` dispatch already exist; M9
  fills the stub bodies, so the enum shape is unchanged.
- `crates/pictura-filters/tests/oracle.rs` and `tests/README.md`: the five
  Distort mapping rows and the recorded observed deltas.
- `crates/pictura-app/**`: the Distort filter kinds and their unit test (M9-C).
- Dependencies: none new. Reuses `pictura-core`, the seeded `ChaCha8Rng` already
  used by the Noise and Pixelate families, and a single bilinear sampler shared
  with the existing warp path.
- Follows `docs/dev/m9-distort.md`, `docs/06-filters/distort-filters.md`
  (`FILT-040`), and `docs/06-filters/filters-overview.md`; those specs are not
  modified.
