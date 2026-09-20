## ADDED Requirements

### Requirement: Transparent-pixel lock is per-pixel across the bridge

With the transparent-pixel lock enabled on the active layer, a paint stroke SHALL
modify the RGB of pixels whose alpha is non-zero while preserving each pixel's
existing alpha value exactly, and SHALL leave fully transparent (`A=0`) pixels
untouched. A pixel with `A=180` SHALL remain `A=180` after editing while its RGB
may change. The filter path SHALL follow the same rule. Content move keeps its
existing separate transparency refusal, recorded as a documented divergence.

#### Scenario: Alpha is preserved while RGB changes [llk_transparency_preserve]

- **WHEN** a stroke is painted over a pixel with `A=180` on a transparency-locked
  layer
- **THEN** the pixel's RGB may change and its alpha remains exactly 180

#### Scenario: Fully transparent pixels are untouched [llk_transparency_zero]

- **WHEN** a stroke is painted over a pixel with `A=0` on a transparency-locked
  layer
- **THEN** the pixel stays fully transparent
