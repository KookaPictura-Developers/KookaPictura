## ADDED Requirements

### Requirement: Layer similarity transform entry point and refusal

The system SHALL provide
`pictura_render::transform_layer(doc: &mut Document, path: &str, transform:
LayerTransform) -> bool`, where `LayerTransform` carries `scale_x: f64`,
`scale_y: f64`, `angle_radians: f64`, `dx: f64`, and `dy: f64`. On success it
SHALL resample every channel plane of the layer at `path` into the transformed
bounding rect, update that layer's `rect`, and return `true`. The operation SHALL
refuse and MUST leave `doc` bit-identical when the path does not resolve, the
target is a group, an adjustment layer, or a Background layer, the target is
position-locked, the source rect has zero area, any transform parameter is
non-finite, either scale factor is zero or below a small epsilon, or the
computed result rect has zero area. The operation SHALL NOT recomposite the
document; the caller owns the composite refresh. It MUST NOT panic on any input,
including a 1×1 source.

#### Scenario: A scale and rotation updates the rect and channels

- **WHEN** `transform_layer` is called on a raster layer with a 90° rotation and a 2× uniform scale
- **THEN** it returns true, the layer `rect` is the transformed bounding box, and every channel plane is sized to that rect

#### Scenario: Group, adjustment, Background, and locked targets are refused

- **WHEN** `transform_layer` is called on a group, an adjustment layer, a Background layer, or a position-locked layer
- **THEN** it returns false and every field of `doc` equals its pre-call value

#### Scenario: Degenerate transforms are refused

- **WHEN** `transform_layer` is called with a zero scale factor, a non-finite value, or a source rect of zero area
- **THEN** it returns false and the document is unchanged

#### Scenario: The op does not recomposite

- **WHEN** `transform_layer` succeeds
- **THEN** the layer channels and rect change but `doc.composite` is left for the caller to rebuild

### Requirement: Bilinear resampling and coverage rules

The transform SHALL map a layer-local point `p` by
`p' = c + R(θ)·(S·(p − c)) + (dx, dy)`, where `c` is the source rect centre, `S`
is `diag(scale_x, scale_y)`, and `R(θ)` is a clockwise-positive screen-space
rotation matching `pictura_ops::rotate_arbitrary`. The result `rect` SHALL be the
integer bounding box of the four transformed corners: `left = floor(min x')`,
`top = floor(min y')`, `right = ceil(max x')`, `bottom = ceil(max y')`. Each
destination pixel SHALL be sampled with bilinear interpolation from the inverse
map `src = c + S⁻¹·R(−θ)·(q − c) − (dx, dy)`; every channel plane is resampled
independently. A destination whose source point lies outside
`[0, w) × [0, h)` SHALL be written as `0` for every channel (transparent for the
alpha plane, black for colour), and bilinear taps SHALL be edge-clamped. When the
layer carries a `LayerMask`, the mask SHALL be transformed by the same
document-space matrix about its own rect, its `rect` updated to the transformed
bounding box, and out-of-source mask samples written as `0`.

#### Scenario: A scale doubles the plane dimensions and rect

- **WHEN** a `w × h` layer is transformed by `scale_x = scale_y = 2.0` with no rotation or offset
- **THEN** the layer rect and every channel plane are `2w × 2h` and the result is a bilinear resample of the source

#### Scenario: Pixels outside the source are transparent

- **WHEN** a layer is rotated by 45° so the destination bounding box has empty corners
- **THEN** every destination pixel whose source point is outside the source rect has all channels equal to 0

#### Scenario: The mask follows the layer

- **WHEN** a masked layer is transformed
- **THEN** the mask rect is the transformed bounding box, the mask plane is resampled, and out-of-source mask samples are 0

### Requirement: Channel-less embedded smart-object materialization

`transform_layer` SHALL materialize the raster proxy of a channel-less embedded
smart object from its payload before resampling, when `path` resolves to such a
layer and its payload renders through the existing embedded source path. On
success it SHALL drop the smart object, its preserved `SoLd`/`plLd` blocks, and
its linked-source record so no stale untransformed source can survive the
transform. A channel-less target that is not a decodable embedded object SHALL
be refused with no mutation.

#### Scenario: A channel-less placed object is transformed

- **WHEN** `transform_layer` runs on a channel-less embedded smart-object layer with a decodable payload
- **THEN** the layer gains resampled channels, its rect is transformed, and `layer.smart_object` is `None`

#### Scenario: An undecodable channel-less target is refused

- **WHEN** the target has no channels and its embedded payload cannot be rendered
- **THEN** the operation returns false and the document is unchanged

### Requirement: Free Transform session lifecycle and undo contract

The system SHALL provide a modal, per-`PictureView` Free Transform session. It
SHALL begin on an explicit target path, hold the target path, the original rect,
and the current `scale_x`, `scale_y`, `angle_radians`, and `(dx, dy)`, and SHALL
expose press, move, and release callbacks that take document-space image
coordinates and modifier flags. Enter/Return SHALL commit: the session SHALL call
`transform_layer` once, recomposite, and record exactly one history state
labelled `"Free Transform"`. Escape SHALL cancel: the session SHALL end with the
document bit-identical to its state before the session began and with no history
state added. Beginning a session on the path already active SHALL be a no-op;
beginning one on a different path SHALL cancel the active session and start a
new session on the new path. Switching tools, switching documents, or closing
the document SHALL cancel the active session. Committing an unchanged (identity)
transform SHALL record no history state.

#### Scenario: Begin, gesture, commit records one state

- **WHEN** a session is begun on a raster layer, a scale gesture is applied, and Enter is pressed
- **THEN** exactly one history state labelled `"Free Transform"` is added and the layer rect reflects the gesture

#### Scenario: Escape restores the document exactly

- **WHEN** a session is begun, a rotation gesture is applied, and Escape is pressed
- **THEN** the document equals its pre-session value and the history count is unchanged

#### Scenario: An identity commit records nothing

- **WHEN** a session is begun and Enter is pressed without any gesture
- **THEN** no history state is added and the document is unchanged

#### Scenario: Beginning on the active path is a no-op

- **WHEN** a session is active on a path and begin is called again on that same path
- **THEN** the active session is unchanged

#### Scenario: Beginning on a different path restarts the session

- **WHEN** a session is active on one path and begin is called on a different transformable path
- **THEN** the first session is cancelled without mutating the document and a new session on the second path begins

#### Scenario: Switching documents cancels the session

- **WHEN** a session is active and the active document changes
- **THEN** the session ends without mutating the original document

### Requirement: Transform bounding-box geometry and hit-testing

The system SHALL compute the session's bounding quad as the four corners of the
source rect under the current transform and SHALL draw the quad with eight scale
handles (four corners and four edge midpoints) and a rotate affordance. Hit
testing SHALL be in screen space with a tolerance of 6 screen pixels for handles;
a press inside the quad but away from a handle SHALL move, and a press within a
20 screen-pixel band outside a corner SHALL rotate about the layer centre. The
canvas SHALL show a diagonal, horizontal, or vertical size cursor over a handle,
an open or closed hand while moving, and a cross cursor in the rotate band.

#### Scenario: Dragging a corner handle scales

- **WHEN** the user presses on a corner handle and drags away from the layer centre
- **THEN** the layer scales about the opposite corner and the bounding quad follows the pointer

#### Scenario: Dragging inside the quad moves

- **WHEN** the user presses inside the quad away from every handle and drags
- **THEN** the layer translates by the pointer delta and does not scale or rotate

#### Scenario: Dragging the rotate band rotates

- **WHEN** the user presses in the rotate band outside a corner and drags
- **THEN** the layer rotates about its centre and the quad rotates with the pointer

### Requirement: Scale and rotate modifiers and minimum size

Holding Shift while dragging a corner handle SHALL lock the aspect ratio by
applying one uniform scale factor to both axes; edge handles SHALL always scale
one axis. Holding Shift while rotating SHALL snap the angle to 15° steps. The
session SHALL clamp the transform so the transformed rect keeps at least 1 pixel
in each axis and SHALL refuse a gesture that would produce a zero-area result.
Negative scale factors SHALL be permitted and flip the content. The reference
point SHALL be the layer centre.

#### Scenario: Shift on a corner keeps the aspect ratio

- **WHEN** the user drags a corner handle with Shift held
- **THEN** both scale factors are equal and the layer aspect ratio is preserved

#### Scenario: Shift rotation snaps to 15° steps

- **WHEN** the user rotates with Shift held
- **THEN** the committed rotation angle is an exact multiple of 15°

#### Scenario: A gesture cannot collapse the layer

- **WHEN** the user drags a handle so the transformed rect would have zero width or height
- **THEN** the session clamps to at least 1 pixel per axis and does not commit a zero-area rect

### Requirement: Live preview reuses the move-preview overlay

The session SHALL preview by drawing a cached base composite with the target
layer hidden plus a cached image of the target layer under the current
`QTransform`, reusing the Move tool's cached base/layer overlay. The session MUST
NOT rebuild the document composite on each mouse move. For a channel-less
embedded target the preview layer image SHALL be rendered from its embedded
source without mutating the document.

#### Scenario: A drag does not recomposite

- **WHEN** a session gesture updates the transform
- **THEN** the cached base and layer images are redrawn with the new transform and no document composite is rebuilt

#### Scenario: Previewing a channel-less target mutates nothing

- **WHEN** a session is begun on a channel-less embedded smart object
- **THEN** the document equals its pre-session value until commit

### Requirement: Place starts a Free Transform session

After a successful Place the application SHALL select the newly placed layer and
start a Free Transform session on it, whether the place came through the
`File > Place…` command or a canvas file drop. For a multi-file canvas drop the
session SHALL target the last successfully placed layer and the other placed
layers SHALL remain where they landed. Cancelling the session SHALL leave every
placed layer where it landed, untransformed, and SHALL NOT remove or roll back
the `"Place"` history state recorded by the placement.

#### Scenario: Menu Place enters the session

- **WHEN** `File > Place…` completes successfully with a supported image
- **THEN** a Free Transform session begins on the new layer and the `"Place"` state remains

#### Scenario: Canvas drop enters the session

- **WHEN** one or more supported files are dropped on the document canvas
- **THEN** a session begins on the last placed layer and every placed layer keeps its `"Place"` state

#### Scenario: Cancelling keeps the placement

- **WHEN** a session started by a Place is cancelled
- **THEN** the placed layer remains at its landed rect and the `"Place"` history state is still present
