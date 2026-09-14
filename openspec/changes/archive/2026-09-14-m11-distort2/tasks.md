## 1. M11-A1 — Coordinate transforms (Polar Coordinates, Shear) and tests

- [x] 1.1 Implement `distort::polar_coordinates` as a direct coordinate reinterpretation: `PolarKind::RectangularToPolar` maps each destination pixel's rectangular position to a source polar coordinate and `PolarKind::PolarToRectangular` the reverse, both through the shared bilinear inverse-mapping sampler with clamp-to-edge, so the pair round-trips within resampling tolerance and neither direction panics on a 1×1 or 1-px image
- [x] 1.2 Implement `distort::shear` as a piecewise-linear vertical column shift driven by the `curve` control points in normalized `-1..=1` space, with the source coordinate sampled through the shared resampler and a flat curve a no-op
- [x] 1.3 Validate the Shear `curve` before writing: at least two finite points, each `x` and `y` within `-1.0..=1.0`, and `x` strictly increasing, otherwise `FilterError::InvalidParams` with the buffer unchanged
- [x] 1.4 Honor `ShearFill` for rows shifted off-canvas: `WrapAround` samples the opposite edge and `RepeatEdgePixels` repeats the nearest edge, and the two modes are observably different where displacement leaves the image
- [x] 1.5 Unit-test Polar Coordinates: `RectangularToPolar` changes the arrangement, the two directions differ from each other, the transform pair round-trips within the recorded tolerance, and tiny / thin images do not panic
- [x] 1.6 Unit-test Shear: a flat curve is a no-op, a curved control-point set displaces columns vertically, an invalid curve is rejected, and `WrapAround` versus `RepeatEdgePixels` differ off-canvas
- [x] 1.7 Unit-test that Polar Coordinates and Shear preserve alpha bit-identically and never read outside the image at the borders

## 2. M11-A2 — Ripple warps (ZigZag, Ocean Ripple) and tests

- [x] 2.1 Implement `distort::zigzag` as a radial displacement whose magnitude scales with `amount`, whose direction-reversal count from center to edge is set by `ridges`, and whose geometry is selected by `ZigZagStyle::AroundCenter / OutFromCenter / PondRipples`, through the shared bilinear inverse-mapping sampler with clamp-to-edge
- [x] 2.2 Validate ZigZag before writing: `amount` within `-100.0..=100.0`, `ridges` within `0..=20`, and any non-finite `amount` rejected with `FilterError::InvalidParams`; `amount` `0.0` is a no-op and `ridges` `0` is a single direction
- [x] 2.3 Implement `distort::ocean_ripple` as a sum of seeded random ripples whose frequency comes from `size` and amplitude from `magnitude`, using `ChaCha8Rng::seed_from_u64(seed)`, through the shared sampler with clamp-to-edge
- [x] 2.4 Validate Ocean Ripple before writing: `size` within `1..=15` and `magnitude` within `0..=20`, otherwise `FilterError::InvalidParams` with the buffer unchanged; `magnitude` `0` is a no-op and the same seed is bit-identical
- [x] 2.5 Unit-test ZigZag: `amount` `0.0` is a no-op, a larger `ridges` count changes the field, each `ZigZagStyle` produces a distinct field, and out-of-range / non-finite parameters are rejected
- [x] 2.6 Unit-test Ocean Ripple: the same seed is bit-identical, a different seed may differ, `size` changes the spatial frequency, `magnitude` `0` is a no-op, and out-of-range parameters are rejected
- [x] 2.7 Unit-test alpha preservation, clamp-to-edge, and 1×1 / 1-px safety for ZigZag and Ocean Ripple
- [x] 2.8 Unit-test determinism across the family: Polar Coordinates, Shear, and ZigZag bit-identical across runs, and Ocean Ripple bit-identical for a fixed seed

## 3. M11-B — ImageMagick oracle and no-equivalent table

- [x] 3.1 Add one mapping-table row per new Distort `Filter` variant in `crates/pictura-filters/tests/oracle.rs`, naming the closest operator for each (`-distort DePolar` / `-distort Polar`, `-shear`, `-swirl`, `-wave`)
- [x] 3.2 Classify ZigZag and Ocean Ripple as no-equivalent with tolerance 0 and a property or known-value test, recording the observed delta against the closest operator with a non-empty note
- [x] 3.3 Diff Polar Coordinates against `-distort DePolar` / `-distort Polar` and Shear against `-shear`, record the measured maximum and mean per-sample delta, justify any tolerance from that measurement, and classify a case that cannot be matched as no-equivalent with the delta recorded
- [x] 3.4 Extend `scripts/filter_oracle.py` with the Polar Coordinates, Shear, ZigZag, and Ocean Ripple cases
- [x] 3.5 Skip the differential tests with a message when `magick` is absent and add no `#[ignore]`
- [x] 3.6 Document the mapping, the exact ImageMagick operators and flags, the measured observed deltas, and the verified ImageMagick version in `crates/pictura-filters/tests/README.md`

## 4. M11-C — App filter kinds and unit test

- [x] 4.1 Register the four Distort filter kinds (Polar Coordinates, Shear, ZigZag, Ocean Ripple) in the app filter menu with their parameter descriptors and the `PolarKind` / `ShearFill` / `ZigZagStyle` options, including the Shear control-point curve and a seed-carrying Ocean Ripple kind
- [x] 4.2 Add an app unit test asserting each new kind maps to the matching `pictura-filters` variant and parameter set

## 5. M11-D — OpenSpec change, reconcile, and verify

- [x] 5.1 Validate `openspec validate m11-distort2 --strict` and `openspec validate --all --strict` green
- [x] 5.2 Run `cargo test --workspace` green with the per-filter unit tests and the recorded oracle deltas
- [x] 5.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] 5.4 Run `scripts/guard.sh` green (no `.8bf`, no artboards, no unmarked `docs/` edits)
- [x] 5.5 Reconcile `docs/dev/m11-distort2.md` against the shipped `Filter` variants and record any divergence
