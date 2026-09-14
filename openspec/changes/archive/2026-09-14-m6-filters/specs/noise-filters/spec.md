## ADDED Requirements

### Requirement: Noise filter application and error contract

The system SHALL provide `pictura_filters::apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError>`. It SHALL operate in place on a planar 8-bit buffer whose `channels` is 3 (RGB) or 4 (RGBA), treating channels 1 through 3 as the color planes and channel 4, when present, as alpha. It SHALL transform every color sample or return an error; it MUST NOT partially apply a filter and then fail. Malformed buffers MUST return `FilterError` instead of panicking.

#### Scenario: Apply a noise filter to a 3-channel planar buffer

- **WHEN** `apply` receives a noise `Filter` variant and a 3-channel planar buffer
- **THEN** the R, G, and B planes are rewritten in place and `Ok(())` is returned

#### Scenario: Reject an unsupported channel count

- **WHEN** the buffer has a channel count other than 3 or 4
- **THEN** `apply` returns `FilterError::Unsupported` and leaves the buffer unchanged

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives an empty buffer, or a buffer whose `data.len()` does not equal `width * height * channels`, or a buffer with zero width or height
- **THEN** `apply` returns a `FilterError` and does not panic

### Requirement: Alpha channel preservation for noise

For 4-channel buffers, every noise filter SHALL leave channel 4 bit-identical. Only channels 1 through 3 SHALL be modified.

#### Scenario: Every noise variant preserves alpha

- **WHEN** each noise variant is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Add Noise

The system SHALL implement `Filter::AddNoise { amount: f64, distribution: NoiseDistribution, monochromatic: bool, seed: u64 }`. For `NoiseDistribution::Uniform` it SHALL add a delta drawn uniformly from `[−amount, +amount]`; for `NoiseDistribution::Gaussian` it SHALL add a zero-mean Gaussian `N(0, σ)` delta with σ proportional to `amount`. `amount` SHALL be a finite percentage in `0..=400`; `amount == 0` SHALL be a no-op, and negative, non-finite, or `> 400` values SHALL be rejected with `FilterError::InvalidParams`. When `monochromatic` is true it SHALL draw one delta per pixel and apply it to all three color channels, preserving hue; otherwise it SHALL draw a delta per channel, producing colored speckle. The deltas SHALL come from a seeded RNG driven by `seed`. Oracle expectation: no faithful ImageMagick equivalent exists (RNG streams differ), so statistical tests cover the distribution shape and same-seed tests cover reproducibility, and the divergence is documented.

#### Scenario: Uniform noise is zero-mean

- **WHEN** Add Noise with `Uniform` is applied to a uniform mid-grey buffer
- **THEN** the mean stays within tolerance of the input and the standard deviation grows with `amount`

#### Scenario: Gaussian noise has a bell-shaped histogram

- **WHEN** Add Noise with `Gaussian` and a sufficiently large amount is applied to a uniform buffer
- **THEN** the delta histogram is bell-shaped rather than flat

#### Scenario: Monochromatic noise preserves hue

- **WHEN** monochromatic Add Noise is applied to a colored buffer
- **THEN** the per-pixel channel differences R−G and G−B stay within tolerance of the input, while non-monochromatic Add Noise changes them

#### Scenario: Amount zero is a no-op

- **WHEN** Add Noise is applied with amount 0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Out-of-range amount is rejected

- **WHEN** Add Noise is applied with a negative amount or an amount above 400
- **THEN** `apply` returns `FilterError::InvalidParams` and does not panic

### Requirement: Median filter

The system SHALL implement `Filter::Median { radius: u32 }` as a per-channel rank filter that replaces each pixel with the median of the channel values in its `(2r+1)²` window, sampled with clamp-to-edge. `radius` SHALL span `0..=100`; `radius == 0` SHALL be a no-op and a radius above 100 SHALL be rejected with `FilterError::InvalidParams`. Oracle expectation: differential against ImageMagick `-median R` within an absolute tolerance of 1 per sample.

#### Scenario: Salt-and-pepper noise is removed

- **WHEN** Median with radius 1 is applied to an image containing isolated impulse pixels
- **THEN** the impulses are removed while a step edge is preserved within a small band

#### Scenario: Radius zero is a no-op

- **WHEN** Median is applied with radius 0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Matches ImageMagick

- **WHEN** Median with a given radius is applied to the oracle image and ImageMagick applies `-median R`
- **THEN** no sample differs by more than 1

### Requirement: Despeckle

The system SHALL implement `Filter::Despeckle` with no parameters. It SHALL detect edges by gradient or color difference and apply a smoothing operation only to pixels not on an edge, so isolated noise is smoothed while strong edges are preserved. Oracle expectation: no faithful ImageMagick equivalent exists (the edge detector and smoothing operator are closed), so a property test covers the isolated-noise smoothing with a preserved edge profile, and the divergence is documented.

#### Scenario: Isolated noise is smoothed

- **WHEN** Despeckle is applied to an image with isolated speckle over a smooth region
- **THEN** the speckle is reduced relative to the input

#### Scenario: Strong edges are preserved

- **WHEN** Despeckle is applied to an image containing a strong step edge
- **THEN** the edge profile stays within tolerance of the input

### Requirement: Seeded RNG determinism

Add Noise SHALL be the only random noise filter. It SHALL drive its RNG from the `seed` field, so two applies with the same seed and input produce bit-identical output, two applies with different seeds produce different output, and a re-apply of the same filter reproduces the first result exactly. Median and Despeckle SHALL be deterministic and SHALL NOT touch the RNG.

#### Scenario: Same seed is bit-identical

- **WHEN** Add Noise with the same seed is applied to two clones of one buffer
- **THEN** the two output buffers are bit-identical

#### Scenario: Different seeds differ

- **WHEN** Add Noise is applied to the same buffer with two different seeds
- **THEN** the two output buffers differ

#### Scenario: Median and Despeckle are deterministic

- **WHEN** Median or Despeckle is applied twice to equal input buffers
- **THEN** the outputs are bit-identical

### Requirement: Clamp-to-edge borders and tiny images for noise

Every neighborhood noise filter SHALL sample with clamp-to-edge at the image borders and SHALL clamp its window to the available source. A 1×1 image and a 1-pixel-wide or 1-pixel-tall image MUST NOT panic and MUST return either a result or a typed error.

#### Scenario: A tiny image does not panic

- **WHEN** every noise variant is applied to a 1×1 buffer and to a 1-pixel-wide buffer
- **THEN** no variant panics and each returns `Ok(())` or a `FilterError`

#### Scenario: Border pixels clamp to the edge

- **WHEN** Median is applied to a small buffer and a pixel at the border is inspected
- **THEN** the result is consistent with repeating the edge sample rather than reading outside the buffer

### Requirement: Noise ImageMagick oracle and no-equivalent classification

The system SHALL ship `scripts/filter_oracle.py` and `crates/pictura-filters/tests/oracle.rs`. The mapping table SHALL have exactly one row per `Filter` variant. Median SHALL be diffed against `-median R` within its stated tolerance. Add Noise and Despeckle SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0 with statistical, same-seed, or property tests. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table covers every noise variant

- **WHEN** the oracle tests run
- **THEN** every noise variant has a table row, each no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes
