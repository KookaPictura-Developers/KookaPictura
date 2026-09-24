# knockout-compositing Specification

## Purpose
TBD - created by archiving change knockout-composite. Update Purpose after archive.
## Requirements
### Requirement: Knockout punches a layer through to the background

The CPU compositor SHALL apply a top-level, non-bottom layer's `Knockout` mode.
When such a layer's mode is `Shallow` or `Deep`, the layer's content SHALL be
composited against the document background — the bottom layer's composite, or
transparency when the layer is the bottom — instead of the running backdrop, and
the result SHALL replace the running backdrop at pixels the layer covers. The
layers between the knockout layer and the background SHALL therefore not
contribute at those pixels. A layer whose mode is `None`, a layer at the bottom
of the document, and a layer inside a group SHALL composite byte-identically to
before.

#### Scenario: Deep punches through the intermediate layer

- **WHEN** a stack of bottom red, middle green, and a top blue layer with `Knockout::Deep` composites at a covered pixel
- **THEN** the result is blue over red, and the middle green does not contribute

#### Scenario: Shallow equals Deep at the document root

- **WHEN** the same stack uses `Knockout::Shallow`
- **THEN** the composited output is byte-identical to the `Deep` result

#### Scenario: None is a no-op

- **WHEN** a non-bottom layer's mode is `None`
- **THEN** the composited output is byte-identical to the same stack without the field

#### Scenario: A transparent pixel does not punch through

- **WHEN** a knockout layer covers a pixel with zero source alpha
- **THEN** the running backdrop at that pixel is unchanged

### Requirement: The GPU compositor declines a knockout layer

The GPU compositor SHALL report an advanced-blending error for a stack containing
a layer, at any depth, whose knockout mode is `Shallow` or `Deep`, so the caller
composites on the CPU. A stack with no such layer SHALL continue to use the GPU
path.

#### Scenario: A knockout layer forces the CPU oracle

- **WHEN** a document contains a layer with `Knockout::Shallow` or `Knockout::Deep`
- **THEN** the GPU compositor declines with the advanced-blending error and the CPU composite is the result

#### Scenario: No knockout keeps the GPU path

- **WHEN** no layer carries a non-`None` knockout
- **THEN** the GPU compositor is attempted as before

