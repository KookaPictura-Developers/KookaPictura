## ADDED Requirements

### Requirement: Color Lookup adjustment

The system SHALL implement `Adjustment::ColorLookup(ColorLookupParams)` where
`ColorLookupParams` carries a `ColorLookupKind` (3-D LUT, abstract profile, or
device link) and an optional parsed three-dimensional lookup
`Lut3d { size, points }` whose points are RGB triples in the `.CUBE` order (red
index fastest, then green, then blue) and whose `size` is the number of grid
steps per axis. `apply` SHALL sample the lookup trilinearly per pixel: normalize
each channel to `0.0..=1.0`, scale by `size - 1`, take the eight surrounding
nodes, and interpolate with the fractional part, writing the sampled RGB to the
colour channels and leaving alpha untouched. When the lookup is `None` (an
abstract-profile or device-link kind, or an embedded format other than a valid
`.CUBE`), `apply` SHALL leave the buffer bit-exactly unchanged. A `Lut3d` with a
size below 2, a size above 64, a point count that is not `size³`, or any
non-finite component SHALL be rejected as `AdjustError::InvalidParams`. Oracle
expectation: no ImageMagick operator applies an arbitrary `.cube` and
Photoshop's sampling is not independently reproducible here, so there is no
Adobe pixel-parity claim; known-value tests cover the identity, exact node
mapping (fixing the red-fastest point order), a trilinear blend, and the
`None` no-op.

#### Scenario: An identity lookup is the identity

- **WHEN** Color Lookup is applied with a `size`-2 identity lookup to any buffer
- **THEN** every colour channel is within 1 LSB of its input and alpha is bit-identical

#### Scenario: A corner node maps exactly

- **WHEN** a lookup's corner node is a distinct colour and a pixel sits exactly on that node
- **THEN** the output is that node's colour within 1 LSB

#### Scenario: The red-fastest point order is honored

- **WHEN** a `size`-3 lookup colours only the node at red 1, green 0, blue 0 and a pixel maps exactly to it
- **THEN** the output is that node's colour, proving the point order is red-fastest

#### Scenario: A missing lookup is a no-op

- **WHEN** `apply` is called with `ColorLookupParams` whose lookup is `None`
- **THEN** the buffer is bit-exactly unchanged

#### Scenario: An invalid lookup is rejected

- **WHEN** a `Lut3d` has a size below 2 or above 64, a point count that is not `size³`, or a non-finite component
- **THEN** `apply` returns `AdjustError::InvalidParams` and does not panic

#### Scenario: Alpha is preserved

- **WHEN** Color Lookup is applied to an RGBA buffer
- **THEN** the alpha plane is bit-identical to the input

## MODIFIED Requirements

### Requirement: ImageMagick differential oracle and no-equivalent classification

The system SHALL ship `scripts/adjust_oracle.py`, which applies an ImageMagick operator to a raw 8-bit image, and `crates/pictura-adjust/tests/oracle.rs`, which diffs `apply` against it. The test table SHALL have exactly one row for each destructive adjustment variant plus a no-equivalent row for the generative `GradientFill` and `PatternFill`. `SolidFill` is also refused fill content with no `apply`, so it has no oracle row; its refusal is covered by the alpha/refusal tests. Only Levels, Invert, and Desaturate SHALL be diffed (`-level ... +level ...`, `-negate`, `-modulate 100,0,100`), with tolerances of 1, 0, and 1. The other seventeen adjustments SHALL be classified as having no faithful ImageMagick operator and SHALL use tolerance 0: BlackWhite, PhotoFilter, GradientMap, GradientFill, PatternFill, Vibrance, ColorBalance, Auto, Curves, Exposure, BrightnessContrast, HueSaturation, ChannelMixer, SelectiveColor, ColorLookup, Posterize, and Threshold. The differential tests SHALL skip with a message when `magick` is not on `PATH` and MUST NOT be marked `#[ignore]`.

#### Scenario: Mapping table matches the implemented contract

- **WHEN** the oracle tests run
- **THEN** the mapping table has 20 rows, every no-equivalent row has tolerance 0, and each row carries a non-empty note

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes
