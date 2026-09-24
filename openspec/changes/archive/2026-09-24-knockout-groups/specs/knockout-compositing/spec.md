# Specs delta: knockout-groups

## MODIFIED Requirements

### Requirement: Knockout punches a layer through to the background

The CPU compositor SHALL apply the `Knockout` mode of a non-bottom layer. When
such a layer's mode is `Shallow` or `Deep`, the layer's content SHALL be
composited against the document background — the bottom layer's composite, or
transparency when the layer is the bottom — instead of the running backdrop, and
the result SHALL replace the running backdrop at pixels the layer covers. The
layers between the knockout layer and the background SHALL therefore not
contribute at those pixels. This SHALL hold both for a top-level layer and for a
layer inside a group whose blend mode is Pass Through, opacity is full, and mask
is absent. A layer whose mode is `None`, a layer at the bottom of the document,
and a layer inside any other group SHALL composite byte-identically to before.

#### Scenario: Deep punches through the intermediate layer

- **WHEN** a stack of bottom red, middle green, and a top blue layer with `Knockout::Deep` composites at a covered pixel
- **THEN** the result is blue over red, and the middle green does not contribute

#### Scenario: Deep punches through a pass-through group

- **WHEN** a pass-through group above a red background contains a green layer and a blue layer with `Knockout::Deep`
- **THEN** the result is blue over red, and the group's green does not contribute

#### Scenario: An isolated group stays inert

- **WHEN** a knockout layer is inside a group whose blend mode is not Pass Through, or whose opacity is below full, or which carries a mask
- **THEN** the composited output is byte-identical to the same stack without the knockout

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

### Requirement: The pass-through-group knockout is verified against psd-tools

The repository SHALL carry a committed knockout-in-a-pass-through-group PSD
fixture and a pixel reference derived from `psd-tools`' own compositor, and a
render test SHALL composite the fixture with the CPU compositor and compare the
result to the reference within the documented tolerance, self-skipping when
`psd_tools` is unavailable. The test SHALL assert that the fixture's group is
`PassThrough` and its child is `Deep`, so a fixture that silently becomes
isolated fails rather than passing for the wrong reason.

#### Scenario: The group fixture matches the reference

- **WHEN** the pass-through-group knockout fixture is composited by the CPU compositor
- **THEN** the result is within the documented tolerance of the psd-tools-derived reference

#### Scenario: The group's intermediate layer is punched through

- **WHEN** the result is examined at a pixel the knockout layer covers
- **THEN** the group's intermediate green does not contribute
