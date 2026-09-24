# Specs delta: type-layer-kind

## MODIFIED Requirements

### Requirement: Layer row tooltips

The system SHALL provide a tooltip for each row that includes the layer's name
and its kind, where the kind is one of pixel, group, adjustment, background,
or type.

#### Scenario: A group row tooltip names the kind [m39_tooltip]

- **WHEN** the pointer rests on a group row
- **THEN** the tooltip contains the group's name and the word `group`

#### Scenario: A pixel row tooltip names the kind [m39_tooltip]

- **WHEN** the pointer rests on a pixel layer row
- **THEN** the tooltip contains the layer's name and the word `pixel`
