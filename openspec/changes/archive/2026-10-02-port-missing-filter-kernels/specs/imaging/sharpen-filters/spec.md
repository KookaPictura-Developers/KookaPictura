## ADDED Requirements

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
