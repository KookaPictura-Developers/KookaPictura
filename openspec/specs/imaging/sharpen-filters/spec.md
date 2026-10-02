# sharpen-filters Specification

## Purpose
Sharpen, Sharpen More, Sharpen Edges, and Unsharp Mask with clamped parameters and an oracle.

## Requirements

### Requirement: Sharpen filter application and error contract

The system SHALL provide `pictura_filters::apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError>`. It SHALL operate in place on a planar 8-bit buffer whose `channels` is 3 (RGB) or 4 (RGBA), treating channels 1 through 3 as the color planes and channel 4, when present, as alpha. It SHALL transform every color sample or return an error; it MUST NOT partially apply a filter and then fail. Malformed buffers MUST return `FilterError` instead of panicking.

#### Scenario: Apply a sharpen to a 3-channel planar buffer

- **WHEN** `apply` receives a sharpen `Filter` variant and a 3-channel planar buffer
- **THEN** the R, G, and B planes are rewritten in place and `Ok(())` is returned

#### Scenario: Reject an unsupported channel count

- **WHEN** the buffer has a channel count other than 3 or 4
- **THEN** `apply` returns `FilterError::Unsupported` and leaves the buffer unchanged

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives an empty buffer, or a buffer whose `data.len()` does not equal `width * height * channels`, or a buffer with zero width or height
- **THEN** `apply` returns a `FilterError` and does not panic

### Requirement: Alpha channel preservation for sharpen

For 4-channel buffers, every sharpen filter SHALL leave channel 4 bit-identical. Only channels 1 through 3 SHALL be modified.

#### Scenario: Every sharpen variant preserves alpha

- **WHEN** each sharpen variant is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Sharpen and Sharpen More

The system SHALL implement `Filter::Sharpen` and `Filter::SharpenMore` with no parameters, as a fixed 3×3 high-pass convolution of the form `[0 −1 0; −1 5 −1; 0 −1 0]`, where `SharpenMore` uses the same kernel shape with a stronger fixed gain. A uniform-color buffer SHALL be unchanged. `SharpenMore`'s deviation from the input SHALL be at least as large as `Sharpen`'s on the same edge image. Oracle expectation: no faithful ImageMagick equivalent exists (fixed kernels), so known-value tests cover the known kernel response and the documented divergence.

#### Scenario: The kernel response is a known value

- **WHEN** `Sharpen` is applied to a buffer whose center pixel sits on an edge
- **THEN** the center output equals the center boosted by the local high-pass response, and a flat neighborhood is unchanged

#### Scenario: Sharpen More is stronger

- **WHEN** `Sharpen` and `SharpenMore` are each applied to the same edge image
- **THEN** the total deviation from the input is larger for `SharpenMore` than for `Sharpen`

### Requirement: Sharpen Edges

The system SHALL implement `Filter::SharpenEdges` with no parameters. It SHALL detect edges by gradient magnitude and apply the high-pass gain only where the gradient exceeds a fixed internal threshold, leaving flat regions unchanged. Oracle expectation: no faithful ImageMagick equivalent exists (a fixed-threshold edge gate), so property tests cover the flat-region no-op and the edge contrast increase, and the divergence is documented.

#### Scenario: Flat regions are unchanged

- **WHEN** `SharpenEdges` is applied to a buffer with a flat area
- **THEN** the flat-area samples are bit-exactly unchanged

#### Scenario: Edge contrast increases

- **WHEN** `SharpenEdges` is applied to a step edge
- **THEN** the contrast across the edge increases relative to the input

### Requirement: Unsharp Mask

The system SHALL implement `Filter::UnsharpMask { amount: f64, radius: f64, threshold: u8 }` as `out = original + (original − blurred) × amount`, where `blurred` is the shared Gaussian kernel from the blur family and the difference is gated by `threshold`. `amount` SHALL default to 100 and span `1.0..=500.0` percent; `radius` SHALL default to 1.0 and span `0.1..=250.0`; `threshold` SHALL default to 0 and span `0..=255`. The gate SHALL leave a pixel unchanged when its absolute difference from the blurred value is below `threshold`, and `threshold == 0` SHALL sharpen every pixel. `amount` outside `1.0..=500.0` and `radius <= 0` SHALL be rejected with `FilterError::InvalidParams`. Oracle expectation: differential against ImageMagick `-unsharp 0xRxA+T` within an absolute tolerance of 6 per sample (ImageMagick's internal blur differs slightly), with any radius-to-sigma mapping divergence documented.

#### Scenario: A uniform image is unchanged

- **WHEN** Unsharp Mask is applied to a uniform-color buffer
- **THEN** the buffer is bit-exactly unchanged within 1 LSB

#### Scenario: Overshoot grows with amount

- **WHEN** Unsharp Mask is applied to a step edge at increasing amounts with a fixed radius
- **THEN** the edge overshoot grows with the amount

#### Scenario: The threshold gates low-contrast pixels

- **WHEN** two adjacent pixels differ by less than the threshold
- **THEN** they are unchanged, and when they differ by the threshold or more they are sharpened

#### Scenario: Out-of-range amount or radius is rejected

- **WHEN** Unsharp Mask is applied with amount 0 or radius 0
- **THEN** `apply` returns `FilterError::InvalidParams` and does not panic

#### Scenario: Matches ImageMagick

- **WHEN** Unsharp Mask with a given amount, radius, and threshold is applied to the oracle image and ImageMagick applies the matching `-unsharp 0xRxA+T`
- **THEN** no sample differs by more than 6, or the divergence is recorded in the no-equivalent documentation

### Requirement: Sharpen parameter validation and clamping

Each parameterized sharpen filter SHALL reject out-of-range or non-finite parameters with `FilterError::InvalidParams` instead of panicking. Sharpen output SHALL be clamped only at the final 8-bit store; it MUST NOT wrap around on overflow or underflow. Each filter SHALL transform every color sample or return an error, never a partial result.

#### Scenario: Out-of-range Unsharp Mask parameters return an error

- **WHEN** Unsharp Mask is applied with amount 0, amount 501, radius 0, or radius 251
- **THEN** `apply` returns `FilterError::InvalidParams` and does not panic

#### Scenario: Overflow clamps instead of wrapping

- **WHEN** sharpening pushes a sample above 255 or below 0
- **THEN** the output clamps to 255 or 0 and does not wrap to the opposite extreme

### Requirement: Clamp-to-edge borders and tiny images for sharpen

Every sharpen kernel SHALL sample with clamp-to-edge at the image borders and SHALL clamp its kernel support to the available source. A 1×1 image and a 1-pixel-wide or 1-pixel-tall image MUST NOT panic and MUST return either a result or a typed error.

#### Scenario: A tiny image does not panic

- **WHEN** every sharpen variant is applied to a 1×1 buffer and to a 1-pixel-wide buffer
- **THEN** no variant panics and each returns `Ok(())` or a `FilterError`

#### Scenario: Border pixels clamp to the edge

- **WHEN** a sharpen filter is applied to a small buffer and a pixel at the border is inspected
- **THEN** the result is consistent with repeating the edge sample rather than reading out of bounds

### Requirement: Deterministic sharpen output

Sharpen filters SHALL be deterministic. The same filter applied to equal input buffers SHALL produce bit-identical output every run, with no random, time, or thread-order dependence.

#### Scenario: Repeated runs match

- **WHEN** each sharpen variant is applied to two clones of one buffer
- **THEN** the two output buffers are bit-identical

### Requirement: Sharpen ImageMagick oracle and no-equivalent classification

The system SHALL ship `scripts/filter_oracle.py` and `crates/pictura-filters/tests/oracle.rs`. The mapping table SHALL have exactly one row per `Filter` variant. Unsharp Mask SHALL be diffed against `-unsharp 0xRxA+T` within its stated tolerance. Sharpen, SharpenMore, and SharpenEdges SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0 with known-value or property tests, including the USM-not-an-edge-detector behavior. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table covers every sharpen variant

- **WHEN** the oracle tests run
- **THEN** every sharpen variant has a table row, each no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes

### Requirement: Smart Sharpen

The system SHALL implement `Filter::SmartSharpen { amount: f64, radius: f64, reduce_noise: f64, remove: SharpenRemove, angle: f64, more_accurate: bool, shadow: TonalFade, highlight: TonalFade }`
with `SharpenRemove::{GaussianBlur, LensBlur, MotionBlur}` and
`TonalFade { amount: u8, width: u8, radius: u32 }`, computing
`out = original + (original − blurred) × amount/100 × gain`, where the detail
`d = original − blurred` and `gain = d²/(d² + floor²)` with
`floor = reduce_noise/100 × 26`; `gain` SHALL be exactly 1 when `reduce_noise`
is 0. The blur model SHALL follow `remove`: `GaussianBlur` uses the shared
Gaussian kernel (the Unsharp Mask method), `LensBlur` uses a circular (disc)
summed-area mean, and `MotionBlur` uses the shared motion blur along `angle`
with distance `2·radius`, following Photoshop's counter-clockwise screen-angle
convention (`+angle` smears toward the upper right in this y-down buffer).
`amount` SHALL span `1.0..=500.0` percent, where 0 is
rejected rather than a no-op; `radius` SHALL be finite and within `0.1..=64.0`;
`reduce_noise` SHALL span `0..=100`; and `angle` SHALL be finite and within
`-360..=360`; a value outside those ranges SHALL be rejected with
`FilterError::InvalidParams` before any mutation. Alpha SHALL be left bit-identical and the output SHALL be
deterministic. Oracle expectation: with `reduce_noise` 0, `more_accurate: false`,
both fades at `amount` 0, and `GaussianBlur` the
result is byte-identical to `unsharp_mask(amount, radius, 0)`, so it is covered
by the existing Unsharp Mask ImageMagick differential (absolute tolerance 6);
`LensBlur` and `MotionBlur` are classified as no-equivalent with tolerance 0 and
covered by property tests, with the closed-PSF divergence documented.

#### Scenario: A uniform image is unchanged

- **WHEN** Smart Sharpen is applied to a uniform-color buffer with any valid amount
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: Amount 0 is rejected

- **WHEN** Smart Sharpen is applied with `amount: 0.0`
- **THEN** `apply` returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

#### Scenario: Gaussian Blur matches Unsharp Mask

- **WHEN** Smart Sharpen with `remove: GaussianBlur` and `reduce_noise: 0.0` and Unsharp Mask with threshold 0 are applied to the same RGBA image at the same amount and radius
- **THEN** the two outputs are byte-identical

#### Scenario: Lens Blur and Motion Blur change the result

- **WHEN** Smart Sharpen with `remove: LensBlur` or `remove: MotionBlur` is applied to an edge image
- **THEN** the output differs from the `GaussianBlur` run at the same amount and radius

#### Scenario: Reduce Noise holds back low-contrast detail

- **WHEN** Smart Sharpen is applied to a low-contrast edge with `reduce_noise: 0.0` and with `reduce_noise: 100.0`
- **THEN** the total change from the input is smaller in the `reduce_noise: 100.0` run

#### Scenario: Motion Blur follows Photoshop's angle

- **WHEN** Smart Sharpen with `remove: MotionBlur` and `angle: 45` is applied to a single bright pixel
- **THEN** the smear runs lower-left to upper-right, matching Photoshop's counter-clockwise angle

#### Scenario: Out-of-range parameters are rejected untouched

- **WHEN** Smart Sharpen is applied with amount below 1.0 or above 500.0, radius below 0.1 or above 64.0, `reduce_noise` above 100, or a non-finite angle
- **THEN** `apply` returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

#### Scenario: Output is deterministic and alpha is preserved

- **WHEN** Smart Sharpen is applied twice to equal RGBA buffers
- **THEN** the two outputs are bit-identical and the alpha plane equals the input alpha plane bit for bit

### Requirement: Smart Sharpen More Accurate

`more_accurate` SHALL select a higher-fidelity blur estimate for the chosen
`remove` in place of the default path. When `more_accurate` is true, the output
SHALL differ from the same call with `more_accurate` false for every `remove` on
a detail image, and the result SHALL be deterministic and leave alpha
bit-identical. The default path (`more_accurate: false`) SHALL be unchanged, so
the Gaussian/Unsharp-Mask equivalence above SHALL hold only with
`more_accurate: false`.

#### Scenario: More Accurate changes every removal path

- **WHEN** Smart Sharpen with `remove: GaussianBlur`, `remove: LensBlur`, or `remove: MotionBlur` is applied to a detail image with `more_accurate: true` and with `more_accurate: false`
- **THEN** the `true` output differs from the `false` output in every case

#### Scenario: More Accurate is deterministic and preserves alpha

- **WHEN** Smart Sharpen with `more_accurate: true` is applied twice to equal RGBA buffers
- **THEN** the two outputs are bit-identical and the alpha plane equals the input alpha plane bit for bit

### Requirement: Smart Sharpen Shadow/Highlight tonal fade

`shadow` and `highlight` SHALL each be a `TonalFade { amount: u8, width: u8, radius: u32 }` that scales the sharpening contribution within the shadow (or
highlight) tonal band, in the spirit of Blend If. `TonalFade::default()` SHALL be
`{ amount: 0, width: 50, radius: 1 }`. `amount` and `width` SHALL span `0..=100`
and `radius` SHALL span `1..=100`; a value outside those ranges SHALL be rejected
with `FilterError::InvalidParams` before any mutation. `amount` 0 SHALL be a
no-op. A high `shadow.amount` SHALL reduce the change in dark tones relative to a
low one, and `highlight.amount` SHALL act analogously in light tones.

#### Scenario: Zero fade amount is a no-op

- **WHEN** Smart Sharpen is applied with `shadow.amount: 0` and `highlight.amount: 0` and again with the default `TonalFade` values
- **THEN** the two outputs are bit-identical

#### Scenario: Shadow fade dampens dark-tone sharpening

- **WHEN** Smart Sharpen is applied to a dark edge with a high `shadow.amount` and with a low one
- **THEN** the total change in the dark tones is smaller under the high `shadow.amount`

#### Scenario: Highlight fade dampens light-tone sharpening

- **WHEN** Smart Sharpen is applied to a bright edge with a high `highlight.amount` and with a low one
- **THEN** the total change in the light tones is smaller under the high `highlight.amount`

#### Scenario: Out-of-range tonal parameters are rejected untouched

- **WHEN** Smart Sharpen is applied with `shadow.amount` or `shadow.width` above 100, `shadow.radius` below 1 or above 100, or the analogous `highlight` values out of range
- **THEN** `apply` returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state
