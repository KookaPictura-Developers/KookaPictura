# Specs delta: blend-if-render

## ADDED Requirements

### Requirement: Blend If ranges gate the CPU compositor

The CPU compositor SHALL apply a layer's typed `BlendIf` view as a per-pixel
gate on that layer's blend weight. The composite source range SHALL gate on the
source pixel's gray and the composite destination range on the running
backdrop's gray; each per-channel group SHALL gate on that channel's source and
backdrop values. A range at the full default (`0, 65535`) SHALL be inactive. A
range SHALL contribute factor `0` when the gated value is at or below its black
endpoint or at or above its white endpoint, and `1` otherwise; the layer's
weight SHALL be multiplied by the product of the active ranges. A layer whose
typed view is absent or at the full default SHALL composite byte-identically to
before.

#### Scenario: Default ranges are a no-op

- **WHEN** a layer's blending ranges are the full default or the typed view is absent
- **THEN** the composited output is byte-identical to the same stack without the ranges

#### Scenario: A source range hides the layer below its black point

- **WHEN** a layer's composite source range has a black endpoint above a source pixel's gray
- **THEN** that layer does not contribute to that pixel

#### Scenario: A destination range gates on the backdrop

- **WHEN** a layer's composite destination range excludes the backdrop's gray at a pixel
- **THEN** that layer does not contribute to that pixel

#### Scenario: A per-channel range gates that channel

- **WHEN** a layer carries a per-channel range whose black endpoint exceeds that channel's source value
- **THEN** that layer does not contribute to that pixel

### Requirement: The GPU compositor declines a non-default Blend If layer

The GPU compositor SHALL report an advanced-blending error for a stack containing
a layer, at any depth, whose typed `BlendIf` view is present and not at the full
default, so the caller composites on the CPU. A stack with no such layer SHALL
continue to use the GPU path.

#### Scenario: A Blend If layer forces the CPU oracle

- **WHEN** a document contains a layer with a non-default Blend If range
- **THEN** the GPU compositor declines and the CPU composite is the result

#### Scenario: No Blend If keeps the GPU path

- **WHEN** no layer carries a non-default Blend If range
- **THEN** the GPU compositor is attempted as before
