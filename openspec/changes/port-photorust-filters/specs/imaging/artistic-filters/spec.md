## MODIFIED Requirements

### Requirement: Film Grain

The system SHALL implement `Filter::FilmGrain { grain, highlight_area, intensity, seed }` as a tonal-zone noise field, stronger in shadows and midtones, with a smoother brightening laid over the tones above the `highlight_area` line, reproducible from its seed.

#### Scenario: Grain vanishes at zero

- **WHEN** Film Grain is applied with `grain` 0 and `highlight_area` 0
- **THEN** the output equals the input

#### Scenario: Grain is deterministic

- **WHEN** Film Grain is applied twice with the same seed and parameters
- **THEN** the outputs match bit-for-bit

### Requirement: Paint Daubs

The system SHALL implement `Filter::PaintDaubs { brush_size, sharpness, brush_type, seed }` with `brush_type` in Simple, Light Rough, Dark Rough, Wide Sharp, Wide Blurry, Sparkle, and daubs that widen with `brush_size`. The `seed` SHALL re-roll the brush types that carry randomness (Sparkle). Sparkle SHALL draw thin lines of light along the boundaries between bands of brightness, with a glow round them, and SHALL NOT draw dark lines.

#### Scenario: Brush types differ

- **WHEN** Paint Daubs is applied with two different brush types
- **THEN** the outputs differ

#### Scenario: Size scales the strokes

- **WHEN** Paint Daubs is applied to a ramp at `brush_size` 1 and 50
- **THEN** the 50 output keeps fewer distinct tones than the 1 output

#### Scenario: Sparkle draws lines of light

- **WHEN** Paint Daubs is applied to a slow ramp with the Simple and the Sparkle brush
- **THEN** the Sparkle output has pixels well above the Simple output and none noticeably below it

### Requirement: Palette Knife

The system SHALL implement `Filter::PaletteKnife { stroke_size, stroke_detail, softness }` as Kuwahara smoothing with a reach of a third of `stroke_size`, then half of each channel rounded onto a palette of `2 + 3 * stroke_detail` levels, then a Gaussian blur of `softness / 2` pixels. The filter is deterministic and carries no seed.

#### Scenario: Non-empty effect

- **WHEN** Palette Knife is applied to a textured test image
- **THEN** the output differs from the input

#### Scenario: Flattens a surface and stops at a boundary

- **WHEN** Palette Knife is applied to two noisy fields that meet at a hard edge
- **THEN** each field comes back flatter and the edge stays hard

#### Scenario: Stroke Detail is the palette

- **WHEN** Palette Knife is applied to a flat 150 grey at `stroke_detail` 1 and 3
- **THEN** the outputs are 139 and 152
