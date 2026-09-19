## MODIFIED Requirements

### Requirement: Alpha channel preservation

For 4-channel buffers, `apply` SHALL leave channel 4 bit-identical whenever it
succeeds, and SHALL leave the whole buffer untouched when it refuses a generative
variant, so channel 4 is never modified by any adjustment. Only channels 1
through 3 SHALL be modified.

#### Scenario: All adjustment variants preserve alpha

- **WHEN** each of the 17 adjustment variants — the sixteen destructive variants and the refused generative `GradientFill` — is passed to `apply` with an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Parameter validation and error signaling

`apply` SHALL return `AdjustError` instead of panicking when parameters are out of range or buffers are malformed. It SHALL reject a zero-pixel buffer and a buffer whose `data.len()` does not equal `width * height * channels`. Each parameterized adjustment SHALL validate its documented ranges: Levels gamma greater than zero and input black below input white; Curves at least 2 and at most 14 points with strictly increasing inputs; Brightness/Contrast brightness in `-150..=150` and contrast in `-50..=100`; Exposure gamma greater than zero and exposure/offset finite; Posterize levels in `2..=255`; Threshold level in `1..=255`; Hue/Saturation hue in `-180..=180` and saturation/lightness in `-100..=100`; Black & White weights in `-200..=300` percent; Photo Filter density in `0..=100`; Channel Mixer weights and constant no greater than 200 percent in absolute value; Vibrance and saturation in `-100..=100`; Color Balance values finite and within `-100..=100`; Gradient Map at least two stops with strictly increasing locations each at most 4096. `GradientFill` has no `apply` parameter validation because `apply` SHALL refuse it as `AdjustError::Unsupported` and leave the buffer unchanged.

#### Scenario: Out-of-range parameters return an error

- **WHEN** an adjustment is given a value outside its documented range, such as Levels gamma `0.0`, Curves with one point, Brightness/Contrast brightness `200`, Exposure gamma `0.0`, Posterize `1`, Threshold `0`, Hue/Saturation hue `200`, Black & White weight `999`, Photo Filter density `150`, Channel Mixer weight `999`, Vibrance `500`, Color Balance `200`, Gradient Map with one stop or a location above 4096, or any Gradient Fill
- **THEN** `apply` returns `AdjustError::InvalidParams` or `AdjustError::Unsupported` and does not panic

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives a zero-pixel buffer, a 2-channel buffer, or a buffer whose data length does not match `width * height * channels`
- **THEN** `apply` returns `AdjustError` and does not panic

### Requirement: ImageMagick differential oracle and no-equivalent classification

The system SHALL ship `scripts/adjust_oracle.py`, which applies an ImageMagick operator to a raw 8-bit image, and `crates/pictura-adjust/tests/oracle.rs`, which diffs `apply` against it. The test table SHALL have exactly one row for each destructive adjustment variant plus a no-equivalent row for the generative `GradientFill`. Only Levels, Invert, and Desaturate SHALL be diffed (`-level ... +level ...`, `-negate`, `-modulate 100,0,100`), with tolerances of 1, 0, and 1. The other fourteen adjustments SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0: BlackWhite, PhotoFilter, GradientMap, GradientFill, Vibrance, ColorBalance, Auto, Curves, Exposure, BrightnessContrast, HueSaturation, ChannelMixer, Posterize, and Threshold. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table matches the implemented contract

- **WHEN** the oracle tests run
- **THEN** the mapping table has 17 rows, every no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes

## ADDED Requirements

### Requirement: Gradient Fill adjustment model

The system SHALL provide `Adjustment::GradientFill(GradientFillParams {
stops, reverse, kind, angle_deg, scale })`, where `stops` is a list of the
existing `GradientStop { location, color }` positions in `0..=4096` with an RGB
colour, `kind` is a `GradientKind` of Linear, Radial, Angle, Reflected, or
Diamond, `angle_deg` is the rotation in degrees, `scale` is a percent defaulting
to 100, and `reverse` defaults to false. A gradient fill is composited
generatively rather than applied to the backdrop, so `apply` SHALL refuse it as
`AdjustError::Unsupported` and SHALL leave the buffer unchanged.

#### Scenario: The model carries the decoded parameters

- **WHEN** a `GradientFill` is constructed with two stops, kind Radial, angle 45, scale 100, and reverse false
- **THEN** it is a distinct `Adjustment` variant carrying those values

#### Scenario: Apply refuses a Gradient Fill

- **WHEN** `apply` is called with `Adjustment::GradientFill` and any buffer
- **THEN** it returns `AdjustError::Unsupported` and the buffer is unchanged
