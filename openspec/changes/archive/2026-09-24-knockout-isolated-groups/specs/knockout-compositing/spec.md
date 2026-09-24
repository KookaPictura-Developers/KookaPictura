# Specs delta: knockout-isolated-groups

## MODIFIED Requirements

### Requirement: Knockout punches a layer through to the background

The CPU compositor SHALL apply the `Knockout` mode of a non-bottom layer. When
such a layer's mode is `Shallow` or `Deep`, the layer's content SHALL be
composited against a knockout base instead of the running backdrop, and the
result SHALL replace the running backdrop at pixels the layer covers, so the
layers between the knockout layer and the base do not contribute there. The base
SHALL be the document background — the bottom layer's composite, or transparency
when the layer is the bottom — for a top-level layer and for a layer inside a
group whose blend mode is Pass Through with full opacity and no mask; the base
SHALL be the enclosing isolated group's initial backdrop for a layer inside any
other group. A layer whose mode is `None`, a layer at the bottom of the
document, and a layer in a group with no knockout SHALL composite
byte-identically to before.

#### Scenario: Deep punches through the intermediate layer

- **WHEN** a stack of bottom red, middle green, and a top blue layer with `Knockout::Deep` composites at a covered pixel
- **THEN** the result is blue over red, and the middle green does not contribute

#### Scenario: Deep punches through a pass-through group to the background

- **WHEN** a pass-through group above a red background contains a green layer and a blue layer with `Knockout::Deep`
- **THEN** the result is blue over red, and the group's green does not contribute

#### Scenario: Deep stops at an isolated group

- **WHEN** an isolated group above a yellow layer contains a green layer and a half-fill blue layer with `Knockout::Deep`
- **THEN** the group's green is punched through to the group's initial backdrop and the yellow below the group still contributes

#### Scenario: Shallow equals Deep at the document root

- **WHEN** the same stack uses `Knockout::Shallow`
- **THEN** the composited output is byte-identical to the `Deep` result

#### Scenario: None is a no-op

- **WHEN** a non-bottom layer's mode is `None`
- **THEN** the composited output is byte-identical to the same stack without the field

#### Scenario: A transparent pixel does not punch through

- **WHEN** a knockout layer covers a pixel with zero source alpha
- **THEN** the running backdrop at that pixel is unchanged

## ADDED Requirements

### Requirement: The isolated-group knockout is verified against psd-tools

The repository SHALL carry a committed knockout-in-an-isolated-group PSD fixture
whose document also holds a layer below the group, and a pixel reference derived
from `psd-tools`' own compositor. A render test SHALL composite the fixture with
the CPU compositor and compare the result to the reference within the documented
tolerance, self-skipping when `psd_tools` is unavailable, and SHALL assert that
the fixture's group is not Pass Through and its knockout child is `Deep`.

#### Scenario: The isolated-group fixture matches the reference

- **WHEN** the isolated-group knockout fixture is composited by the CPU compositor
- **THEN** the result is within the documented tolerance of the psd-tools-derived reference

#### Scenario: The layer below the group still contributes

- **WHEN** the result is examined at a covered pixel
- **THEN** the group's green is punched through, and the layer below the group is blended (not the document background)
