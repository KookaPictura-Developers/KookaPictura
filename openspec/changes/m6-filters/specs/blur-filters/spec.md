## ADDED Requirements

### Requirement: Blur filter application and error contract

The system SHALL provide `pictura_filters::apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError>`. It SHALL operate in place on a planar 8-bit buffer whose `channels` is 3 (RGB) or 4 (RGBA), treating channels 1 through 3 as the color planes and channel 4, when present, as alpha. It SHALL transform every color sample or return an error; it MUST NOT partially apply a filter and then fail. Malformed buffers MUST return `FilterError` instead of panicking.

#### Scenario: Apply a blur to a 3-channel planar buffer

- **WHEN** `apply` receives a blur `Filter` variant and a 3-channel planar buffer
- **THEN** the R, G, and B planes are rewritten in place and `Ok(())` is returned

#### Scenario: Reject an unsupported channel count

- **WHEN** the buffer has a channel count other than 3 or 4
- **THEN** `apply` returns `FilterError::Unsupported` and leaves the buffer unchanged

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives an empty buffer, or a buffer whose `data.len()` does not equal `width * height * channels`, or a buffer with zero width or height
- **THEN** `apply` returns a `FilterError` and does not panic

### Requirement: Alpha channel preservation for blur

For 4-channel buffers, every blur filter SHALL leave channel 4 bit-identical. Only channels 1 through 3 SHALL be modified.

#### Scenario: Every blur variant preserves alpha

- **WHEN** each blur variant is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Gaussian blur

The system SHALL implement `Filter::GaussianBlur { radius: f64 }` as a separable Gaussian FIR convolution. The UI radius SHALL be the 3σ support (`σ = sigma_from_radius(radius)`, with σ floored at 0.1); the 1-D kernel SHALL be normalized and have `⌈3σ⌉` samples of support each side; the two passes SHALL run horizontally then vertically. `radius == 0.0` SHALL be a no-op (or near-identity within 1 LSB). A negative or non-finite radius SHALL be rejected with `FilterError::InvalidParams`. Oracle expectation: differential against ImageMagick `-gaussian-blur 0xσ` with the matching σ within an absolute tolerance of 2 per sample for `radius <= 50`.

#### Scenario: Solid color is unchanged

- **WHEN** Gaussian blur is applied to a uniform-color buffer
- **THEN** every output sample equals the input within 1 LSB

#### Scenario: An edge softens as radius grows

- **WHEN** Gaussian blur is applied to a step edge at increasing radii
- **THEN** the transition width grows monotonically with the radius

#### Scenario: Radius zero is a no-op

- **WHEN** Gaussian blur is applied with radius 0.0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Matches ImageMagick

- **WHEN** Gaussian blur with a given radius is applied to the oracle image and ImageMagick applies `-gaussian-blur 0xσ` with the matching σ
- **THEN** no sample differs by more than 2

### Requirement: Box blur

The system SHALL implement `Filter::BoxBlur { radius: u32 }` as a separable moving average over the `(2r+1)²` neighborhood, applied per color channel. `radius == 0` SHALL be a no-op. Oracle expectation: differential against ImageMagick `-statistic mean NxN` with `N = 2*radius + 1` within an absolute tolerance of 1 per sample.

#### Scenario: A uniform region keeps its exact mean

- **WHEN** Box blur is applied to a uniform-color buffer
- **THEN** every output sample equals the region mean rounded, within 1 LSB

#### Scenario: Radius zero is a no-op

- **WHEN** Box blur is applied with radius 0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Matches ImageMagick

- **WHEN** Box blur with a given radius is applied to the oracle image and ImageMagick applies `-statistic mean NxN` with `N = 2*radius + 1`
- **THEN** no sample differs by more than 1

### Requirement: Motion blur

The system SHALL implement `Filter::MotionBlur { angle: f64, distance: u32 }` as a 1-D line convolution through each pixel along `angle` with length `distance`. `angle` SHALL span `-360.0..=360.0`; `distance` SHALL span `1..=999`; values outside those ranges SHALL be rejected with `FilterError::InvalidParams`. `angle == 0` SHALL streak horizontally and `angle == 90` vertically. `distance <= 1` SHALL be near-identity within 1 LSB. The kernel SHALL be supersampled to reduce aliasing. Oracle expectation: differential against ImageMagick `-motion-blur 0xN+angle` with the divergence documented if its kernel differs.

#### Scenario: Angle zero streaks horizontally

- **WHEN** Motion blur with angle 0 and a distance greater than 1 is applied to an impulse
- **THEN** the energy spreads horizontally and not vertically

#### Scenario: Angle ninety streaks vertically

- **WHEN** Motion blur with angle 90 and a distance greater than 1 is applied to an impulse
- **THEN** the energy spreads vertically and not horizontally

#### Scenario: Distance one is near-identity

- **WHEN** Motion blur is applied with distance 1
- **THEN** every output sample equals the input within 1 LSB

#### Scenario: Out-of-range distance is rejected

- **WHEN** Motion blur is applied with distance 0 or 1000
- **THEN** `apply` returns `FilterError::InvalidParams` and does not panic

### Requirement: Radial blur

The system SHALL implement `Filter::RadialBlur { method: RadialMethod, amount: f64, quality: Quality }`. For `RadialMethod::Spin` it SHALL take a moving average along the polar angle over the rotation amount; for `RadialMethod::Zoom` it SHALL take a moving average along the radial direction. `amount` SHALL be finite and non-negative; for `RadialMethod::Zoom` it SHALL span `0..=100`. `Quality` (`Draft`, `Good`, `Best`) SHALL select the sampling density, with `Draft` visibly grainier than `Best` at equal amount. `amount == 0` SHALL be a no-op. Negative, non-finite, or (`Zoom`) out-of-range amounts SHALL be rejected with `FilterError::InvalidParams`. Oracle expectation: no faithful ImageMagick equivalent exists (ImageMagick `-radial-blur` semantics differ), so property and known-value tests cover the behavior and the divergence is documented.

#### Scenario: Spin smears rotationally

- **WHEN** Radial blur is applied with `RadialMethod::Spin` and a non-zero amount to a radial impulse
- **THEN** the energy spreads along concentric arcs about the center rather than radially

#### Scenario: Zoom smears radially

- **WHEN** Radial blur is applied with `RadialMethod::Zoom` and a non-zero amount to an off-center impulse
- **THEN** the energy spreads along the radial direction from the center

#### Scenario: Amount zero is identity

- **WHEN** Radial blur is applied with amount 0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: No faithful equivalent is documented

- **WHEN** the oracle mapping table is checked
- **THEN** Radial blur has no ImageMagick operator, tolerance 0, and a non-empty no-equivalent note

### Requirement: Average blur

The system SHALL implement `Filter::Average` with no parameters. It SHALL compute the mean color over the whole buffer, per color channel, and write that mean to every pixel. Oracle expectation: no faithful ImageMagick operator exists (it is a trivial region mean), so a known-value test asserts the output is the rounded region mean.

#### Scenario: Uniform color is the exact mean

- **WHEN** Average is applied to a buffer whose color samples sum to a known per-channel mean
- **THEN** every output pixel carries that rounded mean per channel

#### Scenario: A varying region is flattened

- **WHEN** Average is applied to a buffer with more than one distinct color
- **THEN** every output pixel equals every other output pixel

### Requirement: Blur and Blur More

The system SHALL implement `Filter::Blur` and `Filter::BlurMore` with no parameters, as a fixed small 3×3 Gaussian-like convolution. `BlurMore` SHALL use the same kernel shape with a 3 to 4 times stronger gain, so its deviation from the input is at least as large as `Blur`'s. A uniform-color buffer SHALL be unchanged. Oracle expectation: no faithful ImageMagick equivalent exists (fixed kernels), so known-value tests cover the two strengths and the documented divergence.

#### Scenario: Blur More is stronger than Blur

- **WHEN** `Blur` and `BlurMore` are each applied to the same edge image
- **THEN** the total deviation from the input is larger for `BlurMore` than for `Blur`

#### Scenario: Uniform color is unchanged

- **WHEN** `Blur` or `BlurMore` is applied to a uniform-color buffer
- **THEN** the buffer is bit-exactly unchanged

### Requirement: Surface blur (bilateral)

The system SHALL implement `Filter::SurfaceBlur { radius: u32, threshold: u8 }` as a bilateral filter: each output pixel is a weighted average of its neighborhood where the weight is the product of a spatial Gaussian driven by `radius` and a range kernel driven by `threshold`, so averaging is suppressed across intensity edges. `radius` SHALL span `1..=100` and `threshold` SHALL span `1..=255`; values outside those ranges SHALL be rejected with `FilterError::InvalidParams`. Oracle expectation: no faithful ImageMagick equivalent exists (Photoshop's Surface Blur is a closed bilateral variant), so property tests cover edge preservation and threshold-controlled smoothing, and the divergence is documented.

#### Scenario: A small threshold preserves a step edge

- **WHEN** Surface blur with a small threshold is applied to an image containing a step edge
- **THEN** the edge contrast is preserved while the flat regions are smoothed

#### Scenario: A larger threshold smooths across the edge

- **WHEN** the threshold is increased on the same image
- **THEN** more averaging occurs across the edge than at the small threshold

#### Scenario: Invalid parameters are rejected

- **WHEN** Surface blur is applied with radius 0 or threshold 0
- **THEN** `apply` returns `FilterError::InvalidParams` and does not panic

### Requirement: Clamp-to-edge borders and tiny images

Every blur kernel SHALL sample with clamp-to-edge at the image borders, so an out-of-range index maps to the nearest edge sample, and SHALL clamp its kernel support to the available source. A 1×1 image and a 1-pixel-wide or 1-pixel-tall image MUST NOT panic and MUST return either a result or a typed error.

#### Scenario: A 1x1 image does not panic

- **WHEN** every blur variant is applied to a 1×1 buffer
- **THEN** no variant panics and each returns `Ok(())` or a `FilterError`

#### Scenario: Border pixels clamp to the edge

- **WHEN** a blur is applied to a small buffer and a pixel at the border is inspected
- **THEN** the result is consistent with repeating the edge sample rather than reading out of bounds

### Requirement: Deterministic blur output

Blur filters SHALL be deterministic. The same filter applied to equal input buffers SHALL produce bit-identical output every run, with no random, time, or thread-order dependence.

#### Scenario: Repeated runs match

- **WHEN** each blur variant is applied to two clones of one buffer
- **THEN** the two output buffers are bit-identical

### Requirement: Blur ImageMagick oracle and no-equivalent classification

The system SHALL ship `scripts/filter_oracle.py`, which applies an ImageMagick operator to a raw 8-bit image, and `crates/pictura-filters/tests/oracle.rs`, which diffs `apply` against it. The mapping table SHALL have exactly one row per `Filter` variant. Gaussian and Box SHALL be diffed (`-gaussian-blur 0xσ`, `-statistic mean NxN`) within their stated tolerances. Motion, Average, Radial, Surface, Blur, and BlurMore SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0 with property or known-value tests — for Motion because ImageMagick's `-motion-blur` is a one-sided Gaussian line kernel, not a symmetric uniform streak. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table covers every blur variant

- **WHEN** the oracle tests run
- **THEN** every blur variant has a table row, each no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes
