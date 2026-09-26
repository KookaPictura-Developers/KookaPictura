# stylize-filters Specification

## Purpose
Emboss, Find Edges, and Solarize with alpha preservation, edge handling, determinism, and an oracle classification.
## Requirements
### Requirement: Stylize filter application and error contract

The system SHALL provide `pictura_filters::apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError>`. It SHALL operate in place on a planar 8-bit buffer whose `channels` is 3 (RGB) or 4 (RGBA), treating channels 1 through 3 as the color planes and channel 4, when present, as alpha. It SHALL transform every color sample or return an error; it MUST NOT partially apply a filter and then fail. Malformed buffers MUST return `FilterError` instead of panicking.

#### Scenario: Apply a Stylize filter to a 3-channel planar buffer

- **WHEN** `apply` receives a Stylize `Filter` variant and a 3-channel planar buffer
- **THEN** the R, G, and B planes are rewritten in place and `Ok(())` is returned

#### Scenario: Reject an unsupported channel count

- **WHEN** the buffer has a channel count other than 3 or 4
- **THEN** `apply` returns `FilterError::Unsupported` and leaves the buffer unchanged

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives an empty buffer, or a buffer whose `data.len()` does not equal `width * height * channels`, or a buffer with zero width or height
- **THEN** `apply` returns a `FilterError` and does not panic

### Requirement: Alpha channel preservation for Stylize filters

For 4-channel buffers, every Stylize filter SHALL leave channel 4 bit-identical. Only channels 1 through 3 SHALL be modified.

#### Scenario: Every Stylize variant preserves alpha

- **WHEN** each Stylize variant is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Emboss

The system SHALL implement `Filter::Emboss { angle: f64, height: f64, amount: f64 }`. It SHALL compute a directional difference of the image along `angle`, scale the difference by `height`, add a 128 mid-gray bias, and scale the deviation from mid-gray by `amount` percent, so the output is a gray relief in which all three color channels carry the same value. `angle` SHALL be finite and within `-360.0..=360.0`; `height` and `amount` SHALL be finite and strictly greater than 0. A value outside those ranges SHALL be rejected with `FilterError::InvalidParams`. A uniform-color region SHALL become mid-gray 128. Oracle expectation: differential against ImageMagick `-emboss 0xH+angle` if that kernel is verified faithful, otherwise classified as no-equivalent with the divergence documented and covered by a property test that confirms the gray output and the angle-sign highlight/shadow swap.

#### Scenario: The output is gray

- **WHEN** Emboss is applied to a color image
- **THEN** every output pixel has R, G, and B equal to one another

#### Scenario: Negating the angle swaps the highlight and shadow

- **WHEN** Emboss is applied with angle A and then with angle −A to the same edge image
- **THEN** the highlight side of the edge under A becomes the shadow side under −A and vice versa

#### Scenario: A uniform field becomes mid-gray

- **WHEN** Emboss is applied to a uniform-color buffer
- **THEN** every output color sample equals 128 within 1 LSB

#### Scenario: Invalid parameters are rejected

- **WHEN** Emboss is applied with an angle outside `-360.0..=360.0`, or with a non-finite angle, height, or amount, or with `height <= 0` or `amount <= 0`
- **THEN** `apply` returns `FilterError::InvalidParams` and does not panic

### Requirement: Find Edges

The system SHALL implement `Filter::FindEdges` with no parameters. It SHALL compute a Sobel gradient magnitude per color channel and render edges dark on a light field as `out = 255 − clamp(magnitude)`. A constant-color image SHALL map to a uniform 255. Oracle expectation: ImageMagick `-edge` uses a different edge detector and the opposite polarity (bright edges on black), so Find Edges is classified as no-equivalent with tolerance 0 and covered by property and known-value tests, with the polarity inversion documented.

#### Scenario: A constant image is uniformly white

- **WHEN** Find Edges is applied to a constant-color buffer
- **THEN** every output color sample equals 255

#### Scenario: An edge renders darker than its surroundings

- **WHEN** Find Edges is applied to an image containing a step edge
- **THEN** at least one pixel on the edge has a lower value than the flat regions on either side

#### Scenario: No faithful equivalent is documented

- **WHEN** the oracle mapping table is checked
- **THEN** Find Edges has no ImageMagick operator, tolerance 0, and a non-empty no-equivalent note that records the polarity inversion

### Requirement: Solarize

The system SHALL implement `Filter::Solarize` with no parameters. For each color channel it SHALL apply the fixed 50% curve `out = if v >= 128 { 255 − v } else { v }`, so values at or above 128 are inverted and values below 128 are unchanged. Solarize SHALL NOT be an involution: applying it twice differs from applying it once. Oracle expectation: differential against ImageMagick `-solarize 50%` within tolerance 0 at 8-bit.

#### Scenario: Values below the threshold are unchanged

- **WHEN** Solarize is applied to a buffer whose color samples are all below 128
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Values at or above the threshold are inverted

- **WHEN** Solarize is applied to a buffer with a sample at 200
- **THEN** that sample becomes 55

#### Scenario: Applying twice is not the same as applying once

- **WHEN** Solarize is applied twice to a non-uniform buffer
- **THEN** the result differs from a single application

#### Scenario: Matches ImageMagick

- **WHEN** Solarize is applied to the oracle image and ImageMagick applies `-solarize 50%`
- **THEN** no sample differs by more than 0

### Requirement: Clamp-to-edge borders and tiny images for Stylize filters

Every neighborhood Stylize filter (Emboss and Find Edges) SHALL sample with clamp-to-edge at the image borders, so an out-of-range index maps to the nearest edge sample. Solarize is pointwise. A 1×1 image and a 1-pixel-wide or 1-pixel-tall image MUST NOT panic and MUST return either a result or a typed error.

#### Scenario: A 1x1 image does not panic

- **WHEN** every Stylize variant is applied to a 1×1 buffer
- **THEN** no variant panics and each returns `Ok(())` or a `FilterError`

#### Scenario: Border pixels clamp to the edge

- **WHEN** Emboss or Find Edges is applied to a small buffer and a border pixel is inspected
- **THEN** the result is consistent with repeating the edge sample rather than reading out of bounds

### Requirement: Deterministic Stylize output

Stylize filters SHALL be deterministic. The same filter applied to equal input buffers SHALL produce bit-identical output every run, with no random, time, or thread-order dependence.

#### Scenario: Repeated runs match

- **WHEN** each Stylize variant is applied to two clones of one buffer
- **THEN** the two output buffers are bit-identical

### Requirement: Stylize ImageMagick oracle and no-equivalent classification

The system SHALL ship `scripts/filter_oracle.py`, which applies an ImageMagick operator to a raw 8-bit image, and `crates/pictura-filters/tests/oracle.rs`, which diffs `apply` against it. The mapping table SHALL have exactly one row per Stylize `Filter` variant. Solarize SHALL be diffed (`-solarize 50%`) within tolerance 0. Emboss SHALL be diffed against `-emboss` if that kernel is verified faithful, otherwise classified as no-equivalent. Find Edges SHALL be classified as no-equivalent with tolerance 0, a property or known-value test, and a non-empty note recording the polarity inversion. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table covers every Stylize variant

- **WHEN** the oracle tests run
- **THEN** every Stylize variant has a table row, each no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes

