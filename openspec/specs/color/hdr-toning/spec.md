# hdr-toning Specification

## Purpose
The Exposure and Gamma tone-map operator for high-dynamic-range images.

## Requirements

### Requirement: Exposure and Gamma tone-map operator

The system SHALL provide
`pictura_adjust::hdr_toning::exposure_gamma(samples: &[f32], params:
ExposureGamma) -> Result<Vec<f32>, AdjustError>`, where `ExposureGamma` carries
`exposure_ev: f64` and `gamma: f64`. It SHALL apply, elementwise in linear light,
`gain = 2^exposure_ev` and output `sign(x·gain)·|x·gain|^(1/gamma)`, so a value
above `1.0` is not clamped, `0` maps to `0`, and a negative input keeps its sign.
`exposure_ev = 0` with `gamma = 1.0` SHALL be the identity. The operator SHALL
return `AdjustError::InvalidParams` when `exposure_ev` is not finite or `gamma`
is not finite or not strictly positive, and MUST NOT panic on any input.

#### Scenario: Identity at zero exposure and unit gamma

- **WHEN** `exposure_gamma` is called with `exposure_ev = 0.0` and `gamma = 1.0`
- **THEN** the output equals the input within floating-point tolerance

#### Scenario: The documented formula applies above 1.0

- **WHEN** a sample `2.0` is toned with `exposure_ev = 1.0` and `gamma = 2.0`
- **THEN** the output is `(2.0·2)^(1/2) = 2.0` and no pre-clamp to `1.0` occurs

#### Scenario: Zero and negative samples are defined

- **WHEN** the input contains `0.0` and a negative value
- **THEN** `0.0` maps to `0.0` and the negative value stays negative with no NaN

#### Scenario: Bad parameters are rejected

- **WHEN** `gamma` is `0`, negative, or NaN, or `exposure_ev` is NaN or infinite
- **THEN** `exposure_gamma` returns `AdjustError::InvalidParams` and does not panic

### Requirement: Local Adaptation HDR Toning operator

The system SHALL provide a deterministic Local Adaptation tone-mapping operator over a planar 8-bit RGB(A) buffer, parameterised by edge-glow radius (`1..=500`), strength (`0.01..=4.0`), gamma (`0.01..=9.99`), exposure (`-5.0..=5.0`), detail (`-100..=300`), and shadow, highlight, vibrance, and saturation (each `-100..=100`). It SHALL blur a copy of the colour planes to a local base, split log-luminance into a magnitude-gated detail term, compress the base about the mean log-luminance pivot using the gamma, apply exposure, recover shadows and highlights, roll off the shoulder and toe, add the luminance delta to all colour channels, and gate the vibrance/saturation adjustment by the original saturation. Alpha SHALL be preserved, and an out-of-range parameter SHALL be rejected with a filter error and no mutation.

#### Scenario: A non-uniform image is tone-mapped and alpha preserved

- **WHEN** the operator runs on a non-uniform RGBA buffer with default parameters
- **THEN** the colour planes change and the alpha plane is bit-identical

#### Scenario: Out-of-range parameters are rejected

- **WHEN** any parameter lies outside its documented range
- **THEN** the operator returns an error and leaves the buffer unchanged

### Requirement: HDR Toning dialog with CS6 presets

`Image ▸ Adjustments ▸ HDR Toning` SHALL open a dialog exposing the nine Local Adaptation controls and a preset list containing CS6's presets (`Default`, `City Twilight`, `Flat`, `Monochromatic Artistic`, `Monochromatic High Contrast`, `Monochromatic Low Contrast`, `Monochromatic`, `More Saturated`, `Photorealistic High Contrast`, `Photorealistic Low Contrast`, `Photorealistic`, `RCS`, `Saturated`, `ScottS`, `Surrealistic High Contrast`, `Surrealistic Low Contrast`, `Surrealistic`) plus `Custom`. Selecting a preset SHALL load all nine controls with the preset's values. The dialog SHALL preview on the canvas without recording history, OK SHALL commit one history state named `HDR Toning`, and Cancel SHALL restore the pre-dialog pixels bit-identically.

#### Scenario: Selecting a preset populates the controls

- **WHEN** a listed preset is chosen in the dialog
- **THEN** the nine controls show that preset's values

#### Scenario: Apply commits one state and cancel restores

- **WHEN** the dialog is accepted after previewing, or cancelled after previewing
- **THEN** OK records exactly one `HDR Toning` history state, and Cancel restores the pre-dialog pixels bit-identically
