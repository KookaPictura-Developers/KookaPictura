# canvas-tools Specification

## MODIFIED Requirements

### Requirement: Crop tool
The system SHALL place a crop box over the whole canvas when the Crop tool is
chosen (fitted to the aspect ratio when one is set), shading the area outside
it and drawing rule-of-thirds guides, its frame, and eight handles. Dragging a
handle SHALL resize the box (keeping the aspect ratio when one is set), dragging
inside SHALL move it, and dragging outside SHALL draw a new box, with the
matching move/resize cursor over each part. Enter, a double-click inside the
box, or the options bar's Apply SHALL crop the document to the box as one
"Crop" state; Escape or the options bar's Cancel SHALL reset the box to the
canvas. The options bar SHALL offer aspect-ratio presets (Unconstrained, 1:1,
4:5, 5:7, 2:3, 16:9) and Delete Cropped Pixels (on by default): when on, each
pixel layer's pixels and mask outside the new canvas SHALL be discarded (layers
with live type, smart objects, vector masks, or retained 16/32-bit samples keep
theirs); when off they SHALL be kept beyond the canvas edge. A crop SHALL be
pending only while the tool is active and the box differs from the canvas. The
system SHALL also expose `Image > Crop`, which commits a pending crop or else
crops to the current selection bounds.

#### Scenario: Crop by tool
- **WHEN** the Crop tool's box is resized to a region and Enter is pressed
- **THEN** the document is resized to the region and its content is shifted so the region's top-left becomes the origin

#### Scenario: The default box is not a pending crop
- **WHEN** the Crop tool is chosen and its box is left canvas-sized
- **THEN** a crop box is shown, no crop is pending, and `Image > Crop` still crops to the selection

#### Scenario: Move, reset, and ratio lock
- **WHEN** the `crop_tool` self-test resizes the box by its bottom-right handle, drags inside it, presses Escape, and resizes with a 1:1 ratio set
- **THEN** the box resizes, moves by the drag, returns to the canvas, and stays square

#### Scenario: Delete Cropped Pixels
- **WHEN** a crop is committed with Delete Cropped Pixels on, and again with it off
- **THEN** with it on the layer is trimmed to the new canvas, and with it off the layer keeps its pixels beyond the canvas edge

#### Scenario: Crop by selection
- **WHEN** `Image > Crop` is invoked with an active selection
- **THEN** the document is cropped to the selection bounds

#### Scenario: Crop without a region
- **WHEN** crop is committed with no pending region and no selection
- **THEN** nothing changes
