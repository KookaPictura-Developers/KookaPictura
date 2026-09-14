## Why

M6 shipped the Blur, Sharpen, and Noise filter families, but the Filter menu's
**Other** and **Stylize** submenus are still unimplemented and described only in
`docs/`. `docs/dev/m7-stylize-other.md` freezes the M7 contract — five Other
filters and three Stylize filters as pure functions over the same planar 8-bit
`PixelBuffer`, with an independent ImageMagick oracle where the semantics
actually match — so the dialogs, the Smart Filter stack, and a later
integration wave have a reviewable target.

## What Changes

- Add five **Other** family filters to `pictura-filters`: Maximum, Minimum,
  Offset, High Pass, and Custom, with the exact `Filter` variants and module
  signatures frozen in `docs/dev/m7-stylize-other.md`.
- Add three **Stylize** family filters: Emboss, Find Edges, and Solarize. Find
  Edges and Solarize take no parameters and apply immediately.
- Maximum / Minimum: per-color-channel `(2r+1)²` dilation / erosion; `radius`
  0 is a no-op and `radius` is clamped to 100 (the `FILT-070` scriptable max).
- Offset: horizontal / vertical translation with Wrap Around or Set To
  Background fill.
- High Pass: `clamp(orig − gaussian(orig, σ) + 128)` per channel with a finite,
  positive radius, reusing the M6 Gaussian kernel.
- Custom: a 5×5 convolution with `scale` / `offset` and f64 accumulation; the
  identity kernel with `scale 1`, `offset 0` is an exact pass-through; `scale ==
  0` and non-finite parameters are rejected.
- Emboss: directional relief from `angle` / `height` / `amount`, rendered gray.
- Find Edges: Sobel gradient magnitude rendered dark on light as
  `255 − clamp(magnitude)`.
- Solarize: the fixed 50% curve `out = if v >= 128 { 255 − v } else { v }`.
- Shared invariants: planar in-place application, alpha (channel 4) untouched,
  clamp-to-edge border sampling, 1×1 and 1-px images never panic, `FilterError`
  instead of panics for malformed buffers and bad parameters, deterministic
  output.
- `scripts/filter_oracle.py` gains Maximum / Minimum (`-morphology Dilate` /
  `Erode Square:N`), Offset wrap (`-roll`), Custom (`-convolve`), Emboss
  (`-emboss`), Find Edges (`-edge`), and Solarize (`-solarize 50%`) operators;
  `crates/pictura-filters/tests/oracle.rs` diffs the faithful operators within
  tolerance and records the no-equivalent rows (Offset background fill, High
  Pass, and Find Edges) with the divergence documented.
- The app filter kinds and their unit test ship (M7-D).
- Out of scope (later): Stylize Wind / Tiles / Extrude / Trace Contour, Oil
  Paint, Diffuse, Glowing Edges; Distort (geometric warps), Pixelate, Render,
  Liquify, Blur Gallery, Camera Raw, Lens Correction; Smart Filters; 16/32-bit;
  CMYK/Lab. The optional HSB/HSL plug-in is also out of scope.

## Capabilities

### New Capabilities

- `other-filters`: the destructive Other family in `pictura-filters` —
  Maximum, Minimum, Offset, High Pass, and Custom — including their parameter
  ranges and validation, the morphological / convolution / band-split
  algorithms, clamp-to-edge borders, alpha preservation, deterministic output,
  and the ImageMagick correspondence or documented no-equivalent classification.
- `stylize-filters`: the destructive Stylize family in `pictura-filters` —
  Emboss, Find Edges (no parameters), and Solarize (no parameters) — including
  the parameter ranges and validation for Emboss, the gray-relief / Sobel /
  solarization algorithms, clamp-to-edge borders, alpha preservation,
  deterministic output, and the ImageMagick correspondence or documented
  no-equivalent classification.

### Modified Capabilities

None. The M6 filter capabilities (`blur-filters`, `sharpen-filters`,
`noise-filters`) are untouched; this change adds new variants to the existing
`Filter` enum without changing M6 requirements.

## Impact

- `crates/pictura-filters/src/other.rs` (new): `maximum`, `minimum`, `offset`,
  `high_pass`, `custom`.
- `crates/pictura-filters/src/stylize.rs` (new): `emboss`, `find_edges`,
  `solarize`.
- `crates/pictura-filters/src/lib.rs`: the new `Filter` variants and their
  `apply` dispatch.
- `crates/pictura-filters/tests/oracle.rs` and `tests/README.md`: the new
  differential rows and the one-row-per-variant mapping table.
- `scripts/filter_oracle.py`: the new ImageMagick operators.
- `crates/pictura-app/**`: the Other and Stylize filter kinds and their unit
  test (M7-D).
- Dependencies: none new. High Pass reuses the M6 Gaussian kernel; the crate
  keeps `pictura-core`, `thiserror`, and the existing test tooling.
- Follows `docs/dev/m7-stylize-other.md`,
  `docs/06-filters/other-filters.md` (`FILT-070`),
  `docs/06-filters/stylize-filters.md` (`FILT-050`), and
  `docs/06-filters/filters-overview.md`; those specs are not modified.
