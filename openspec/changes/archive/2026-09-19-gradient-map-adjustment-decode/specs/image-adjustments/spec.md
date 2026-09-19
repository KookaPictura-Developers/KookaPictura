## ADDED Requirements

### Requirement: Gradient Map adjustment

The system SHALL implement `Adjustment::GradientMap(GradientMapParams {
stops, reverse })`, where each `GradientStop { location, color }` carries a
Photoshop gradient position in `0..=4096` and an RGB colour, and `reverse`
defaults to false. It SHALL map each pixel's Rec.601 luminance `L` through the
gradient: when `reverse` is set it SHALL sample `1 − L` instead, it SHALL
interpolate linearly between the two stops bracketing the sampled position, it
SHALL clamp sampled positions below the first stop and above the last stop to
the endpoint colours, and it SHALL write the sampled colour to the three colour
channels while leaving alpha untouched. `apply` SHALL reject a stop list with
fewer than two stops, a non-increasing `location`, or any `location` above
4096. Oracle expectation: no faithful ImageMagick operator exists, so property
tests cover the identity of a black-to-white ramp over a neutral grey range, the
reversed mapping, and the endpoint clamp.

#### Scenario: Black to white gradient is the identity on neutral greys

- **WHEN** Gradient Map is applied with stops at location 0 colour `[0, 0, 0]` and location 4096 colour `[255, 255, 255]` to a neutral grey ramp
- **THEN** each grey sample maps to itself within 1 LSB

#### Scenario: Reverse flips the lookup

- **WHEN** the same black-to-white gradient is applied with `reverse` true
- **THEN** an input of black maps near white and an input of white maps near black

#### Scenario: Samples outside the stop range clamp to the endpoints

- **WHEN** a two-stop gradient places its stops at locations 1024 and 3072
- **THEN** samples below location 1024 read the first stop's colour and samples above location 3072 read the last stop's colour

#### Scenario: A three-stop gradient honors the interior stop

- **WHEN** a gradient has a distinct interior stop at location 2048
- **THEN** a pixel whose luminance samples location 2048 reads the interior stop's colour rather than a blend of the outer stops

#### Scenario: Invalid stops are rejected

- **WHEN** Gradient Map is given fewer than two stops, duplicate or decreasing locations, or a location above 4096
- **THEN** `apply` returns `AdjustError::InvalidParams` and does not panic

#### Scenario: Alpha is preserved

- **WHEN** Gradient Map is applied to an RGBA buffer
- **THEN** the alpha plane is bit-identical to the input

## MODIFIED Requirements

### Requirement: Alpha channel preservation

For 4-channel buffers, `apply` SHALL leave channel 4 bit-identical for every adjustment. Only channels 1 through 3 SHALL be modified.

#### Scenario: All adjustment variants preserve alpha

- **WHEN** each of the 16 adjustment variants is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Parameter validation and error signaling

`apply` SHALL return `AdjustError` instead of panicking when parameters are out of range or buffers are malformed. It SHALL reject a zero-pixel buffer and a buffer whose `data.len()` does not equal `width * height * channels`. Each parameterized adjustment SHALL validate its documented ranges: Levels gamma greater than zero and input black below input white; Curves at least 2 and at most 14 points with strictly increasing inputs; Brightness/Contrast brightness in `-150..=150` and contrast in `-50..=100`; Exposure gamma greater than zero and exposure/offset finite; Posterize levels in `2..=255`; Threshold level in `1..=255`; Hue/Saturation hue in `-180..=180` and saturation/lightness in `-100..=100`; Black & White weights in `-200..=300` percent; Photo Filter density in `0..=100`; Channel Mixer weights and constant no greater than 200 percent in absolute value; Vibrance and saturation in `-100..=100`; Color Balance values finite and within `-100..=100`; Gradient Map at least two stops with strictly increasing locations each at most 4096.

#### Scenario: Out-of-range parameters return an error

- **WHEN** an adjustment is given a value outside its documented range, such as Levels gamma `0.0`, Curves with one point, Brightness/Contrast brightness `200`, Exposure gamma `0.0`, Posterize `1`, Threshold `0`, Hue/Saturation hue `200`, Black & White weight `999`, Photo Filter density `150`, Channel Mixer weight `999`, Vibrance `500`, Color Balance `200`, or Gradient Map with one stop or a location above 4096
- **THEN** `apply` returns `AdjustError::InvalidParams` or `AdjustError::Unsupported` and does not panic

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives a zero-pixel buffer, a 2-channel buffer, or a buffer whose data length does not match `width * height * channels`
- **THEN** `apply` returns `AdjustError` and does not panic

### Requirement: ImageMagick differential oracle and no-equivalent classification

The system SHALL ship `scripts/adjust_oracle.py`, which applies an ImageMagick operator to a raw 8-bit image, and `crates/pictura-adjust/tests/oracle.rs`, which diffs `apply` against it. The test table SHALL have exactly one row per adjustment variant. Only Levels, Invert, and Desaturate SHALL be diffed (`-level ... +level ...`, `-negate`, `-modulate 100,0,100`), with tolerances of 1, 0, and 1. The other thirteen adjustments SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0: BlackWhite, PhotoFilter, GradientMap, Vibrance, ColorBalance, Auto, Curves, Exposure, BrightnessContrast, HueSaturation, ChannelMixer, Posterize, and Threshold. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table matches the implemented contract

- **WHEN** the oracle tests run
- **THEN** the mapping table has 16 rows, every no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes
