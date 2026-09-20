## MODIFIED Requirements

### Requirement: Alpha channel preservation

For 4-channel buffers, `apply` SHALL leave channel 4 bit-identical whenever it
succeeds, and SHALL leave the whole buffer untouched when it refuses a generative
variant, so channel 4 is never modified by any adjustment. Only channels 1
through 3 SHALL be modified.

#### Scenario: All adjustment variants preserve alpha

- **WHEN** each of the 20 adjustment variants — the seventeen destructive variants and the refused fills `SolidFill`, `GradientFill`, and `PatternFill` — is passed to `apply` with an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Parameter validation and error signaling

`apply` SHALL return `AdjustError` instead of panicking when parameters are out of range or buffers are malformed. It SHALL reject a zero-pixel buffer and a buffer whose `data.len()` does not equal `width * height * channels`. Each parameterized adjustment SHALL validate its documented ranges: Levels gamma greater than zero and input black below input white; Curves at least 2 and at most 14 points with strictly increasing inputs; Brightness/Contrast brightness in `-150..=150` and contrast in `-50..=100`; Exposure gamma greater than zero and exposure/offset finite; Posterize levels in `2..=255`; Threshold level in `1..=255`; Hue/Saturation hue in `-180..=180` and saturation/lightness in `-100..=100`; Black & White weights in `-200..=300` percent; Photo Filter density in `0..=100`; Channel Mixer weights and constant no greater than 200 percent in absolute value; Vibrance and saturation in `-100..=100`; Color Balance values finite and within `-100..=100`; Selective Color corrections in `-100..=100`; Gradient Map at least two stops with strictly increasing locations each at most 4096. `SolidFill`, `GradientFill`, and `PatternFill` have no `apply` parameter validation because `apply` SHALL refuse them as `AdjustError::Unsupported` and leave the buffer unchanged.

#### Scenario: Out-of-range parameters return an error

- **WHEN** an adjustment is given a value outside its documented range, such as Levels gamma `0.0`, Curves with one point, Brightness/Contrast brightness `200`, Exposure gamma `0.0`, Posterize `1`, Threshold `0`, Hue/Saturation hue `200`, Black & White weight `999`, Photo Filter density `150`, Channel Mixer weight `999`, Vibrance `500`, Color Balance `200`, Selective Color correction `200`, Gradient Map with one stop or a location above 4096, or any Solid Fill, Gradient Fill, or Pattern Fill
- **THEN** `apply` returns `AdjustError::InvalidParams` or `AdjustError::Unsupported` and does not panic

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives a zero-pixel buffer, a 2-channel buffer, or a buffer whose data length does not match `width * height * channels`
- **THEN** `apply` returns `AdjustError` and does not panic

### Requirement: ImageMagick differential oracle and no-equivalent classification

The system SHALL ship `scripts/adjust_oracle.py`, which applies an ImageMagick operator to a raw 8-bit image, and `crates/pictura-adjust/tests/oracle.rs`, which diffs `apply` against it. The test table SHALL have exactly one row for each destructive adjustment variant plus a no-equivalent row for the generative `GradientFill` and `PatternFill`. `SolidFill` is also refused fill content with no `apply`, so it has no oracle row; its refusal is covered by the alpha/refusal tests. Only Levels, Invert, and Desaturate SHALL be diffed (`-level ... +level ...`, `-negate`, `-modulate 100,0,100`), with tolerances of 1, 0, and 1. The other sixteen adjustments SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0: BlackWhite, PhotoFilter, GradientMap, GradientFill, PatternFill, Vibrance, ColorBalance, Auto, Curves, Exposure, BrightnessContrast, HueSaturation, ChannelMixer, SelectiveColor, Posterize, and Threshold. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table matches the implemented contract

- **WHEN** the oracle tests run
- **THEN** the mapping table has 19 rows, every no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes


## ADDED Requirements

### Requirement: Selective Color adjustment

The system SHALL implement `Adjustment::SelectiveColor(SelectiveColorParams {
method, ranges })` where `method` is `SelectiveColorMethod::Relative` or
`SelectiveColorMethod::Absolute` and `ranges` is nine `SelectiveRange` records in
reds, yellows, greens, cyans, blues, magentas, whites, neutrals, and blacks
order, each holding `c`, `m`, `y`, and `k` `i16` corrections in `-100..=100`
(default 0). `apply` SHALL reject any correction outside `-100..=100` with
`AdjustError::InvalidParams`. When all nine ranges are zero the kernel SHALL
return the buffer bit-exactly unchanged. Otherwise, per composite pixel, it
SHALL run libpsd's integer pipeline: convert RGB to an integer CMYK with
`c = (255 - r - k) * 255 / (255 - k)` and `k = min(255 - r, 255 - g, 255 - b)`
(and zero c/m/y when `k == 255`), compute the hue as an integer in `0..=359`
from the max/min channels, and copy the source ink to the destination. For the
six hue families (i = 1..=6 at model indices 0..=5) it SHALL compute the window
`r0 = -105 + i*60`, `r1 = r0 + 30`, `r2 = r1 + 30`, `r3 = r2 + 30`; when
`r0 <= hue < r3` it SHALL set the opacity to `255` inside `[r1, r2)`, to
`(hue - r0) * 255 / 30` below `r1`, and to `(r3 - hue) * 255 / 30` above `r2`,
then add, per ink component, `dst += src * corr * opacity / 25500` for Relative
and `dst += 255 * corr * opacity / 25500` for Absolute. For whites, neutrals, and
blacks (i = 7..=9 at model indices 6..=8) it SHALL select by the source black ink
(`i == 7` when `src_black == 0`, `i == 8` when `0 < src_black < 255`, `i == 9`
when `src_black == 255`) with no hue opacity and add, per ink component,
`dst += src * corr / 100` for Relative and `dst += 255 * corr / 100` for
Absolute. All divisions SHALL truncate toward zero, matching C integer division.
The destination ink SHALL be clamped to `0..=255` and converted back to RGB with
`out = (65535 - (ink * (255 - k) + (k << 8))) >> 8`. Oracle expectation: the
conversion is profile-free and its integer RGB-to-CMYK-to-RGB round-trip can
shift a channel by about 2 LSB, so there is no ImageMagick equivalent and no
Photoshop pixel-parity claim; known-value tests cover the identity, the Relative
and Absolute hue windows, and the tonal ranges.

#### Scenario: Neutral Selective Color is the exact identity

- **WHEN** Selective Color is applied with the relative method and all nine ranges zero to any buffer
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: The Relative hue window scales the existing ink

- **WHEN** Selective Color is applied with the relative method and the reds range `m = 50`, the other ranges zero, to a pixel `(200, 100, 50)`
- **THEN** the output is `(200, 61, 51)`, matching libpsd's `src * corr * opacity / 25500` on the reds/yellows overlap

#### Scenario: The Absolute hue window adds full ink

- **WHEN** Selective Color is applied with the absolute method and the reds range `y = 100`, the other ranges zero, to a pixel `(200, 100, 50)`
- **THEN** the output is `(200, 101, 1)`

#### Scenario: Blacks range lightens pure black

- **WHEN** Selective Color is applied with the relative method and the blacks range `k = -100`, the other ranges zero, to a pixel `(0, 0, 0)`
- **THEN** the output is `(255, 255, 255)`

#### Scenario: Whites range adds cyan to pure white

- **WHEN** Selective Color is applied with the absolute method and the whites range `c = 100`, the other ranges zero, to a pixel `(255, 255, 255)`
- **THEN** the output is `(1, 255, 255)`

#### Scenario: Out-of-range corrections are rejected

- **WHEN** a `SelectiveRange` carries a correction above 100 or below -100
- **THEN** `apply` returns `AdjustError::InvalidParams` and does not panic
