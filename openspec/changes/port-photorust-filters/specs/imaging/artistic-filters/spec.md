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

The system SHALL implement `Filter::PaintDaubs { brush_size, sharpness, brush_type, seed }` with `brush_type` in Simple, Light Rough, Dark Rough, Wide Sharp, Wide Blurry, Sparkle, and daubs that widen with `brush_size`. The `seed` SHALL re-roll the brush types that carry randomness (Sparkle).

#### Scenario: Brush types differ

- **WHEN** Paint Daubs is applied with two different brush types
- **THEN** the outputs differ

#### Scenario: Size scales the strokes

- **WHEN** Paint Daubs is applied to a ramp at `brush_size` 1 and 50
- **THEN** the 50 output keeps fewer distinct tones than the 1 output
