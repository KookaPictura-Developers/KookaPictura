# Spec Delta

## ADDED Requirements

### Requirement: Layer Lock Layers commands

The system SHALL provide the `Layer > Lock Layers` commands **All**,
**Transparency**, **Image**, and **Position**, each applying the corresponding
lock flag to every layer in the Layers-panel selection. `All` SHALL set the full
lock set. The Background layer SHALL be skipped, as the engine lock setter
already refuses it. Each applied command SHALL recomposite and record exactly
one undo state, and a batch that changes no layer SHALL record no history state.
The commands SHALL be disabled when the panel selection is empty.

The system SHALL also provide `Layer > Lock All Layers In Group…`, which SHALL
apply the full lock set (`All`) to every descendant layer inside the group that
contains the current layer — the current row itself when it is a group, else its
parent group — in one undo state. The command SHALL be disabled when there is no
group to target.

#### Scenario: Lock All locks the selection [llk_cmd_all]

- **WHEN** one or more movable layers are selected and `Layer > Lock Layers >
  All` runs
- **THEN** every selected layer reports the full lock set and the change is one
  undo step

#### Scenario: A single lock flag is applied [llk_cmd_flag]

- **WHEN** a layer is selected and `Layer > Lock Layers > Transparency` runs
- **THEN** that layer reports the transparency bit and the change is one undo
  step

#### Scenario: Lock All Layers In Group locks the group's layers [llk_cmd_group]

- **WHEN** a layer inside a group is current and `Layer > Lock All Layers In
  Group…` runs
- **THEN** every descendant of that group reports the full lock set in one undo
  step, and the group node's own locks are unchanged

#### Scenario: Empty selection disables the commands [llk_cmd_disabled]

- **WHEN** no layer is selected
- **THEN** the `Layer > Lock Layers` commands and `Lock All Layers In Group…` are
  disabled
