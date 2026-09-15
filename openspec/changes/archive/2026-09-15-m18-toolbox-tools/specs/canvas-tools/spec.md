## ADDED Requirements

### Requirement: Move tool translates the active layer
The system SHALL translate the topmost pixel layer of the active document by a
dragged offset, recompositing the document. Group and adjustment layers SHALL be
ignored, and an empty or absent document SHALL be a no-op.

#### Scenario: Drag moves the layer content
- **WHEN** the Move tool is dragged by a delta and the active document has a pixel layer
- **THEN** that layer's content is translated by the delta and the composite updates

#### Scenario: No pixel layer
- **WHEN** the Move tool is dragged and the document has no pixel layer
- **THEN** nothing changes and the document is not corrupted

### Requirement: Crop tool
The system SHALL let the Crop tool define a rectangular region over the canvas
and commit it on Enter, cropping the document destructively to that region. The
system SHALL also expose `Image > Crop`, which crops to the current selection
bounds.

#### Scenario: Crop by tool
- **WHEN** the Crop tool defines a rectangle and Enter is pressed
- **THEN** the document is resized to the rectangle and its content is shifted so the region's top-left becomes the origin

#### Scenario: Crop by selection
- **WHEN** `Image > Crop` is invoked with an active selection
- **THEN** the document is cropped to the selection bounds

#### Scenario: Crop without a region
- **WHEN** crop is committed with no pending region and no selection
- **THEN** nothing changes

### Requirement: Eyedropper samples a colour
The system SHALL sample the composited colour at a canvas point, expose it as a
packed ARGB value, and record it as the frame's foreground colour.

#### Scenario: Sample a pixel
- **WHEN** the Eyedropper is clicked at a canvas point over an opaque red pixel
- **THEN** the sampled ARGB is opaque red and the frame's foreground colour becomes red

#### Scenario: Sample outside the document
- **WHEN** the Eyedropper is clicked outside the document bounds
- **THEN** sampling fails and the foreground colour is unchanged

### Requirement: Hand and Zoom tools
The system SHALL provide Hand and Zoom as first-class tools. Hand SHALL pan the
canvas on drag; Zoom SHALL magnify on click and reduce on modified click.

#### Scenario: Zoom in by click
- **WHEN** the Zoom tool is clicked on the canvas
- **THEN** the canvas magnification increases

#### Scenario: Zoom out by modified click
- **WHEN** the Zoom tool is clicked while a modifier is held
- **THEN** the canvas magnification decreases
