# hdr-toning Specification

## Purpose
TBD - created by archiving change hdr-exposure-gamma. Update Purpose after archive.
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

