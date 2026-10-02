## ADDED Requirements

### Requirement: Dust & Scratches

The system SHALL implement `Filter::DustAndScratches { radius: u32, threshold: u32 }`.
For each color plane it SHALL compute the median over the `(2r+1)²` neighborhood
sampled clamp-to-edge, and SHALL replace the center sample with that median only
when the absolute difference between the center and the median is greater than
`threshold`; otherwise the sample SHALL be left unchanged. `threshold` 0 SHALL
therefore behave as Median (every dissimilar sample is replaced) and `threshold`
255 SHALL replace nothing. `radius` SHALL span `1..=16` and `threshold` SHALL
span `0..=255`; a value outside either range SHALL be rejected with
`FilterError::InvalidParams` before any mutation. A uniform-color plane SHALL be
bit-exactly unchanged. Alpha SHALL be left bit-identical. Oracle expectation: no
faithful ImageMagick equivalent exists (the local statistic and gate are closed),
so property tests cover the gate and the radius, and the divergence is
documented.

#### Scenario: A scratch at threshold 0 is replaced by the median

- **WHEN** Dust & Scratches with radius 1 and threshold 0 is applied to a smooth plane containing an isolated bright speck
- **THEN** the speck sample takes the local median while the surrounding smooth samples are unchanged

#### Scenario: Threshold 255 replaces nothing

- **WHEN** Dust & Scratches with threshold 255 is applied to a smooth plane containing an isolated bright speck
- **THEN** the speck sample is left bit-identical

#### Scenario: A uniform plane is unchanged

- **WHEN** Dust & Scratches is applied to a uniform-color plane
- **THEN** the plane is bit-exactly the same as the input

#### Scenario: Out-of-range parameters are rejected untouched

- **WHEN** Dust & Scratches is applied with radius 0 or radius 17, or with threshold 256
- **THEN** `apply` returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

#### Scenario: Alpha is preserved

- **WHEN** Dust & Scratches is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit
