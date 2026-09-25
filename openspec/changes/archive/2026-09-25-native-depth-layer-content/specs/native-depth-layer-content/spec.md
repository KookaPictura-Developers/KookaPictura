# native-depth-layer-content Specification

## ADDED Requirements

### Requirement: High-depth RGB and Grayscale pixel layers compose from native samples

The CPU compositor SHALL read a pixel layer's color and alpha from its typed
native store for a document read at 16 or 32 bits in RGB or Grayscale mode whose
`Layer.source_channels` matches the layer rect and the document source depth,
converting each native sample to the unit `f32` domain. A layer without such a
store SHALL fall back to the 8-bit channel path. A document whose `source_mode`
records a converted color mode (for example Lab or CMYK) SHALL use the 8-bit
path, because its native store holds source-mode planes rather than working RGB.

#### Scenario: A depth-16 RGB layer keeps its native color

- **WHEN** a depth-16 RGB document with a pixel layer whose retained native
  samples are not the 8-bit widening is composited
- **THEN** the native composite carries samples that are not the widening of the
  corresponding `composite_rgba` bytes

#### Scenario: A converted-mode document uses the 8-bit path

- **WHEN** a depth-16 CMYK document (`source_mode` is CMYK) is composited
- **THEN** its pixel layers are read from the 8-bit channels as before and the
  native composite is the widening of the composited color

#### Scenario: A layer without a native store falls back

- **WHEN** a high-depth document carries a layer whose rect does not match its
  retained store (or which has none)
- **THEN** the layer is read from its 8-bit channels

### Requirement: The 8-bit composite is unchanged

The composited bytes SHALL be identical to before this change for an 8-bit or
constructed document, and for a high-depth document at the `composite_rgba`
output.

#### Scenario: The render suite is unmoved

- **WHEN** the existing render, composite, and document-oracle tests run
- **THEN** they pass with no golden change
