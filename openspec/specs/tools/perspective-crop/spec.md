# tools/perspective-crop Specification

## Purpose
The Perspective Crop tool: straighten and crop a user-marked quadrilateral through a homography warp of every layer.

## Requirements

### Requirement: Perspective crop warp

`pictura_render::perspective_crop(doc, quad)` SHALL map the quad (TL, TR, BR,
BL, document pixels) onto a `w×h` canvas, where `perspective_crop_size(quad)` is
the rounded longer length of each pair of opposite edges clamped to 1–30 000.
Every pixel layer's channels and layer mask SHALL be resampled bilinearly
through the inverse homography onto the new canvas, with alpha 0 outside the
source for a layer with alpha, white for a layer without alpha, and the mask's
default colour for a mask; document extra channels SHALL warp the same way;
channel-less layers SHALL be unchanged. It SHALL refuse without change a
degenerate quad and any document for which `perspective_crop_refusal` gives a
reason (live type, smart objects, vector masks, retained 16/32-bit samples).

#### Scenario: The canvas quad is the identity

- **WHEN** a document is perspective-cropped to its own corners
- **THEN** its size and pixels are unchanged

#### Scenario: A skewed region is straightened

- **WHEN** a trapezoid's corners are the quad
- **THEN** the document takes `perspective_crop_size` and the trapezoid fills its middle row

#### Scenario: Refusals change nothing

- **WHEN** the quad is degenerate or the document holds a type layer
- **THEN** `perspective_crop` returns false and the document is unchanged

### Requirement: Perspective Crop tool

The Perspective Crop tool SHALL stage a quad from a drag, move a corner dragged
within 8 screen pixels of it, and move the whole quad when dragged inside. Enter
or a double-click inside the quad (not on a corner) SHALL commit it as one
"Perspective Crop" history state and drop the selection; a refused commit SHALL
keep the quad staged and report why; Escape SHALL discard it. Starting a quad on
a refused document SHALL report the reason. The canvas SHALL shade outside the
quad and draw its edges, a perspective-following 3×3 grid, and corner handles.

#### Scenario: Enter commits a staged quad

- **WHEN** the `crop_group` self-test drags a box over a 20×20 black square in a 40×40 image and presses Enter
- **THEN** one "Perspective Crop" state is added and the document is 20×20 and black

#### Scenario: A degenerate quad stays staged

- **WHEN** a corner is dragged onto its neighbour and Enter is pressed
- **THEN** no state is added, the size is unchanged, and the quad is still shown

### Requirement: Perspective Crop pointer

With the Perspective Crop tool active, the canvas pointer SHALL be a precise
crosshair (hot spot at its centre), and SHALL change to the move cursor while
hovering within 8 screen pixels of a staged quad's corner handle.

#### Scenario: Crosshair and corner cursor

- **WHEN** a quad is staged and the pointer hovers inside it and then over a corner
- **THEN** the pointer is the crosshair inside and the move cursor over the corner
