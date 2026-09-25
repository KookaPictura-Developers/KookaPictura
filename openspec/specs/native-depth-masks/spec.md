# native-depth-masks Specification

## Purpose
TBD - created by archiving change native-depth-masks. Update Purpose after archive.
## Requirements
### Requirement: High-depth layers gate on their native raster mask

The CPU compositor SHALL gate a layer's blend by its native raster-mask sample
for a document read at 16 or 32 bits whose `Layer.source_channels` holds a `-2`
plane matching the mask rect, converted to the unit `f32` domain and combined
with the vector-mask coverage in unit space. A layer without a matching native
mask plane, and every 8-bit document, SHALL keep the existing 8-bit
`mask_alpha`/`blend_into` gating byte-for-byte.

#### Scenario: A depth-16 native mask gates the blend

- **WHEN** a depth-16 document's layer carries a native `-2` mask whose samples
  are not the 8-bit widening, and the layer is composited
- **THEN** the native composite reflects the native mask coverage, not the
  widened 8-bit mask

#### Scenario: A missing or disabled mask yields full coverage

- **WHEN** a layer has no mask, a disabled mask, or no mask data
- **THEN** the gate is `1.0`, as in the 8-bit path

#### Scenario: An out-of-rect pixel uses the mask default colour

- **WHEN** the sample point lies outside the mask rect
- **THEN** the gate is the mask's `default_color / 255`, as in the 8-bit path

### Requirement: The 8-bit composite is unchanged

The composited bytes SHALL be identical to before this change for an 8-bit or
constructed document, and for a high-depth document at the `composite_rgba`
output.

#### Scenario: The render suite is unmoved

- **WHEN** the existing render, composite, and document-oracle tests run
- **THEN** they pass with no golden change

