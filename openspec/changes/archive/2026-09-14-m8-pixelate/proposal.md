## Why

M6 shipped Blur, Sharpen, and Noise; M7 added Other and Stylize, but the Filter
menu's **Pixelate** submenu is still unimplemented and described only in
`docs/`. `docs/dev/m8-pixelate.md` freezes the M8 contract — seven Pixelate
filters as pure functions over the same planar 8-bit `PixelBuffer`, with
clamp-to-edge sampling, untouched alpha, and a seeded RNG for the stochastic
members — so the dialogs, the Smart Filter stack, and a later integration wave
have a reviewable target.

## What Changes

- Add the seven **Pixelate** family filters (`FILT-084`) to `pictura-filters`:
  Color Halftone, Crystallize, Facet, Fragment, Mezzotint, Mosaic, and
  Pointillize, with the exact `Filter` variants and `src/pixelate.rs`
  signatures frozen in `docs/dev/m8-pixelate.md`.
- Mosaic: average each `cell_size × cell_size` block and write the mean to every
  pixel in it.
- Crystallize: Voronoi tessellation — scatter one seeded, jittered point per
  `cell_size` grid cell, assign each pixel to its nearest seed, and fill the
  cell with the mean color of its members.
- Facet: a fixed number of iterative similar-neighbor local-averaging passes
  that flatten detail into patches.
- Fragment: four copies offset by a small fixed `(dx, dy)` and averaged
  (deterministic ghost). No parameters.
- Mezzotint: a per-`kind` seeded procedural dot / line / stroke pattern over
  the image; grayscale uses the luma pattern and color keeps saturated color.
- Pointillize: scattered dots whose radius is proportional to `cell_size`, each
  filled with the local source color, over the supplied `background`.
- Color Halftone: per color channel, a rotated grid at `angles[channel]`
  (grayscale uses `angles[0]`; a 3-channel image uses the first three) with the
  dot radius proportional to cell brightness and grid spacing from
  `max_radius`.
- Parameter validation per the brief: Mosaic `cell_size` 2..=200, Crystallize
  `cell_size` 3..=300, Pointillize `cell_size` 3..=300, Color Halftone
  `max_radius` 4..=127 with every angle finite; out-of-range or non-finite
  values are rejected with `FilterError::InvalidParams`.
- Shared invariants: planar in-place application, alpha (channel 4) untouched,
  clamp-to-edge border sampling, 1×1 and 1-px images never panic, `FilterError`
  instead of panics for malformed buffers and bad parameters, and bit-identical
  output for a repeated apply with the same seed.
- `scripts/filter_oracle.py` gains the Mosaic block-average operator (box
  downsample then point upsample); `crates/pictura-filters/tests/oracle.rs`
  diffs Mosaic within tolerance and records the no-equivalent rows
  (Crystallize, Facet, Fragment, Mezzotint, Pointillize, Color Halftone) with
  the observed delta documented.
- The app filter kinds and their unit test ship (M8-C).
- Out of scope (later): Distort, Render, Liquify, Blur Gallery, Camera Raw,
  Lens Correction; Smart Filters; 16/32-bit; CMYK/Lab; and the Filter-Gallery
  stack — **Pixelate is not a gallery category in CS6**.

## Capabilities

### New Capabilities

- `pixelate-filters`: the destructive Pixelate family in `pictura-filters` —
  Color Halftone, Crystallize, Facet, Fragment, Mezzotint, Mosaic, and
  Pointillize — including their parameter ranges and validation, the
  block-mean / Voronoi / local-average / offset-average / procedural-pattern /
  dot-scatter / halftone-screen algorithms, clamp-to-edge borders, alpha
  preservation, seeded determinism, and the ImageMagick correspondence or
  documented no-equivalent classification.

### Modified Capabilities

None. The M6/M7 filter capabilities (`blur-filters`, `sharpen-filters`,
`noise-filters`, `other-filters`, `stylize-filters`) are untouched; this change
adds new variants to the existing `Filter` enum without changing their
requirements.

## Impact

- `crates/pictura-filters/src/pixelate.rs`: the seven Pixelate functions
  (`mosaic`, `crystallize`, `facet`, `fragment`, `mezzotint`, `pointillize`,
  `color_halftone`) behind the existing stubs.
- `crates/pictura-filters/src/lib.rs`: the `MezzotintType` enum, the seven
  `Filter` variants, and their `apply` dispatch.
- `crates/pictura-filters/tests/oracle.rs` and `tests/README.md`: the Mosaic
  differential row, the no-equivalent rows, and the one-row-per-variant mapping
  table.
- `scripts/filter_oracle.py`: the Mosaic block-average operator.
- `crates/pictura-app/**`: the Pixelate filter kinds and their unit test
  (M8-C).
- Dependencies: none new. The stochastic filters reuse the seeded `ChaCha8Rng`
  already used by the Noise family; the block mean reuses the shared kernel
  helpers.
- Follows `docs/dev/m8-pixelate.md`,
  `docs/06-filters/pixelate-filters.md` (`FILT-084`), and
  `docs/06-filters/filters-overview.md`; those specs are not modified.
