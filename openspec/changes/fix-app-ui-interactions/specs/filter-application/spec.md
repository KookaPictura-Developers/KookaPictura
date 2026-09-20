## ADDED Requirements

### Requirement: Filter application targets the active layer

The application-level filter entry point SHALL apply a filter to the active
layer, where the active layer is the exactly-one layer resolved by the shared
active-layer resolver, rather than defaulting to the topmost raster layer. When
no layer is active or more than one layer is selected, the application SHALL
refuse the filter and leave the document unchanged with no history state. The
refusal SHALL be surfaced to the user.

#### Scenario: A filter applies to the active layer [lfa_active_layer]

- **WHEN** a filter is applied with exactly one layer active
- **THEN** that layer is filtered and the other layers are unchanged

#### Scenario: No single active layer refuses [lfa_no_active_layer]

- **WHEN** a filter is applied with no layer active or with more than one layer
  selected
- **THEN** the filter is refused, no layer is mutated, and the document is
  unchanged

### Requirement: Filter respects the transparency lock per pixel

When the active layer's lock state includes `TRANSPARENCY`, a filter SHALL change
only the color channels of pixels whose pre-existing alpha is greater than zero
and SHALL write each such pixel's pre-existing alpha back unchanged; a pixel whose
alpha is zero SHALL be left untouched. The filter SHALL NOT lower any pixel's
alpha, and applying it SHALL leave every pixel's alpha value equal to its
pre-filter value.

#### Scenario: Filtering a transparency-locked layer keeps alpha [lfa_transparency_alpha]

- **WHEN** a filter runs on a transparency-locked layer
- **THEN** the color channels change where alpha is greater than zero and every
  pixel's alpha equals its pre-filter value

#### Scenario: Fully transparent pixels are untouched [lfa_transparency_clear_pixel]

- **WHEN** a filter runs on a transparency-locked layer at a pixel whose alpha is
  zero
- **THEN** that pixel's color and alpha are unchanged
