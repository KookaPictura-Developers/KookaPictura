# texture-filters Specification

## Purpose
Craquelure, Grain, Mosaic Tiles, Patchwork, Stained Glass, and Texturizer with foreground, background, and texture inputs.
## Requirements
### Requirement: Texture filter application and error contract

The system SHALL implement the 6 CS6 Texture filters as `Filter` variants and
apply them through the shared `pictura_filters::apply(filter, buf)` entry point.
`apply` SHALL transform every color sample of a planar 8-bit buffer whose channel
count is 3 (RGB) or 4 (RGBA) and return `Ok(())`, or return a `FilterError`
without partially applying; malformed buffers MUST error instead of panicking.

#### Scenario: Apply a Texture filter to a color buffer

- **WHEN** `apply` receives a Texture variant and a 3- or 4-channel planar buffer
- **THEN** the color planes are rewritten in place and `Ok(())` is returned

#### Scenario: Malformed buffers error

- **WHEN** the buffer is empty, has zero width or height, or an inconsistent length
- **THEN** `apply` returns a `FilterError` and does not panic

### Requirement: Alpha channel preservation

For 4-channel buffers, every Texture filter SHALL leave channel 4 bit-identical.

#### Scenario: Every Texture variant preserves alpha

- **WHEN** each of the 6 Texture variants is applied to an RGBA buffer
- **THEN** the alpha plane equals the input alpha plane bit for bit

### Requirement: Parameter validation

The system SHALL validate each Texture filter's parameters before writing any
sample and SHALL reject out-of-range or non-finite values with
`FilterError::InvalidParams` without panicking. The accepted ranges SHALL be:
Craquelure `crack_spacing 2..=100`, `crack_depth 0..=10`,
`crack_brightness 0..=10`; Grain `intensity 0..=100`, `contrast 0..=100`;
Mosaic Tiles `tile_size 2..=100`, `grout_width 1..=15`, `lighten_grout 0..=10`;
Patchwork `square_size 0..=10`, `relief 0..=25`; Stained Glass
`cell_size 2..=50`, `border_thickness 1..=20`, `light_intensity 0..=10`.
Texture options SHALL require `scaling 50..=200`.

#### Scenario: Out-of-range parameters are rejected

- **WHEN** a Texture filter is applied with a parameter outside its range
- **THEN** `apply` returns `FilterError::InvalidParams` and leaves the buffer unchanged

#### Scenario: Boundary values are accepted

- **WHEN** each Texture filter is applied at the minimum and maximum of its ranges
- **THEN** each returns `Ok(())` without a panic

### Requirement: Determinism and seeding

Every randomised Texture filter SHALL take a `seed: u64` and SHALL be
deterministic: the same input, parameters, and seed SHALL produce bit-identical
output.

#### Scenario: Repeat applies match exactly

- **WHEN** a stochastic Texture filter is applied twice with the same seed
- **THEN** the two outputs are bit-identical

#### Scenario: Different seeds differ

- **WHEN** a stochastic Texture filter is applied with two different seeds
- **THEN** at least one output sample differs

### Requirement: Background and foreground colour inputs

The colour-dependent Texture filters SHALL take their colours as explicit
parameters: Grain SHALL accept a `background` RGB used by the Sprinkles and
Stippled grain types, and Stained Glass SHALL accept a `foreground` RGB used for
its cell borders. Equal colours SHALL NOT panic or divide by zero.

#### Scenario: Background colour shows in Sprinkles and Stippled grain

- **WHEN** Grain is applied as Sprinkles or Stippled with two different background colours
- **THEN** the resulting grain colour differs between the two runs

#### Scenario: Border is drawn in the foreground colour

- **WHEN** Stained Glass is applied with two different foreground colours
- **THEN** the cell-border colour differs between the two runs

### Requirement: Texture options

Texturizer SHALL accept shared `TextureOptions`: a surface preset (Brick, Burlap,
Canvas, Sandstone), `scaling` in `50..=200`, `relief`, one of eight
`light_direction` values, and `invert`. The surface SHALL be generated
procedurally and lit by the chosen direction; the kernel MUST NOT materialize a
canvas-sized texture.

#### Scenario: Texture options change the result

- **WHEN** Texturizer is applied with two different surface presets or light directions
- **THEN** the two outputs differ

### Requirement: Craquelure

The system SHALL implement `Filter::Craquelure { crack_spacing, crack_depth, crack_brightness }` as a fine crack network over a high-relief surface, where `crack_spacing` sets crack frequency, `crack_depth` the apparent depth, and `crack_brightness` the crack luminance.

#### Scenario: Crack spacing changes the network

- **WHEN** Craquelure is applied at `crack_spacing` 2 and at 100
- **THEN** the two outputs differ

#### Scenario: Crack brightness changes the crack luminance

- **WHEN** Craquelure is applied at `crack_brightness` 0 and at 10
- **THEN** the two outputs differ

### Requirement: Grain

The system SHALL implement `Filter::Grain { intensity, contrast, grain_type, background, seed }` with `grain_type` in `GrainType { Regular, Soft, Sprinkles, Clumped, Contrasty, Enlarged, Stippled, Horizontal, Vertical, Speckle }`, where `intensity` and `contrast` scale the noise field.

#### Scenario: Grain types differ

- **WHEN** Grain is applied with two different `grain_type` values
- **THEN** the two outputs differ

#### Scenario: Grain is deterministic

- **WHEN** Grain is applied twice with the same parameters and seed
- **THEN** the outputs are bit-identical

### Requirement: Mosaic Tiles

The system SHALL implement `Filter::MosaicTiles { tile_size, grout_width, lighten_grout, seed }` as a chip tessellation with grout bands between cells, where `tile_size` sets the chip grid, `grout_width` the band width, and `lighten_grout` lifts the grout luminance.

#### Scenario: Tile size changes the grid

- **WHEN** Mosaic Tiles is applied at `tile_size` 2 and at 100
- **THEN** the two outputs differ

#### Scenario: Lighten grout lifts the grout

- **WHEN** Mosaic Tiles is applied at `lighten_grout` 0 and at 10
- **THEN** the 10 output has greater grout luminance than the 0 output

### Requirement: Patchwork

The system SHALL implement `Filter::Patchwork { square_size, relief, seed }` as a square-block mosaic filled with each block's predominant colour and raised or lowered by `relief` to create highlights and shadows.

#### Scenario: Relief changes the tile depth

- **WHEN** Patchwork is applied at `relief` 0 and at 25
- **THEN** the two outputs differ

#### Scenario: Non-empty effect

- **WHEN** Patchwork is applied to a textured test image
- **THEN** the output differs from the input

### Requirement: Stained Glass

The system SHALL implement `Filter::StainedGlass { cell_size, border_thickness, light_intensity, foreground, seed }` as single-coloured adjacent cells outlined in the `foreground` colour, where `cell_size` sets the segmentation scale and `light_intensity` modulates cell luminance.

#### Scenario: Larger cells render fewer, larger cells

- **WHEN** Stained Glass is applied at `cell_size` 2 and at 50
- **THEN** the 50 output has larger same-colour cells than the 2 output

#### Scenario: Non-empty effect

- **WHEN** Stained Glass is applied to a textured test image
- **THEN** the output differs from the input

### Requirement: Texturizer

The system SHALL implement `Filter::Texturizer { texture }` as a lit, scaled, relief-adjusted surface applied under the image using the shared `TextureOptions`.

#### Scenario: Texture options change the result

- **WHEN** Texturizer is applied with two different `TextureOptions`
- **THEN** the two outputs differ

