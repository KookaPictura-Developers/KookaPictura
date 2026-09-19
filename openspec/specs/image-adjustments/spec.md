# image-adjustments Specification

## Purpose
TBD - created by archiving change m4-image-adjustments. Update Purpose after archive.
## Requirements
### Requirement: Destructive adjustment application contract

The system SHALL provide `pictura_adjust::apply(adjustment: &Adjustment, buf: &mut PixelBuffer) -> Result<(), AdjustError>`. It SHALL operate in place on a planar 8-bit buffer whose `channels` is 3 (RGB) or 4 (RGBA), treating channels 1, 2, and 3 as the composite color planes and channel 4, when present, as alpha. It SHALL transform every color sample or return an error; it MUST NOT partially apply an adjustment and then fail.

#### Scenario: Apply to a 3-channel planar buffer

- **WHEN** `apply` receives any supported `Adjustment` and a 3-channel planar buffer
- **THEN** the R, G, and B planes are rewritten in place and `Ok(())` is returned

#### Scenario: Reject an unsupported channel count

- **WHEN** the buffer has a channel count other than 3 or 4
- **THEN** `apply` returns `AdjustError::Unsupported` and leaves the buffer unchanged

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

### Requirement: Deterministic application

`apply` SHALL be deterministic. The same adjustment applied to equal input buffers SHALL produce bit-identical output every run, with no random, time, or thread-order dependence.

#### Scenario: Repeated runs match

- **WHEN** the same adjustment is applied to two clones of one buffer
- **THEN** the two output buffers are equal, including for `Auto(Color)` and `HueSaturation`

### Requirement: Levels adjustment

The system SHALL implement `Adjustment::Levels(LevelsParams { input_black, input_white, gamma, output_black, output_white })`, applying, per composite color channel, `t = clamp((v - input_black) / (input_white - input_black), 0, 1)`, then `t' = t^(1/gamma)`, then `out = output_black + t' * (output_white - output_black)`. Defaults SHALL be input black 0, input white 255, gamma 1.0, output black 0, output white 255; the documented input and output levels span 0 through 253 (scripting-DOM clamp) and gamma spans 0.10 through 9.99. Oracle expectation: differential against ImageMagick `-level B%,W%,gamma +level Ob%,Ow%` within an absolute tolerance of 1/255 per sample.

#### Scenario: Gamma above one lightens midtones

- **WHEN** Levels is applied with input black 0, input white 255, gamma 2.0 to a mid-grey sample
- **THEN** the output is lighter than the input

#### Scenario: Input and output points remap and clamp

- **WHEN** Levels maps input black 5 to output black 20 and input white 243 to output white 235
- **THEN** samples at or below 5 read 20, samples at or above 243 read 235, and every output stays within 20 through 235

#### Scenario: Matches ImageMagick

- **WHEN** Levels with gamma 2.0 is applied to the 8x8 oracle image and ImageMagick applies `-level 0%,100%,2.0 +level 0%,100%`
- **THEN** no sample differs by more than 1

### Requirement: Curves adjustment

The system SHALL implement `Adjustment::Curves(CurvesParams { points })` with 2 through 14 control points in `(input, output)` order, where inputs strictly increase. The curve SHALL be a monotone cubic Hermite interpolation (Fritsch-Carlson tangents) evaluated into a 256-entry LUT applied per composite color channel. Two endpoints `(0, 0)` and `(255, 255)` SHALL be the exact identity. Oracle expectation: no faithful ImageMagick operator exists, so the behavior is covered by known-value and property tests (identity, control points, monotonicity) rather than a differential test.

#### Scenario: Two-point curve is the identity

- **WHEN** Curves is applied with points `[(0, 0), (255, 255)]`
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: A control point is honored

- **WHEN** Curves is applied with points `[(0, 0), (100, 200), (255, 255)]` to a sample at 100
- **THEN** the output sample is 200

#### Scenario: Output is monotone

- **WHEN** Curves is applied to a 0 to 255 ramp with any valid monotone point set
- **THEN** output samples never decrease as the input increases

### Requirement: Brightness/Contrast adjustment

The system SHALL implement `Adjustment::BrightnessContrast(BrightnessContrastParams { brightness, contrast, use_legacy })`. Brightness SHALL default to 0 and span `-150..=150`; contrast SHALL default to 0 and span `-50..=100`; `use_legacy` SHALL default to false. Legacy mode SHALL apply an additive brightness shift then a linear contrast expansion about mid-grey with hard clipping; modern mode SHALL apply a gamma-shaped brightness change and a monotone S-curve through `(0,0)`, `(0.5,0.5)`, `(1,1)` whose central slope is `1 + contrast/100`. Zero brightness and zero contrast SHALL be an exact identity in both modes. Oracle expectation: ImageMagick `-brightness-contrast` diverges (observed max delta 14), so the behavior is covered by property tests for identity, monotonicity in brightness, and mean shift.

#### Scenario: Neutral parameters are identity

- **WHEN** Brightness/Contrast is applied with brightness 0 and contrast 0, in either mode
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Legacy brightness clips, modern contrast stays soft

- **WHEN** legacy brightness +20 is applied to a sample at 250
- **THEN** the output clips to 255
- **WHEN** modern contrast +50 is applied to a 0 to 255 ramp
- **THEN** the endpoints stay 0 and 255, shadows darken, highlights lighten, and the result is monotone

### Requirement: Exposure adjustment

The system SHALL implement `Adjustment::Exposure(ExposureParams { exposure, offset, gamma })` in linear light. It SHALL decode each 8-bit sample from sRGB to linear, apply `c * 2^exposure`, then `+ offset`, then raise to `gamma`, then re-encode to sRGB. Exposure SHALL default to 0.0 EV and span `-20.0..=20.0`; offset SHALL default to 0.0 and span `-0.5..=0.5`; gamma SHALL default to 1.0 and span 0.01 through 9.99. The identity tuple SHALL return before any math. Oracle expectation: Photoshop applies the transfer in linear light while ImageMagick operators run in encoded space, so no differential test is attempted; property tests cover the identity and the one-EV doubling of linear light.

#### Scenario: Identity exposure is bit-exact

- **WHEN** Exposure is applied with exposure 0.0, offset 0.0, and gamma 1.0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: One EV doubles linear light

- **WHEN** Exposure +1.0 is applied to a mid-grey sample
- **THEN** the decoded linear value of the output is approximately twice the decoded linear value of the input

### Requirement: Invert adjustment

The system SHALL implement `Adjustment::Invert`, computing `out = 255 - v` independently per composite color channel with no parameters. Applying Invert twice SHALL restore the original buffer. Oracle expectation: differential against ImageMagick `-negate` with tolerance 0.

#### Scenario: Known values and involution

- **WHEN** Invert is applied to `(0, 5, 200)`
- **THEN** the result is `(255, 250, 55)`, and applying Invert again restores `(0, 5, 200)`

#### Scenario: Matches ImageMagick

- **WHEN** Invert is applied to the 8x8 oracle image and ImageMagick applies `-negate`
- **THEN** the results are identical

### Requirement: Posterize adjustment

The system SHALL implement `Adjustment::Posterize(levels: u8)` with `levels` in `2..=255`, default 4. Per composite color channel it SHALL set `q = round(v * (levels - 1) / 255)` and output `q * 255 / (levels - 1)`. Levels 255 SHALL be the 8-bit identity and a no-op. The operation SHALL be idempotent. Oracle expectation: ImageMagick `-posterize` bins on an adjacent level (observed max delta 85), so a property test asserts every output is an allowed quantization level and that levels 255 is the identity.

#### Scenario: Quantizes to the target levels

- **WHEN** Posterize 4 is applied to a sample at 64
- **THEN** the output is 85, one of `{0, 85, 170, 255}`

#### Scenario: 255 levels is the identity

- **WHEN** Posterize 255 is applied to a 0 to 255 ramp
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Re-applying does not change output

- **WHEN** Posterize 4 is applied twice to the same buffer
- **THEN** the second application leaves the result unchanged

### Requirement: Threshold adjustment

The system SHALL implement `Adjustment::Threshold(level: u8)` with `level` in `1..=255`, default 128. It SHALL compute composite luminance `Y = 0.299*R + 0.587*G + 0.114*B` (Rec.601) and write 255 to all three color channels when `Y > level`, otherwise 0, so the output is neutral. Equality SHALL fall to black. Oracle expectation: ImageMagick `-threshold` uses Rec.709 luma and diverges up to 255, so property tests assert the output is a subset of `{0, 255}` and that the split is monotone in the level.

#### Scenario: Binarizes by luma

- **WHEN** Threshold 128 is applied to pure red (luma 76), black, white, and mid-grey
- **THEN** the outputs are black, black, white, and black respectively

#### Scenario: Level zero is rejected

- **WHEN** Threshold 0 is applied
- **THEN** `apply` returns `AdjustError::InvalidParams`

### Requirement: Desaturate adjustment

The system SHALL implement `Adjustment::Desaturate` with no parameters. Per pixel it SHALL compute `(min(R, G, B) + max(R, G, B)) / 2`, rounded, and write that value to all three color channels, which is HSL lightness with saturation zero. It SHALL be equivalent to Hue/Saturation with saturation -100 within 1 LSB. Oracle expectation: differential against ImageMagick `-modulate 100,0,100` with tolerance 1.

#### Scenario: Known value and neutral passthrough

- **WHEN** Desaturate is applied to `(12, 104, 22)` and to `(77, 77, 77)`
- **THEN** the outputs are `(58, 58, 58)` and `(77, 77, 77)` respectively

#### Scenario: Matches ImageMagick

- **WHEN** Desaturate is applied to the 8x8 oracle image and ImageMagick applies `-modulate 100,0,100`
- **THEN** no sample differs by more than 1

### Requirement: Auto adjustments

The system SHALL implement `Adjustment::Auto(AutoKind::Tone | AutoKind::Contrast | AutoKind::Color)`. Tone SHALL stretch each color channel independently between percentile bounds that discard 0.1 percent of the population at each extreme. Contrast SHALL stretch all three channels by one joint pair of bounds that discards 0.5 percent, preserving channel ratios so no cast is introduced. Color SHALL apply the per-channel stretch with 0.5 percent clipping, then scale each channel's midtone gamma so the mean near-neutral midtone converges to the shared mean. Bounds SHALL be derived from the buffer histogram. Oracle expectation: Adobe's Auto Color Correction solver is closed, so behavior is covered by property tests (per-channel stretch, joint no-cast, midtone neutralization).

#### Scenario: Auto Tone stretches the range

- **WHEN** Auto Tone is applied to a ramp whose values span 60 through 200
- **THEN** the dark end maps near 0 and the light end maps near 255

#### Scenario: Auto Contrast does not introduce a cast

- **WHEN** Auto Contrast is applied to a neutral grey ramp
- **THEN** every output pixel keeps R equal to G equal to B

#### Scenario: Auto Color pulls channels toward neutral

- **WHEN** Auto Color is applied to a ramp with a fixed channel offset
- **THEN** the per-channel sums move closer together than before

### Requirement: Hue/Saturation adjustment

The system SHALL implement `Adjustment::HueSaturation(HueSaturationParams { hue, saturation, lightness })`. Hue SHALL default to 0 and span `-180..=180`; saturation SHALL default to 0 and span `-100..=100`; lightness SHALL default to 0 and span `-100..=100`. It SHALL convert each pixel to HSL, rotate hue by the hue value, scale saturation by `1 + saturation/100`, adjust lightness toward white for positive values and toward black for negative values, and convert back. All-zero parameters SHALL be an exact identity, and saturation -100 SHALL produce a neutral grey at the pixel's HSL lightness. This change covers the composite (Master) path only. Oracle expectation: ImageMagick `-modulate` uses its own HSL space (observed max delta 45), so property tests cover the identity and increased channel spread under positive saturation.

#### Scenario: Neutral parameters are identity

- **WHEN** Hue/Saturation is applied with hue 0, saturation 0, and lightness 0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Saturation minus 100 is grey

- **WHEN** Hue/Saturation is applied with saturation -100
- **THEN** all three color channels of every pixel are equal to the rounded HSL lightness

### Requirement: Black & White adjustment

The system SHALL implement `Adjustment::BlackWhite(BlackWhiteParams { red, yellow, green, cyan, blue, magenta, tint, tint_color })`. The six weights SHALL be percentages in `-200..=300` with defaults red 40, yellow 60, green 40, cyan 60, blue 20, magenta 80. It SHALL decompose each pixel into a neutral component plus a hue-sector pair and apply the matching weights so the result is a pure grey. When `tint` is set, it SHALL map the grey through the tint colour's hue and saturation; a black tint colour SHALL be rejected. Oracle expectation: no faithful ImageMagick operator exists, so known-value tests cover the neutral passthrough, the red weight, and tinting.

#### Scenario: Neutral grey passes through

- **WHEN** Black & White is applied with the default weights to a neutral grey `(100, 100, 100)`
- **THEN** the output is `(100, 100, 100)`

#### Scenario: Pure red uses the red weight

- **WHEN** the default weights are applied to `(200, 0, 0)`
- **THEN** the output is grey 80

#### Scenario: Tint shifts the grey

- **WHEN** tint is enabled with a sepia tint colour and the result is applied to a neutral grey
- **THEN** the red channel is greater than the blue channel

### Requirement: Photo Filter adjustment

The system SHALL implement `Adjustment::PhotoFilter(PhotoFilterParams { color, density, preserve_luminosity })`. Density SHALL default to 25 and span `0..=100`; `preserve_luminosity` SHALL default to true. Per pixel it SHALL compute `C' = C * (1 - d) + F * d` where `d = density/100` and `F` is the filter colour, then rescale toward the input luminance when `preserve_luminosity` is set. Density 0 SHALL be a no-op. Oracle expectation: no faithful ImageMagick operator exists, so property tests cover warming, luminosity preservation, and the density-zero no-op.

#### Scenario: Density zero is a no-op

- **WHEN** Photo Filter is applied with density 0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Warming filter preserves luminosity

- **WHEN** a warm filter `(255, 180, 80)` is applied at density 25 with luminosity preservation to neutral greys
- **THEN** red exceeds blue and each output luminance is within 3 of the input

### Requirement: Channel Mixer adjustment

The system SHALL implement `Adjustment::ChannelMixer(ChannelMixerParams { monochrome, red, green, blue, constant })`. Source weights and constants SHALL be percentages no greater than 200 in absolute value, with the identity matrix `[100, 0, 0]`, `[0, 100, 0]`, `[0, 0, 100]` and zero constants. Per output channel it SHALL compute `out = (w[0]*R + w[1]*G + w[2]*B) / 100 + constant/100 * 255`, clamped. Monochrome SHALL use the red row as one source mix for all three outputs. Oracle expectation: ImageMagick `-color-matrix` takes fractions while Photoshop uses percent weights (observed max delta 252), so property tests cover the exact identity and a hand-computed blend.

#### Scenario: Identity matrix is a no-op

- **WHEN** the identity matrix is applied
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Known blend

- **WHEN** the red row is set to sources `[50, 100, 0]` and the pixel is `(50, 100, 200)`
- **THEN** the red output is 125 and green and blue are unchanged

### Requirement: Vibrance adjustment

The system SHALL implement `Adjustment::Vibrance(VibranceParams { vibrance, saturation })`. Both values SHALL default to 0 and span `-100..=100`. It SHALL convert each pixel to HSL, apply the saturation scale `S * (1 + saturation/100)`, and add a vibrance delta that falls off as `1 - S` and is damped in the skin-tone hue band, then clamp saturation to `[0, 1]`. Saturation -100 SHALL produce a neutral grey. Oracle expectation: no faithful ImageMagick operator exists, so property tests cover the identity, the diminishing boost, and saturation -100.

#### Scenario: Neutral parameters are identity

- **WHEN** Vibrance is applied with vibrance 0 and saturation 0
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Saturation minus 100 is grey

- **WHEN** Vibrance is applied with saturation -100
- **THEN** all three color channels of the pixel are equal

### Requirement: Color Balance adjustment

The system SHALL implement `Adjustment::ColorBalance(ColorBalanceParams { shadows, midtones, highlights, preserve_luminosity })`. Each tonal band SHALL hold three shifts in the order cyan-red, magenta-green, yellow-blue, each defaulting to 0 and spanning `-100..=100`. Per pixel it SHALL compute `Y = 0.299*R + 0.587*G + 0.114*B`, weight the three bands with overlapping parabola windows that peak at `Y = 0`, `Y = 0.5`, and `Y = 1`, add the weighted per-channel shifts, and clamp. `preserve_luminosity` SHALL default to true and rescale the result toward the input luminance. A midtone-only shift SHALL leave black untouched. Oracle expectation: no faithful ImageMagick operator exists, so property tests cover the midtone red shift and luminosity preservation.

#### Scenario: Midtone shift moves grey toward red

- **WHEN** a midtones red shift of +100 is applied to mid-grey
- **THEN** the red channel increases and black is unchanged

#### Scenario: Luminosity is preserved

- **WHEN** a midtone shift is applied with `preserve_luminosity` set to true to mid-grey
- **THEN** the output luminance is within 3 of the input

### Requirement: ImageMagick differential oracle and no-equivalent classification

The system SHALL ship `scripts/adjust_oracle.py`, which applies an ImageMagick operator to a raw 8-bit image, and `crates/pictura-adjust/tests/oracle.rs`, which diffs `apply` against it. The test table SHALL have exactly one row for each destructive adjustment variant plus a no-equivalent row for the generative `GradientFill`. Only Levels, Invert, and Desaturate SHALL be diffed (`-level ... +level ...`, `-negate`, `-modulate 100,0,100`), with tolerances of 1, 0, and 1. The other fourteen adjustments SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0: BlackWhite, PhotoFilter, GradientMap, GradientFill, Vibrance, ColorBalance, Auto, Curves, Exposure, BrightnessContrast, HueSaturation, ChannelMixer, Posterize, and Threshold. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table matches the implemented contract

- **WHEN** the oracle tests run
- **THEN** the mapping table has 17 rows, every no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes

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

