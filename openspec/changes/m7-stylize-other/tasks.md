## 1. M7-A — Other family and tests

- [x] 1.1 Add the `Filter::Maximum { radius: u32 }` and `Filter::Minimum { radius: u32 }` variants and dispatch them from `apply`
- [x] 1.2 Implement `other::maximum` and `other::minimum` as a per-color-channel max/min over the `(2r+1)²` square footprint with clamp-to-edge, `radius == 0` a no-op, and `radius` clamped to 100
- [x] 1.3 Add the `Filter::Offset { horizontal: i32, vertical: i32, wrap: bool, background: [u8; 3] }` variant and implement `other::offset` with modulo wrap or background fill, `(0, 0)` a no-op, alpha untouched
- [x] 1.4 Add the `Filter::HighPass { radius: f64 }` variant and implement `other::high_pass` as `clamp(orig − gaussian(orig, sigma_from_radius(radius)) + 128)` reusing the M6 Gaussian kernel, rejecting a non-finite or non-positive radius
- [x] 1.5 Add the `Filter::Custom { kernel: [[f64; 5]; 5], scale: f64, offset: f64 }` variant and implement `other::custom` as a 5×5 f64-accumulating convolution with clamp-to-edge, rejecting `scale == 0` and any non-finite kernel/scale/offset
- [x] 1.6 Unit-test Maximum/Minimum on a white-on-black mask, radius 0 no-op, radius above 100 clamping, and determinism
- [x] 1.7 Unit-test Offset wrap by a full image width, background fill at the edge, and `(0, 0)` no-op
- [x] 1.8 Unit-test High Pass: a uniform field becomes 128, an edge deviates more than a flat region, and invalid radii are rejected
- [x] 1.9 Unit-test Custom: identity pass-through, embedded 3×3 mean equals the box blur, `scale == 0` and non-finite parameters are rejected, and kernel orientation is left-to-right/top-to-bottom
- [x] 1.10 Unit-test alpha preservation, clamp-to-edge, and 1×1 / 1-px images for every Other variant

## 2. M7-B — Stylize family and tests

- [x] 2.1 Add the `Filter::Emboss { angle: f64, height: f64, amount: f64 }` variant and implement `stylize::emboss` as a directional difference scaled by height and amount plus the 128 gray bias, with gray output
- [x] 2.2 Add the `Filter::FindEdges` variant and implement `stylize::find_edges` as the per-channel Sobel magnitude rendered `255 − clamp(magnitude)`
- [x] 2.3 Add the `Filter::Solarize` variant and implement `stylize::solarize` as the fixed `if v >= 128 { 255 − v } else { v }` curve
- [x] 2.4 Unit-test Emboss: output is gray, negating the angle swaps highlight and shadow, a uniform field becomes 128, and an out-of-range or non-finite angle/height/amount is rejected
- [x] 2.5 Unit-test Find Edges: a constant image is uniformly 255, and an edge renders darker than its flat surroundings
- [x] 2.6 Unit-test Solarize: values below 128 are unchanged, a value at 128 and above is inverted, and applying it twice differs from applying it once
- [x] 2.7 Unit-test alpha preservation, clamp-to-edge, and 1×1 / 1-px images for every Stylize variant

## 3. M7-C — ImageMagick oracle and no-equivalent table

- [x] 3.1 Extend `scripts/filter_oracle.py` with named operators for Maximum/Minimum (`-morphology Dilate/Erode Square:N`), Offset wrap (`-roll`), Custom (`-convolve` with `convolve:scale` and `-bias`), Emboss (`-emboss`), Find Edges (`-edge`), and Solarize (`-solarize 50%`)
- [x] 3.2 Diff Maximum, Minimum, Offset wrap, and Custom within their stated tolerances against the oracle image
- [x] 3.3 Diff Solarize (`-solarize 50%`) within tolerance 0 and Emboss against `-emboss` if that kernel is verified faithful
- [x] 3.4 Add one mapping-table row per M7 `Filter` variant and assert Offset background fill, High Pass (unless a faithful `-compose mathematics` recipe is verified), Find Edges, and unfaithful Emboss use tolerance 0 with a non-empty no-equivalent note
- [x] 3.5 Skip the differential tests with a message when `magick` is absent and add no `#[ignore]`
- [x] 3.6 Document the mapping, exact flags, divergences, and the verified ImageMagick version in `crates/pictura-filters/tests/README.md`

## 4. M7-D — App filter kinds and unit test

- [x] 4.1 Register the Other and Stylize filter kinds (Maximum, Minimum, Offset, High Pass, Custom, Emboss, Find Edges, Solarize) in the app filter menu with their parameter descriptors
- [x] 4.2 Add an app unit test asserting each new kind maps to the matching `pictura-filters` variant and parameter set

## 5. M7-E — OpenSpec change, reconcile, and verify

- [x] 5.1 Validate `openspec validate m7-stylize-other --strict` and `openspec validate --all --strict` green
- [x] 5.2 Run `cargo test --workspace` green with the per-filter unit tests and oracle differentials within tolerance or documented no-equivalent
- [x] 5.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] 5.4 Run `scripts/guard.sh` green (no `.8bf`, no artboards, no unmarked `docs/` edits)
- [x] 5.5 Reconcile `docs/dev/m7-stylize-other.md` against the shipped `Filter` variants and record any divergence
