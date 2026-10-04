# Spec Delta

## ADDED Requirements

### Requirement: Smart Filters tree rows

When a smart-object layer carries one or more smart filters, the Layers panel SHALL show a `Smart Filters` parent row directly under that layer, with one child row per filter carrying that filter's name. The parent row SHALL expose a visibility toggle for the group enable flag and each child row SHALL expose a visibility toggle for that filter's enable flag. Toggling either SHALL update the composite, record one history state, and persist the flag to the layer's preserved smart-object descriptor so it survives a write and re-read; the synthetic rows SHALL be non-editable and non-draggable and SHALL NOT appear as selectable layers to layer operations.

#### Scenario: The parent and one child per filter appear

- **WHEN** a smart-object layer with a filterFX chain is present
- **THEN** the panel shows a `Smart Filters` parent with one child row per filter, named for each filter

#### Scenario: Toggling a filter eye updates the composite

- **WHEN** a child row's visibility toggle is flipped
- **THEN** that filter's enable flag flips, the composite updates, and one history state is recorded

#### Scenario: Toggling the group eye bypasses the chain

- **WHEN** the parent row's visibility toggle is flipped off
- **THEN** the group enable flag is cleared and the composite falls back to the unfiltered source

#### Scenario: A toggle survives a write and re-read

- **WHEN** a filtered document whose filter or group visibility was toggled is written and read back
- **THEN** the re-read smart object reports the toggled flag

#### Scenario: Synthetic rows are not real layers

- **WHEN** a smart-filter row is double-clicked to rename or dragged
- **THEN** it is not renamed or moved and no layer operation targets it
