# Specs delta: knockout-oracle

## ADDED Requirements

### Requirement: Knockout is verified against a psd-tools reference

The repository SHALL carry a committed knockout PSD fixture and a pixel
reference derived from `psd-tools`' own compositor, and a render test SHALL
composite the fixture with the CPU compositor and compare the result to the
reference within a documented tolerance. The reference-comparison test SHALL
self-skip when `psd_tools` is unavailable, and SHALL independently assert that
the knockout layer punches through the intermediate layer.

#### Scenario: The fixture decodes as a knockout

- **WHEN** the committed knockout fixture is read
- **THEN** the knockout layer's typed mode is `Deep`

#### Scenario: The CPU composite matches the reference

- **WHEN** the fixture is composited by the CPU compositor
- **THEN** the result is within the documented tolerance of the psd-tools-derived reference at every pixel

#### Scenario: The intermediate layer is punched through

- **WHEN** the result is examined at a pixel the knockout layer covers
- **THEN** the intermediate layer's green does not contribute

#### Scenario: Missing oracle self-skips

- **WHEN** `psd_tools` is not importable
- **THEN** the reference-comparison test skips rather than fails
