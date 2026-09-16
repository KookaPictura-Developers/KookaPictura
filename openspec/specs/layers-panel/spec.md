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
The system SHALL apply visibility, name, blend mode, and opacity edits to the
active document through the bridge, and each applied edit SHALL mark the
document modified and be undoable.

#### Scenario: Change opacity
- **WHEN** the user changes a layer's opacity in the panel
- **THEN** the document's layer opacity changes, the composite updates, and the document reports modified

#### Scenario: Change blend mode
- **WHEN** the user selects a blend mode for a layer
- **THEN** the layer's blend mode changes and the composite updates

#### Scenario: Rename a layer
- **WHEN** the user edits a layer's name
- **THEN** the layer name changes and the tab title reflects an untitled-name change where applicable

### Requirement: Layer operations
The system SHALL offer Add Adjustment, Delete Layer, and Move Layer Up/Down from
the panel, applying them to the active document through the bridge.

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

