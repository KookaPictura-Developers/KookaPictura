## ADDED Requirements

### Requirement: Layers panel action strip icons

The system SHALL give each Layers-panel action-strip button an icon from the
frozen layers asset set: `layers.link` (Link Layers), `layers.fx` (Layer Style),
`layers.mask` (Add Layer Mask), `layers.fillAdjustment` (New Fill / Adjustment
Layer), `layers.group` (New Group), `layers.newLayer` (New Layer), and
`layers.delete` (Delete). The buttons SHALL keep their text labels, and adding
an icon SHALL NOT change what each button does. Buttons whose operation is not
yet implemented (link, fx, mask) SHALL be shown disabled until their operation
lands.

#### Scenario: The implemented strip buttons carry icons

- **WHEN** the Layers panel is shown
- **THEN** its fill/adjustment, group, new-layer, and delete buttons each carry their documented icon

#### Scenario: A deferred button is disabled

- **WHEN** the Layers panel is shown before the link, fx, or mask operation exists
- **THEN** that button is disabled
