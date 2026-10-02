## ADDED Requirements

### Requirement: Extrude

The system SHALL implement `Filter::Extrude { kind: ExtrudeType, size: u32, depth: f32, level_based: bool, solid_front: bool, mask_incomplete: bool }`
with `ExtrudeType::{Blocks, Pyramids}`. It SHALL divide the image into cells of
`size` pixels and render each as a 3D block (a front face plus four shaded
sides) or a pyramid (four triangular sides to an apex), protruding by `depth`;
towers SHALL be drawn back-to-front. When `level_based` is true the protrusion
SHALL follow the cell's average luma, otherwise it SHALL come from a coordinate
hash. When `solid_front` is true (Blocks) the front face SHALL be filled with
the cell average, otherwise it SHALL sample the source picture. When
`mask_incomplete` is true partial cells on the right and bottom edge SHALL be
hidden. `size` SHALL span `2..=255` and `depth` SHALL be finite and within
`1.0..=255.0`; a value outside those ranges SHALL be rejected with
`FilterError::InvalidParams` before any mutation. Alpha SHALL be left
bit-identical, and the output SHALL be deterministic (the coordinate hash
carries no seed). Oracle expectation: no faithful ImageMagick equivalent exists
(the extrusion renderer is closed), so property tests cover the level ordering
and the front faces, and the divergence is documented.

#### Scenario: Level-based depth follows source brightness

- **WHEN** Extrude with `level_based: true` and `solid_front: true` at full depth renders a dark cell next to a bright cell
- **THEN** the bright cell's front is painted over the dark neighbour

#### Scenario: Solid front faces carry the cell average

- **WHEN** Extrude with `solid_front: true` renders a whole cell
- **THEN** every pixel of that cell's front face carries the same average color

#### Scenario: Output is deterministic

- **WHEN** Extrude with the same parameters is applied to two clones of one buffer
- **THEN** the two outputs are bit-identical

#### Scenario: Out-of-range size or depth is rejected untouched

- **WHEN** Extrude is applied with `size` below 2 or above 255, or a `depth` that is non-finite, below 1.0, or above 255.0
- **THEN** `apply` returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

#### Scenario: Alpha is preserved

- **WHEN** Extrude is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Tiles

The system SHALL implement `Filter::Tiles { count: u32, offset: u32, fill: TileFill, foreground: [u8; 3], background: [u8; 3] }`
with
`TileFill::{BackgroundColor, ForegroundColor, InverseImage, UnalteredImage}`.
It SHALL break the image into `count` square tiles counted across the shorter
side, displace each tile by a coordinate-hash amount up to `offset` percent of
the tile size, and fill the exposed gaps per `fill`: the `background` color, the
`foreground` color, the inverse of the source pixels, or the source pixels
unchanged. `count` SHALL span `1..=99` and `offset` SHALL span `1..=99`
percent; a value outside those ranges SHALL be rejected with
`FilterError::InvalidParams` before any mutation. Alpha SHALL be left
bit-identical, and the output SHALL be deterministic (the coordinate hash
carries no seed). Oracle expectation: no faithful ImageMagick equivalent exists
(the tiling and fill semantics are closed), so property tests cover the fill
rules and determinism, and the divergence is documented.

#### Scenario: The background fill is flat

- **WHEN** Tiles with `fill: BackgroundColor` is applied
- **THEN** every pixel left uncovered by a displaced tile equals the `background` color

#### Scenario: Inverse Image negates the ground

- **WHEN** Tiles with `fill: InverseImage` is applied to a flat buffer
- **THEN** every uncovered pixel equals the source value subtracted from 255

#### Scenario: Unaltered Image leaves a flat picture alone

- **WHEN** Tiles with `fill: UnalteredImage` is applied to a flat buffer
- **THEN** the buffer is bit-identical to the input

#### Scenario: Output is deterministic

- **WHEN** Tiles with the same parameters is applied to two clones of one buffer
- **THEN** the two outputs are bit-identical

#### Scenario: Out-of-range parameters are rejected untouched

- **WHEN** Tiles is applied with `count` 0 or 100, or `offset` 0 or 100
- **THEN** `apply` returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

#### Scenario: Alpha is preserved

- **WHEN** Tiles is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Trace Contour

The system SHALL implement `Filter::TraceContour { level: u8, edge: ContourEdge }`
with `ContourEdge::{Lower, Upper}`. For each color channel it SHALL mark a
sample as a crossing when `edge` is `Upper`, the sample is at or above `level`,
and at least one of its four-connected neighbours is below `level`, or when
`edge` is `Lower`, the sample is below `level`, and at least one neighbour is at
or above `level`. A marked sample SHALL become 0 in that channel and every other
sample SHALL become 255 in that channel. `level` is a `u8`, so no out-of-range
value can be constructed. A flat field SHALL map to 255 everywhere, alpha SHALL
be left bit-identical, and the output SHALL be deterministic. Oracle
expectation: no faithful ImageMagick equivalent exists (the contour test is
closed), so property tests cover the flat no-op and the two edge directions, and
the divergence is documented.

#### Scenario: A flat image has no contour

- **WHEN** Trace Contour is applied to a constant-color buffer
- **THEN** every color sample equals 255 in every channel

#### Scenario: Lower and Upper ink opposite sides of a step

- **WHEN** Trace Contour with `edge: Upper` and with `edge: Lower` is applied to the same dark-to-bright step
- **THEN** `Upper` marks the bright side and `Lower` marks the dark side

#### Scenario: Shifting the level moves the contour

- **WHEN** Trace Contour is applied at level 0 and at level 255 to the same ramp
- **THEN** level 0 crosses nothing and level 255 marks the peak

#### Scenario: Output is deterministic and alpha is preserved

- **WHEN** Trace Contour is applied twice to equal RGBA buffers
- **THEN** the two outputs are bit-identical and the alpha plane equals the input alpha plane bit for bit

### Requirement: Wind

The system SHALL implement `Filter::Wind { method: WindMethod, from_right: bool }`
with `WindMethod::{Wind, Blast, Stagger}`. It SHALL draw horizontal streaks
from the upwind side toward the downwind side, only where the upwind pixel's
luma exceeds the current pixel's by a fixed gate. `method` SHALL select the
streak length, the fraction of edges that receive a streak, and the strength
retained at the far end (`Wind` 12/55/0.0, `Blast` 32/85/0.55, `Stagger`
16/70/0.25), and `Stagger` SHALL additionally drift each streak vertically.
`from_right` true SHALL blow toward the left and false toward the right. The
filter SHALL be deterministic (the candidate hash carries no seed), SHALL leave
alpha bit-identical, and MUST NOT wrap or overflow at the final clamp. Oracle
expectation: no faithful ImageMagick equivalent exists (the streak geometry is
closed), so property tests cover the flat-ground no-op, the direction mirror,
and determinism, and the divergence is documented.

#### Scenario: Flat ground is untouched

- **WHEN** Wind is applied with any method to a flat buffer
- **THEN** the buffer is bit-identical to the input

#### Scenario: Direction mirrors the displacement

- **WHEN** Wind with `from_right: true` and with `from_right: false` is applied to the same buffer
- **THEN** the two outputs differ

#### Scenario: Output is deterministic and alpha is preserved

- **WHEN** Wind is applied twice to equal RGBA buffers
- **THEN** the two outputs are bit-identical and the alpha plane equals the input alpha plane bit for bit
