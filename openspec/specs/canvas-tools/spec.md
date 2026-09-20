# canvas-tools Specification

## Purpose
TBD - created by archiving change m18-toolbox-tools. Update Purpose after archive.
## Requirements
### Requirement: Move tool translates the active layer
The system SHALL translate the topmost pixel layer of the active document by a
dragged offset, recompositing the document. Group and adjustment layers SHALL be
ignored, and an empty or absent document SHALL be a no-op. A layer whose lock
state includes the position lock SHALL be refused, and the refusal SHALL leave
the document unchanged with no history state.

#### Scenario: Drag moves the layer content
- **WHEN** the Move tool is dragged by a delta and the active document has a pixel layer
- **THEN** that layer's content is translated by the delta and the composite updates

#### Scenario: No pixel layer
- **WHEN** the Move tool is dragged and the document has no pixel layer
- **THEN** nothing changes and the document is not corrupted

#### Scenario: A position-locked layer is refused [lct_move_locked]
- **WHEN** the Move tool is dragged and the topmost pixel layer is position-locked
- **THEN** the layer does not move, the composite is unchanged, and no history
  state is added

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
canvas on drag; Zoom SHALL magnify on click and reduce on modified click. The
canvas offset SHALL be clamped by one shared range helper at every mutation point
— pan, zoom anchoring, centre, fit, actual pixels, initial view, resize, and the
Navigator proxy — so that at least one display inch of 96 logical pixels of the
canvas remains visible on each axis whenever the document is larger than that
axis. The clamp SHALL slide each axis by the minimum needed rather than
recentering, and the canvas SHALL remain the single source of the offset.

#### Scenario: Zoom in by click
- **WHEN** the Zoom tool is clicked on the canvas
- **THEN** the canvas magnification increases

#### Scenario: Zoom out by modified click
- **WHEN** the Zoom tool is clicked while a modifier is held
- **THEN** the canvas magnification decreases

#### Scenario: Panning cannot push the canvas fully off-screen [lct_pan_margin]
- **WHEN** the canvas is panned in any direction past the reveal margin
- **THEN** the offset is clamped so at least 96 logical pixels of the canvas stay
  visible on that axis and the canvas does not jump to the other side

#### Scenario: Zoom anchoring is clamped [lct_zoom_clamp]
- **WHEN** a zoom-out at a corner would place the canvas outside the reveal margin
- **THEN** the resulting offset is clamped by the same shared range helper

#### Scenario: Fit and centre remain fully visible [lct_fit_within]
- **WHEN** Fit on Screen or centre is applied
- **THEN** the resulting offset is inside the shared range and the canvas is not
  pushed off-screen

