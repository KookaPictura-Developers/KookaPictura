# Spec Delta

## ADDED Requirements

### Requirement: Lighting Effects filter

The system SHALL implement `Filter::Lighting` with a light type (Spot, Point, Infinite), a colour, intensity and hotspot, colourize, ambience, exposure, gloss, metallic, a bump texture channel (None/Red/Green/Blue), height, a unit centre, size, and angle, using a CS6-style ambient plus diffuse plus Blinn specular model over the layer colour and the selected bump channel's gradient. It SHALL default to a white Spot light at the unit centre with intensity 25 and hotspot 44. Alpha SHALL be preserved, output SHALL be deterministic, and an out-of-range parameter SHALL be rejected as `FilterError::InvalidParams` without mutating the buffer.

#### Scenario: Lighting relights the colour planes and preserves alpha

- **WHEN** Lighting Effects is applied to a non-uniform RGBA buffer
- **THEN** the colour planes change, the result is deterministic across runs, and the alpha plane is bit-identical

#### Scenario: Each light type differs

- **WHEN** the same scene is lit with Spot, Point, and Infinite lights
- **THEN** the three outputs differ

#### Scenario: Bad parameters are refused untouched

- **WHEN** a parameter is out of range
- **THEN** the operator returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

### Requirement: Post-CS6 render entries are not surfaced

The `Filter ▸ Render` menu SHALL list only the CS6 entries (Clouds, Difference Clouds, Fibers, Lens Flare, Lighting Effects). `Flame`, `Tree`, and `Picture Frame` were added in Photoshop CC 2014.2 and are not CS6; they MUST NOT be surfaced in the menu and SHALL remain unsurfaced unless the project's parity scope changes.

#### Scenario: Flame, Tree, and Picture Frame are absent

- **WHEN** the `Filter ▸ Render` menu is built for CS6 parity
- **THEN** it does not contain `Flame`, `Tree`, or `Picture Frame`
