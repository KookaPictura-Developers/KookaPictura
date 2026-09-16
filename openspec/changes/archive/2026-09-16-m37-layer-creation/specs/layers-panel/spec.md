## MODIFIED Requirements

### Requirement: Layer operations
The system SHALL offer Add Adjustment, New Layer, New Group, Duplicate Layer,
Delete Layer, and Move Layer Up/Down from the panel, applying them to the active
document through the bridge, and the document's rows SHALL update afterwards. A
new layer SHALL be an empty, fully transparent raster layer at the document size
and SHALL NOT change the composite. A new group SHALL be an empty group with a
Normal blend. Duplicating a layer SHALL deep-copy it — children, mask,
adjustment data, and all attributes included — directly above the source, and
SHALL name the copy `"<name> copy"`. A new layer, group, or duplicate SHALL be
inserted directly above the currently selected layer, or at the top of the stack
when there is no selection, and SHALL become the selected row. Each applied
operation SHALL mark the document modified and be one undoable step.

#### Scenario: Add a layer
- **WHEN** the user clicks New Layer with a layer selected
- **THEN** an empty transparent layer is inserted directly above it, becomes selected, the row count grows by one, and the composite is unchanged

#### Scenario: Add a group
- **WHEN** the user clicks New Group with no layer selected
- **THEN** an empty group is inserted at the top of the stack, becomes selected, and a new row appears

#### Scenario: Duplicate a layer
- **WHEN** the user duplicates the selected layer
- **THEN** a deep copy named `"<name> copy"` is inserted directly above it, the row count grows by one, and the copy becomes selected

#### Scenario: Delete a layer
- **WHEN** the user deletes the selected layer
- **THEN** the layer is removed and the panel updates

#### Scenario: Move a layer
- **WHEN** the user moves the selected layer up or down
- **THEN** its order changes and the composite updates

## ADDED Requirements

### Requirement: Layer grouping commands
The system SHALL offer Group Layers and Ungroup Layers from the panel and the
Layer menu, acting on the active document's currently selected layer. Group
Layers SHALL wrap that layer in a new group at the same stack position, with the
group taking the layer's slot and the layer becoming its only child, and SHALL
name the group `"Group N"` where N is one more than the highest existing
`Group <number>` name. Ungroup Layers SHALL splice a group's children into the
parent at the group's position, preserving their order, and SHALL be refused for
a layer that is not a group. Each applied operation SHALL be one undoable step.

#### Scenario: Group a layer
- **WHEN** the user runs Group Layers on a selected layer
- **THEN** the layer is wrapped in a new group that occupies the layer's position and the wrapped layer becomes its only child

#### Scenario: Ungroup a group
- **WHEN** the user runs Ungroup Layers on a selected group with children
- **THEN** the group is replaced in place by its children in their existing order

#### Scenario: Ungroup refuses a non-group
- **WHEN** the user runs Ungroup Layers on a pixel or adjustment layer
- **THEN** the operation is refused and the document is unchanged
