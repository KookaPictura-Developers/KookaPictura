# distort-filters Specification

## Purpose
TBD - created by archiving change m9-distort. Update Purpose after archive.
## Requirements
### Requirement: Distort filter application and error contract

The system SHALL provide `pictura_filters::apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError>`. It SHALL operate in place on a planar 8-bit buffer whose `channels` is 3 (RGB) or 4 (RGBA), treating channels 1 through 3 as the color planes and channel 4, when present, as alpha. It SHALL transform every color sample or return an error; it MUST NOT partially apply a filter and then fail. Malformed buffers MUST return `FilterError` instead of panicking.

#### Scenario: Apply a Distort filter to a 3-channel planar buffer

- **WHEN** `apply` receives one of the five Distort `Filter` variants and a 3-channel planar buffer
- **THEN** the R, G, and B planes are rewritten in place and `Ok(())` is returned

#### Scenario: Reject an unsupported channel count

- **WHEN** the buffer has a channel count other than 3 or 4
- **THEN** `apply` returns `FilterError::Unsupported` and leaves the buffer unchanged

#### Scenario: Malformed buffers error instead of panicking

- **WHEN** `apply` receives an empty buffer, or a buffer whose `data.len()` does not equal `width * height * channels`, or a buffer with zero width or height
- **THEN** `apply` returns a `FilterError` and does not panic

### Requirement: Alpha channel preservation for Distort filters

For 4-channel buffers, every Distort filter SHALL leave channel 4 bit-identical. Only channels 1 through 3 SHALL be modified, even where the warp moves color samples.

#### Scenario: Every Distort variant preserves alpha

- **WHEN** each of the five Distort variants is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Parameter validation for Distort filters

The system SHALL validate each Distort filter's parameters before writing any sample, and a rejected parameter MUST leave the buffer unchanged. `Twirl` `angle` SHALL be within `-999.0..=999.0`; `Pinch` and `Spherize` `amount` SHALL be within `-100.0..=100.0`; `Ripple` `amount` SHALL be within `-999.0..=999.0`; `Wave` `generators` SHALL be within `1..=999`, each of `wavelength.0` and `amplitude.0` SHALL be within `1.0..=998.0`, `wavelength.1` SHALL be at least `wavelength.0 + 1.0`, `amplitude.1` SHALL be at least `amplitude.0 + 1.0`, and each of `scale.0` and `scale.1` SHALL be within `1.0..=100.0`. Any value outside its range, and any non-finite `f64` parameter, SHALL be rejected with `FilterError::InvalidParams` and MUST NOT panic. The `SpherizeMode`, `RippleSize`, and `WaveType` enums are closed sets and require no range check.

#### Scenario: Out-of-range scalar amounts are rejected

- **WHEN** Twirl is applied with `angle` `-1000.0` or `1000.0`, Pinch is applied with `amount` `-101.0` or `101.0`, Spherize is applied with `amount` `-101.0` or `101.0`, or Ripple is applied with `amount` `-1000.0` or `1000.0`
- **THEN** `apply` returns `FilterError::InvalidParams` and leaves the buffer unchanged

#### Scenario: Non-finite parameters are rejected

- **WHEN** any Distort variant is applied with a `NaN` or infinite `f64` parameter
- **THEN** `apply` returns `FilterError::InvalidParams` and does not panic

#### Scenario: Out-of-range Wave parameters are rejected

- **WHEN** Wave is applied with `generators` 0 or 1000, `wavelength` min 0 or 999, `wavelength` max not greater than its min, or `scale` 0 or 101
- **THEN** `apply` returns `FilterError::InvalidParams` and leaves the buffer unchanged

#### Scenario: Boundary values are accepted

- **WHEN** Twirl `angle` `-999.0` or `999.0`, Pinch / Spherize `amount` `-100.0` or `100.0`, Ripple `amount` `-999.0` or `999.0`, and Wave `generators` `1` or `999` with the minimum `wavelength` / `amplitude` pair and `scale` exactly `1.0` are applied
- **THEN** each returns `Ok(())` with no division-by-zero or empty-region panic

### Requirement: Bilinear inverse-mapping resampling

Every Distort filter SHALL be an inverse-mapping warp: for each destination pixel it SHALL compute a source coordinate and sample the input there. A non-integral source coordinate SHALL be interpolated bilinearly from its four nearest samples; an integral coordinate SHALL return that sample exactly. Twirl, Pinch, Spherize, and Ripple SHALL sample with clamp-to-edge at the image borders; Wave SHALL use the edge policy fixed by `repeat_edge`. The interpolation MUST read no sample outside the image.

#### Scenario: A non-integral source coordinate interpolates four samples

- **WHEN** a destination pixel maps to a fractional source coordinate and the four surrounding samples are known
- **THEN** the output equals their bilinear blend within 1 LSB

#### Scenario: An identity warp is a no-op

- **WHEN** a Distort filter's displacement is the identity (Twirl angle 0, Pinch amount 0, Spherize amount 0, or Ripple amount 0)
- **THEN** the output equals the input within 1 LSB

### Requirement: Undefined-area edge handling

For a source coordinate outside the image, Twirl, Pinch, Spherize, and Ripple SHALL clamp to the nearest edge sample. Wave SHALL honor its `repeat_edge` flag: when `repeat_edge` is `true` it SHALL repeat the nearest edge sample, and when `repeat_edge` is `false` it SHALL wrap around and sample from the opposite edge. The two `Wave` modes MUST be observably different where the displacement reaches beyond the image bounds.

#### Scenario: Wave repeat-edge and wrap differ off-canvas

- **WHEN** Wave with the same seed and parameters but `repeat_edge` `true` and `false` is applied to a buffer whose displacement reaches beyond an edge
- **THEN** the two outputs differ, the repeat-edge output equals a clamp-to-edge sample, and the wrap output equals a sample from the opposite edge

#### Scenario: Radial filters clamp at the border

- **WHEN** Twirl, Pinch, Spherize, or Ripple maps a destination pixel to a source coordinate outside the image
- **THEN** the sample is taken from the nearest edge and the call does not panic

### Requirement: Tiny-image safety for Distort filters

A 1×1 image MUST NOT panic and SHALL be returned bit-identical for every Distort filter. A 1-pixel-wide or 1-pixel-tall image MUST NOT panic and SHALL return either `Ok(())` or a typed `FilterError`, with no division by zero and no out-of-bounds read in the center or radius computation.

#### Scenario: A 1x1 image is a no-op

- **WHEN** every Distort variant is applied to a 1×1 buffer
- **THEN** no variant panics, each returns `Ok(())`, and the buffer is bit-identical to the input

#### Scenario: A 1-pixel-wide or 1-pixel-tall image does not panic

- **WHEN** every Distort variant is applied to a 1×N and an N×1 buffer
- **THEN** no variant panics and each returns `Ok(())` or a `FilterError`

### Requirement: Deterministic Distort output

Twirl, Pinch, Spherize, and Ripple SHALL produce bit-identical output for equal input and parameters, with no time or thread-order dependence. Wave SHALL be seeded: applying the same `seed` and parameters to equal input buffers SHALL produce bit-identical output every run, and a different `seed` SHALL be permitted to differ.

#### Scenario: The unseeded variants are repeatable

- **WHEN** Twirl, Pinch, Spherize, or Ripple is applied twice to two clones of one buffer
- **THEN** the two output buffers are bit-identical

#### Scenario: Wave is reproducible from its seed

- **WHEN** Wave is applied twice with the same seed and parameters to two clones of one buffer
- **THEN** the two output buffers are bit-identical

### Requirement: Twirl

The system SHALL implement `Filter::Twirl { angle: f64 }`. It SHALL rotate the source coordinate about the image center by an angle that decreases with distance from the center, so the center rotates most and the edge least, and the sign of `angle` SHALL set the direction of rotation. `angle` SHALL be within `-999.0..=999.0`, and `angle` `0.0` SHALL be a no-op. Oracle expectation: no faithful ImageMagick operator (ImageMagick `-swirl` uses a different angular falloff), so Twirl is classified as no-equivalent with tolerance 0 and property tests, and the observed delta SHALL be recorded.

#### Scenario: The center rotates more than the edge

- **WHEN** Twirl is applied to a buffer with a marker offset from the center, at the center radius and at a larger radius
- **THEN** the sample near the center is displaced by a larger angle than the sample near the edge

#### Scenario: Opposite angles rotate in opposite directions

- **WHEN** Twirl is applied with `angle` `+A` and with `angle` `-A` to the same buffer
- **THEN** the two outputs are related by a reflection of the rotation direction

#### Scenario: Zero angle is a no-op

- **WHEN** Twirl is applied with `angle` `0.0`
- **THEN** the output equals the input within 1 LSB

#### Scenario: No faithful equivalent is documented

- **WHEN** the oracle mapping table is checked
- **THEN** Twirl has no ImageMagick operator, tolerance 0, a non-empty no-equivalent note, and a recorded observed delta

### Requirement: Pinch

The system SHALL implement `Filter::Pinch { amount: f64 }`. It SHALL remap the source coordinate along the normalized radius through a monotone curve so that a positive `amount` moves pixels toward the selection center and a negative `amount` moves them away, with larger magnitudes increasing the effect. `amount` SHALL be within `-100.0..=100.0`, and `amount` `0.0` SHALL be a no-op. Oracle expectation: no faithful ImageMagick operator (ImageMagick `-implode` / `-explode` use a global nonlinear model, not Photoshop's radial remap), so Pinch is classified as no-equivalent with tolerance 0 and property tests, and the observed delta SHALL be recorded.

#### Scenario: Positive amount contracts toward the center

- **WHEN** Pinch is applied with a positive `amount` to a radial test pattern
- **THEN** features at a given radius move toward the center and the center is the fixed point

#### Scenario: Negative amount expands away from the center

- **WHEN** Pinch is applied with a negative `amount` to a radial test pattern
- **THEN** features at a given radius move away from the center

#### Scenario: Zero amount is a no-op

- **WHEN** Pinch is applied with `amount` `0.0`
- **THEN** the output equals the input within 1 LSB

#### Scenario: No faithful equivalent is documented

- **WHEN** the oracle mapping table is checked
- **THEN** Pinch has no ImageMagick operator, tolerance 0, a non-empty no-equivalent note, and a recorded observed delta

### Requirement: Spherize

The system SHALL implement `Filter::Spherize { amount: f64, mode: SpherizeMode }`. It SHALL apply a radial inverse-mapping warp that wraps the selection around a sphere, and `SpherizeMode` SHALL restrict the axes: `Normal` displaces both, `HorizontalOnly` displaces only the horizontal axis, and `VerticalOnly` displaces only the vertical axis. `amount` SHALL be within `-100.0..=100.0`, and `amount` `0.0` SHALL be a no-op. Oracle expectation: no faithful ImageMagick operator, so Spherize is classified as no-equivalent with tolerance 0 and property tests, and the observed delta SHALL be recorded.

#### Scenario: Horizontal Only displaces one axis

- **WHEN** Spherize with `SpherizeMode::HorizontalOnly` is applied to a test pattern
- **THEN** samples move along the horizontal axis only, and a purely vertical displacement is not produced

#### Scenario: Vertical Only displaces the other axis

- **WHEN** Spherize with `SpherizeMode::VerticalOnly` is applied to the same test pattern
- **THEN** samples move along the vertical axis only

#### Scenario: Zero amount is a no-op

- **WHEN** Spherize is applied with `amount` `0.0` in any mode
- **THEN** the output equals the input within 1 LSB

#### Scenario: No faithful equivalent is documented

- **WHEN** the oracle mapping table is checked
- **THEN** Spherize has no ImageMagick operator, tolerance 0, a non-empty no-equivalent note, and a recorded observed delta

### Requirement: Ripple

The system SHALL implement `Filter::Ripple { amount: f64, size: RippleSize }`. It SHALL add a periodic sinusoidal displacement whose magnitude is proportional to `amount` and whose spatial frequency is fixed by `RippleSize` (`Small`, `Medium`, `Large`). `amount` SHALL be within `-999.0..=999.0`, and `amount` `0.0` SHALL be a no-op. Oracle expectation: no faithful ImageMagick operator (ImageMagick `-wave` uses a different periodic kernel), so Ripple is classified as no-equivalent with tolerance 0 and property tests, and the observed delta SHALL be recorded.

#### Scenario: Zero amount is a no-op

- **WHEN** Ripple is applied with `amount` `0.0` in any `RippleSize`
- **THEN** the output equals the input within 1 LSB

#### Scenario: Larger amount increases displacement

- **WHEN** Ripple is applied at a small `amount` and at a larger `amount` to the same buffer
- **THEN** the larger `amount` produces a larger maximum displacement

#### Scenario: Size changes spatial frequency

- **WHEN** Ripple is applied at `RippleSize::Small` and at `RippleSize::Large` with the same `amount`
- **THEN** the two outputs have different spatial frequencies

#### Scenario: No faithful equivalent is documented

- **WHEN** the oracle mapping table is checked
- **THEN** Ripple has no ImageMagick operator, tolerance 0, a non-empty no-equivalent note, and a recorded observed delta

### Requirement: Wave

The system SHALL implement `Filter::Wave { generators: u32, wavelength: (f64, f64), amplitude: (f64, f64), kind: WaveType, scale: (f64, f64), seed: u64, repeat_edge: bool }`. It SHALL sum `generators` wave generators, each with a wavelength drawn from `wavelength` and an amplitude drawn from `amplitude` using the seeded RNG, a shape selected by `WaveType` (`Sine`, `Triangle`, `Square`), and axis-wise scaling from `scale`, to form the displacement field. `generators` SHALL be within `1..=999`; each range min SHALL be within `1.0..=998.0` with its max at least `min + 1.0`; each `scale` SHALL be within `1.0..=100.0`. The same `seed` SHALL reproduce the field; `repeat_edge` SHALL select the undefined-area policy. Oracle expectation: no faithful ImageMagick operator (ImageMagick `-wave` has a different generator model), so Wave is classified as no-equivalent with tolerance 0 and property tests, and the observed delta SHALL be recorded.

#### Scenario: The same seed is reproducible

- **WHEN** Wave is applied twice with the same parameters and seed to two clones of one buffer
- **THEN** the two output buffers are bit-identical

#### Scenario: A different seed changes the field

- **WHEN** Wave is applied with two different seeds and all other parameters equal
- **THEN** the two outputs are permitted to differ

#### Scenario: The generator count changes the field

- **WHEN** Wave is applied with `generators` `1` and with a larger `generators` count, same seed
- **THEN** the two outputs differ

#### Scenario: Each wave type produces a distinct field

- **WHEN** Wave is applied with `WaveType::Sine`, `WaveType::Triangle`, and `WaveType::Square`, same seed
- **THEN** the three outputs differ

#### Scenario: No faithful equivalent is documented

- **WHEN** the oracle mapping table is checked
- **THEN** Wave has no ImageMagick operator, tolerance 0, a non-empty no-equivalent note, and a recorded observed delta

### Requirement: Distort oracle classification

The system SHALL extend `scripts/filter_oracle.py` and `crates/pictura-filters/tests/oracle.rs` so the mapping table has exactly one row per Distort `Filter` variant. Because Adobe's warp kernels and falloff curves are closed, Twirl, Pinch, Spherize, Ripple, and Wave SHALL each be classified as no-equivalent with tolerance 0, a property or known-value test, a non-empty note, and the observed delta against the closest ImageMagick operator (`-swirl`, `-implode` / `-explode`, or `-wave`) recorded. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table covers every Distort variant

- **WHEN** the oracle tests run
- **THEN** every Distort variant has a table row, each row has tolerance 0, and each row carries a non-empty no-equivalent note with a recorded observed delta

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes

