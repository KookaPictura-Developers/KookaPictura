# image-resize Specification

## Purpose
Image resize with resample kernels and per-channel sampling, checked against the ImageMagick oracle.
## Requirements
### Requirement: Image resize entry point and error contract

The system SHALL provide `pictura_ops::resize(buf: &PixelBuffer, width: u32, height: u32, method: Resample) -> Result<PixelBuffer, OpsError>`. It SHALL return a newly allocated `PixelBuffer` whose `width` and `height` equal the requested values and whose `channels` equals the input's, and it MUST leave `buf` bit-identical. `width` or `height` below 1, or an input whose `data.len()` does not equal `width * height * channels`, SHALL be rejected with `OpsError::InvalidParams` before any sample is written. The function MUST NOT panic for any input, including 1×1 images and 1-pixel-wide or 1-pixel-tall images.

#### Scenario: Resize returns a new buffer and leaves the input untouched

- **WHEN** `resize` is called on a valid `PixelBuffer`
- **THEN** it returns `Ok` with a buffer of the requested width and height, the same channel count, and the input buffer still equals its pre-call bytes

#### Scenario: Invalid dimensions are rejected

- **WHEN** `resize` is called with `width` 0 or `height` 0
- **THEN** it returns `OpsError::InvalidParams` and the input buffer is unchanged

#### Scenario: A malformed input buffer errors instead of panicking

- **WHEN** `resize` is called on a buffer whose `data.len()` is not `width * height * channels`
- **THEN** it returns `OpsError::InvalidParams` and does not panic

#### Scenario: Tiny images do not panic

- **WHEN** `resize` is called from a 1×1 buffer and from a 1×N or N×1 buffer to any valid target size
- **THEN** no call panics and each returns `Ok` or `OpsError::InvalidParams`

### Requirement: Resample kernels and per-channel sampling

`Resample` SHALL have exactly the variants `Nearest`, `Bilinear`, and `Bicubic`, and `resize` SHALL resample every channel, including any alpha channel, with the selected kernel. `Nearest` SHALL select the single nearest source sample, so an upscaled two-color image contains only the original colors. `Bilinear` SHALL blend the 2×2 neighborhood, and `Bicubic` SHALL convolve the 4×4 neighborhood with a Keys cubic kernel whose coefficient is `a = -0.75`, i.e. a Mitchell–Netravali `cubic(B = 0, C = 0.75)` kernel matching the publicly documented Photoshop Bicubic (not Catmull-Rom `a = -0.5`). Every sample SHALL clamp to the image edge, mapping an out-of-range source coordinate to the nearest in-bounds sample. `Bicubic` MUST NOT panic or read out of bounds when the source is smaller than the 4×4 support.

#### Scenario: Nearest preserves source colors

- **WHEN** `resize` with `Resample::Nearest` upscales a two-color image
- **THEN** every output sample is one of the two source colors and no intermediate value appears

#### Scenario: Bilinear and Bicubic smooth gradients

- **WHEN** `resize` with `Resample::Bilinear` and `Resample::Bicubic` upscales a linear gradient
- **THEN** both produce monotone intermediate values and Bicubic is no rougher than Nearest on the same input

#### Scenario: Edge sampling clamps

- **WHEN** `resize` scales a 1-pixel-wide or 1-pixel-tall buffer
- **THEN** edge sampling repeats the edge sample and no out-of-bounds read or panic occurs

#### Scenario: Alpha is resampled with the color channels

- **WHEN** `resize` is applied to a 4-channel buffer
- **THEN** channel 4 is resampled by the same kernel as the color channels and the output has the same 4-channel planar layout

### Requirement: Image resize ImageMagick differential oracle

The system SHALL ship `scripts/ops_oracle.py`, which applies an ImageMagick
resize operator to a raw 8-bit image, and `crates/pictura-ops/tests/oracle.rs`,
which diffs `resize` against it. The mapping SHALL map `Resample::Nearest` to
ImageMagick `-filter point`, `Resample::Bilinear` to `-filter triangle`, and
`Resample::Bicubic` to `-filter cubic` with `-define filter:b=0 -define
filter:c=0.75` (the same `cubic(0, 0.75)` kernel), and SHALL record the measured
maximum and mean per-sample delta for each. Nearest SHALL match `-filter point`
exactly; Bilinear SHALL match `-filter triangle` exactly on upscale, while
downscale is no-equivalent because ImageMagick widens the filter support
(anti-aliases) and Pictura samples a fixed footprint. Bicubic SHALL use a
tolerance justified by the measured delta against the defined cubic. Because
ImageMagick's edge and phase handling need not match Adobe's closed kernels, the
differential test SHALL use the measured tolerance, and any method or scale that
cannot be matched SHALL be classified no-equivalent with the observed delta
recorded. The differential tests SHALL skip with a message when `magick` is not
on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Each resample method has a measured mapping row

- **WHEN** the oracle mapping table is checked
- **THEN** `Nearest`, `Bilinear`, and `Bicubic` each have a row naming the ImageMagick operator, a tolerance justified by the measured delta, and a non-empty note

#### Scenario: Bicubic maps to the defined cubic kernel

- **WHEN** the Bicubic mapping row is checked
- **THEN** it names `-filter cubic` with `-define filter:b=0 -define filter:c=0.75` and records the measured maximum delta

#### Scenario: Downscaled Bilinear is classified no-equivalent

- **WHEN** `Resample::Bilinear` is compared with `-filter triangle` at a scale below 1
- **THEN** the observed delta is recorded as no-equivalent rather than asserted equal, because ImageMagick anti-aliases by widening the kernel

