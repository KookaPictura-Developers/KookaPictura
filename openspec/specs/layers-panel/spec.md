# layers-panel Specification

## Purpose
TBD - created by archiving change m20-panels. Update Purpose after archive.
## Requirements
### Requirement: Layers panel rows
The system SHALL present the top-level layers of the active document as rows,
each showing a visibility toggle, a thumbnail, an editable name, a blend mode,
and an opacity value. The rows SHALL reflect the document and update when the
document changes or the active document changes.

#### Scenario: Rows reflect the document
- **WHEN** a document with layers is active
- **THEN** the panel shows one row per top-level layer with its name, blend mode, opacity, and visibility

#### Scenario: No document
- **WHEN** no document is open
- **THEN** the panel shows no rows and does not crash

### Requirement: Layer property editing

The system SHALL apply visibility, name, blend mode, opacity, fill, lock, and
color-label edits to the active document through the bridge, and each applied
edit SHALL mark the document modified and be undoable. Opacity and fill SHALL
each be reported and edited as a value in `0..=255`, where `255` is fully
opaque. A layer's lock state SHALL be reported and edited per flag
(transparency, image pixels, position, or all). A layer's color label SHALL be
one of `None`, `Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Violet`, or `Gray`.
The system SHALL refuse, rather than apply, an edit that the CS6 contract
excludes: a group has no Fill control, and the Background layer SHALL expose
neither opacity nor fill and SHALL refuse lock and color-label edits. A fully
locked layer SHALL refuse opacity and fill edits; its lock flags SHALL remain
editable so it can be unlocked. A refused edit SHALL return failure and leave
the document unchanged.

#### Scenario: Change opacity

- **WHEN** the user changes a layer's opacity in the panel
- **THEN** the document's layer opacity changes, the composite updates, and the document reports modified

#### Scenario: Change blend mode

- **WHEN** the user selects a blend mode for a layer
- **THEN** the layer's blend mode changes and the composite updates

#### Scenario: Rename a layer

- **WHEN** the user edits a layer's name
- **THEN** the layer name changes and the tab title reflects an untitled-name change where applicable

#### Scenario: Change fill opacity

- **WHEN** the user changes a layer's fill opacity to 128 in the panel
- **THEN** the layer's fill becomes 128, the composite updates with the layer's pixels at half strength, and the edit is undoable as one step

#### Scenario: Toggle a lock flag

- **WHEN** the user enables Lock Position for a layer and then enables Lock All
- **THEN** the layer reports the position bit and then all three lock bits, the panel disables opacity and fill, and each toggle is one undo state

#### Scenario: Set a color label

- **WHEN** the user picks Red from the row's color-label menu
- **THEN** the layer's color label becomes Red and the row reflects it

#### Scenario: Fill is unavailable for a group

- **WHEN** a group is selected
- **THEN** the Fill control is disabled and a fill edit is refused, while Opacity remains editable

#### Scenario: Background and fully locked layers refuse edits

- **WHEN** the Background layer or a fully locked layer is selected and opacity or fill is changed
- **THEN** the edit is refused, the document is unchanged, and the control is disabled

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

### Requirement: Layers panel docking and toggle
The system SHALL host the Layers panel in a registered dock with a stable
`objectName` and SHALL expose a `Window > Panels > Layers` toggle.

#### Scenario: Toggle the Layers panel
- **WHEN** the user toggles Layers from the Window menu
- **THEN** the panel is shown or hidden without changing the document

### Requirement: Thumbnails downsample without a full-size intermediate

Layer thumbnails SHALL be produced by downsampling the layer's channel data
directly, without first materializing a full-resolution RGBA image, so that
thumbnail cost is not proportional to the layer's full pixel buffer. The
thumbnail's visual result SHALL be unchanged.

#### Scenario: A thumbnail of a large layer avoids a full-resolution image

- **WHEN** a 24 px thumbnail is generated for a 4000×4000 layer
- **THEN** no full-resolution RGBA image is allocated, the thumbnail is visually
  unchanged, and generation completes within a small time budget

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

