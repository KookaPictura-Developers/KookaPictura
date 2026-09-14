## 1. M9-A1 — Radial warps (Twirl, Pinch, Spherize) and tests

- [ ] 1.1 Add the shared bilinear inverse-mapping helper in `distort/mod.rs`, promoting the existing `blur::bilinear` sampler to `pub(crate)` and adding a wrap branch, so Twirl, Pinch, Spherize, Ripple, and Wave sample through one resampler
- [ ] 1.2 Implement `distort::twirl` as an angular inverse-mapping warp whose rotation decays from center to edge with the `angle` sign setting direction, `angle` `0.0` a no-op, and `angle` within `-999.0..=999.0` rejected with `FilterError::InvalidParams` outside the range or when non-finite
- [ ] 1.3 Implement `distort::pinch` as a monotone radial remap that contracts toward the center for a positive `amount` and expands for a negative `amount`, `amount` `0.0` a no-op, and `amount` within `-100.0..=100.0` rejected outside the range or when non-finite
- [ ] 1.4 Implement `distort::spherize` as a radial sphere-wrap remap masked by `SpherizeMode::Normal / HorizontalOnly / VerticalOnly`, `amount` `0.0` a no-op, and `amount` within `-100.0..=100.0` rejected outside the range or when non-finite
- [ ] 1.5 Unit-test Twirl: the center rotates more than the edge, `+angle` and `-angle` rotate in opposite directions, `angle` `0.0` is a no-op, and out-of-range / non-finite angles are rejected
- [ ] 1.6 Unit-test Pinch: a positive amount contracts toward the center, a negative amount expands, `amount` `0.0` is a no-op, and out-of-range / non-finite amounts are rejected
- [ ] 1.7 Unit-test Spherize: `HorizontalOnly` displaces one axis, `VerticalOnly` the other, `Normal` both, `amount` `0.0` is a no-op, and out-of-range / non-finite amounts are rejected
- [ ] 1.8 Unit-test bilinear resampling: a fractional source coordinate equals the four-sample blend within 1 LSB, an integral coordinate returns the sample exactly, and no out-of-image read occurs

## 2. M9-A2 — Undulating warps (Ripple, Wave) and tests

- [ ] 2.1 Implement `distort::ripple` as a periodic sinusoidal displacement whose magnitude scales with `amount` and whose spatial frequency comes from `RippleSize::Small / Medium / Large`, `amount` `0.0` a no-op, and `amount` within `-999.0..=999.0` rejected outside the range or when non-finite
- [ ] 2.2 Implement `distort::wave` as the sum of `generators` seeded generators, each with a wavelength and amplitude drawn from its range via `ChaCha8Rng::seed_from_u64(seed)`, a `WaveType::Sine / Triangle / Square` shape, axis-wise `scale`, and `repeat_edge` selecting repeat-edge versus wrap
- [ ] 2.3 Validate the Wave parameters before writing: `generators` within `1..=999`, each `wavelength` / `amplitude` min within `1.0..=998.0` with max at least `min + 1.0`, each `scale` within `1.0..=100.0`, and every non-finite value rejected with `FilterError::InvalidParams`
- [ ] 2.4 Unit-test Ripple: `amount` `0.0` is a no-op, a larger amount increases the maximum displacement, `RippleSize` changes the spatial frequency, and out-of-range / non-finite amounts are rejected
- [ ] 2.5 Unit-test Wave: the same seed is bit-identical, a different seed may differ, the generator count changes the field, and each `WaveType` produces a distinct field
- [ ] 2.6 Unit-test Wave validation and edge mode: `generators` 0 / 1000, an inverted `wavelength` or `amplitude` range, and `scale` 0 / 101 are rejected; `repeat_edge` `true` and `false` differ where the displacement leaves the image, with the wrap result equal to an opposite-edge sample
- [ ] 2.7 Unit-test alpha preservation, clamp-to-edge, and 1×1 / 1-px / degenerate-radius safety for every Distort variant
- [ ] 2.8 Unit-test that every Distort variant is repeatable: Twirl / Pinch / Spherize / Ripple bit-identical across runs, Wave bit-identical for a fixed seed

## 3. M9-B — ImageMagick oracle and no-equivalent table

- [ ] 3.1 Add one mapping-table row per Distort `Filter` variant in `crates/pictura-filters/tests/oracle.rs` and classify Twirl, Pinch, Spherize, Ripple, and Wave as no-equivalent with tolerance 0
- [ ] 3.2 Add a property or known-value test per Distort variant that holds independently of ImageMagick, and record the observed delta against the closest operator (`-swirl`, `-implode` / `-explode`, `-wave`) with a non-empty no-equivalent note
- [ ] 3.3 Skip the differential tests with a message when `magick` is absent and add no `#[ignore]`
- [ ] 3.4 Document the mapping, the exact ImageMagick operators and flags, the measured observed deltas, and the verified ImageMagick version in `crates/pictura-filters/tests/README.md`

## 4. M9-C — App filter kinds and unit test

- [ ] 4.1 Register the Distort filter kinds (Twirl, Pinch, Spherize, Ripple, Wave) in the app filter menu with their parameter descriptors and the `SpherizeMode` / `RippleSize` / `WaveType` options, including a seed-carrying Wave kind and a `repeat_edge` option
- [ ] 4.2 Add an app unit test asserting each new kind maps to the matching `pictura-filters` variant and parameter set

## 5. M9-D — OpenSpec change, reconcile, and verify

- [ ] 5.1 Validate `openspec validate m9-distort --strict` and `openspec validate --all --strict` green
- [ ] 5.2 Run `cargo test --workspace` green with the per-filter unit tests and the recorded no-equivalent deltas
- [ ] 5.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 5.4 Run `scripts/guard.sh` green (no `.8bf`, no artboards, no unmarked `docs/` edits)
- [ ] 5.5 Reconcile `docs/dev/m9-distort.md` against the shipped `Filter` variants and record any divergence
