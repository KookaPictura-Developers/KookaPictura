## 1. M6-A — Crate skeleton and shared helpers

- [ ] 1.1 Create `crates/pictura-filters` with `pictura-core` and `thiserror` dependencies, `rand_chacha` for the seeded RNG, and `pictura-testkit` as a dev-dependency; add the crate to the workspace `Cargo.toml`
- [ ] 1.2 Define `FilterError { Unsupported(String), InvalidParams(String) }` and the support enums `RadialMethod { Spin, Zoom }`, `Quality { Draft, Good, Best }`, and `NoiseDistribution { Uniform, Gaussian }`
- [ ] 1.3 Define the `Filter` enum with all M6 variants and their parameter fields per the M6 brief
- [ ] 1.4 Implement `apply(filter, buf)` dispatch plus shared `validate` for channel count (3 or 4), non-empty pixels, and `data.len() == width * height * channels`
- [ ] 1.5 Implement `kernel::clamp_index`, `kernel::sigma_from_radius` (radius / 3, σ floored at 0.1), and `kernel::gaussian_kernel` (normalized, `⌈3σ⌉` support each side)
- [ ] 1.6 Implement `luma::LUMA` (Rec.601) and `luma::luma` for the luma-preserving paths
- [ ] 1.7 Unit-test `gaussian_kernel` normalization and symmetry, `clamp_index` at both borders, and `sigma_from_radius` flooring

## 2. M6-B — Blur family

- [ ] 2.1 Implement `blur::gaussian` as separable horizontal then vertical FIR passes with clamp-to-edge
- [ ] 2.2 Implement `blur::box` as a separable `(2r+1)²` moving average
- [ ] 2.3 Implement `blur::motion` as a 1-D line convolution along `angle` with `distance` and supersampling
- [ ] 2.4 Implement `blur::radial` for Spin (angular moving average) and Zoom (radial moving average) with `Quality` sampling density
- [ ] 2.5 Implement `blur::average` as the region mean written to every color sample
- [ ] 2.6 Implement `blur::simple(more)` as the fixed 3×3 Blur / Blur More kernel with a 3–4× stronger Blur More gain
- [ ] 2.7 Implement `blur::surface` as a bilateral filter with a spatial (radius) and range (threshold) kernel
- [ ] 2.8 Unit-test each blur variant: solid-color no-op, radius/distance/amount zero or one identity, edge softening growing with radius, and validation of out-of-range parameters

## 3. M6-C — Sharpen family

- [ ] 3.1 Implement `sharpen::sharpen` and `sharpen::sharpen_more` as fixed 3×3 high-pass kernels with per-variant gain
- [ ] 3.2 Implement `sharpen::edges` as the fixed high-pass gated on a fixed gradient threshold so flat regions are unchanged
- [ ] 3.3 Implement `sharpen::unsharp_mask` as the blurred-difference `original + (original − blurred) × amount` with a threshold gate, reusing the shared Gaussian kernel
- [ ] 3.4 Unit-test the fixed kernels on known values and a step edge, and assert Sharpen More is stronger than Sharpen
- [ ] 3.5 Unit-test USM identity at amount 0 and radius 0, uniform-image no-op, overshoot proportional to amount, and the threshold gate
- [ ] 3.6 Unit-test clamp-to-edge and 1×1 / 1-px images for every sharpen variant

## 4. M6-D — Noise family

- [ ] 4.1 Implement `noise::add` drawing Uniform on `[−amount, +amount]` or Gaussian `N(0, σ)` per pixel, monochromatic (one delta per pixel) or per-channel, from a `rand_chacha` stream seeded by `seed`
- [ ] 4.2 Implement `noise::median` as a per-channel rank filter over a `(2r+1)²` window with clamp-to-edge
- [ ] 4.3 Implement `noise::despeckle` as edge detection plus smoothing of non-edge pixels
- [ ] 4.4 Unit-test Add Noise: same seed bit-identical, different seeds differ, zero-mean mean preserved, monochromatic hue preserved, and Gaussian vs Uniform histogram shape
- [ ] 4.5 Unit-test Median on salt-and-pepper noise and across a step edge, and Despeckle on isolated noise with a preserved edge
- [ ] 4.6 Unit-test clamp-to-edge, 1×1 / 1-px images, and parameter validation (Add Noise amount range, Median radius range)

## 5. M6-E — ImageMagick differential oracle

- [ ] 5.1 Add `scripts/filter_oracle.py` with `version` and `apply` subcommands and planar/interleaved raw I/O
- [ ] 5.2 Expose named operators for gaussian, statistic-mean, motion-blur, median, and unsharp plus verbatim `--im-args`
- [ ] 5.3 Add `crates/pictura-filters/tests/oracle.rs` with a small RGB oracle image and a planar round-trip
- [ ] 5.4 Diff Gaussian `-gaussian-blur 0xσ`, Box `-statistic mean NxN`, Motion `-motion-blur 0xN+angle`, Median `-median R`, and Unsharp Mask `-unsharp 0xRxA+T` within their stated tolerances
- [ ] 5.5 Add the one-row-per-`Filter`-variant mapping table and assert Average, Radial, Surface, Despeckle, Add Noise, Sharpen / Sharpen More / Sharpen Edges, and Blur / Blur More use tolerance 0 and carry a non-empty no-equivalent note
- [ ] 5.6 Skip differential tests with a message when `magick` is absent; add no `#[ignore]`
- [ ] 5.7 Document the mapping, exact flags, divergences, and the verified ImageMagick version in `crates/pictura-filters/tests/README.md`

## 6. M6-F — Integration, verification, and proposal

- [ ] 6.1 Confirm the shared invariants hold across every variant: alpha channel 4 bit-identical, malformed buffers error without panicking, and repeated runs are bit-identical
- [ ] 6.2 Run `cargo test --workspace` green with the per-filter unit tests and oracle differentials passing within tolerance
- [ ] 6.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 6.4 Run `scripts/guard.sh` green (no `.8bf`, no artboards, no unmarked `docs/` edits)
- [ ] 6.5 Run `openspec validate m6-filters --strict` and `openspec validate --all --strict` green
