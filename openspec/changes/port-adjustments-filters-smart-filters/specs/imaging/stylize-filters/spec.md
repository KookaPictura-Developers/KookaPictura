# Spec Delta

## ADDED Requirements

### Requirement: Diffuse filter

The system SHALL implement `Filter::Diffuse { mode }` where `mode` is `Normal`, `DarkenOnly`, `LightenOnly`, or `Anisotropic`. For a coordinate-hashed choice it SHALL replace each pixel with one of its eight neighbours (Normal), or with the neighbour only when it is darker (Darken Only) or lighter (Lighten Only); Anisotropic SHALL blend each pixel along its edge over four passes. Alpha SHALL be preserved, output SHALL be deterministic (a fixed coordinate hash, with no per-apply re-roll), and `apply` SHALL report a `FilterError` rather than panic on a malformed buffer.

#### Scenario: The modes change colour and preserve alpha

- **WHEN** each Diffuse mode is applied to a non-uniform RGBA buffer
- **THEN** the colour planes change and the alpha plane is bit-identical

#### Scenario: Diffuse is deterministic

- **WHEN** Diffuse is applied twice to clones of one buffer
- **THEN** the two outputs are bit-identical

### Requirement: Glowing Edges filter

The system SHALL implement `Filter::GlowingEdges { width, brightness, smoothness }` with `width` spanning `1..=14`, `brightness` `0..=20`, and `smoothness` `1..=15`. It SHALL blur a copy of the colour planes by the smoothness, take a Sobel gradient magnitude per channel, dilate it by half the width, and scale it into the colour planes by `brightness / 5`, clamping to `0..=255`. Alpha SHALL be preserved, output SHALL be deterministic, and an out-of-range parameter SHALL be rejected as `FilterError::InvalidParams` without mutating the buffer.

#### Scenario: Glowing Edges lights the edges and preserves alpha

- **WHEN** Glowing Edges is applied to a non-uniform RGBA buffer
- **THEN** the colour planes change on an otherwise black ground and the alpha plane is bit-identical

#### Scenario: Out-of-range parameters are rejected

- **WHEN** width, brightness, or smoothness is outside its documented range
- **THEN** the operator returns `FilterError::InvalidParams` and does not mutate the buffer
