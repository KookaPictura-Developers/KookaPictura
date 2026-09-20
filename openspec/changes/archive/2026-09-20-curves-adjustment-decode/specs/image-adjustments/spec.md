## MODIFIED Requirements

### Requirement: Curves adjustment

The system SHALL implement `Adjustment::Curves(CurvesParams { points, red,
green, blue })` where `points` is the composite curve and `red`, `green`, and
`blue` are optional per-channel curves. The composite `points` curve SHALL have
2 through 14 control points in `(input, output)` order, where inputs strictly
increase; each present per-channel curve SHALL satisfy the same 2-through-14
strictly-increasing contract. Each curve SHALL be a monotone cubic Hermite
interpolation (Fritsch-Carlson tangents) evaluated into a 256-entry LUT. The
kernel SHALL apply each present per-channel curve to its own color plane and
then apply the composite `points` LUT to all three color planes. Two endpoints
`(0, 0)` and `(255, 255)` SHALL be the exact identity, so absent per-channel
curves and an identity composite curve leave the buffer bit-exactly unchanged.
Oracle expectation: no faithful ImageMagick operator exists, so the behavior is
covered by known-value and property tests (identity, control points,
monotonicity, per-channel isolation) rather than a differential test.

#### Scenario: Two-point curve is the identity

- **WHEN** Curves is applied with `points` `[(0, 0), (255, 255)]` and no per-channel curves
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: A control point is honored

- **WHEN** Curves is applied with `points` `[(0, 0), (100, 200), (255, 255)]` to a sample at 100
- **THEN** the output sample is 200

#### Scenario: Output is monotone

- **WHEN** Curves is applied to a 0 to 255 ramp with any valid monotone point set
- **THEN** output samples never decrease as the input increases

#### Scenario: A per-channel curve touches only its channel

- **WHEN** Curves is applied with a `red` curve and an identity composite `points` curve to a buffer whose R, G, and B planes differ
- **THEN** the green and blue planes are bit-exactly unchanged and only the red plane follows the `red` curve

#### Scenario: The composite curve applies after the per-channel curves

- **WHEN** Curves is applied with both a per-channel curve and a non-identity composite `points` curve
- **THEN** each color plane is first mapped through its per-channel curve (when present) and then through the composite curve
