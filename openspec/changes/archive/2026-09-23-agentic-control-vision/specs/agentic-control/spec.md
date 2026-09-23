## ADDED Requirements

### Requirement: Screenshot readback

The server SHALL provide a `screenshot` method with a `scope` of `window`,
`canvas`, or `document` (default `window`) and an optional `max_dim` (default
1280). It SHALL return `{mime:"image/png", base64, width, height, source_width,
source_height}`: the PNG of the whole frame (`window`), the active canvas widget
including its checkerboard/zoom/overlay (`canvas`), or the raw composited
document image (`document`). When the image's long edge exceeds `max_dim` it
SHALL be scaled down preserving aspect ratio, and the pre-scale dimensions SHALL
be reported as `source_width`/`source_height`; `width`/`height` are the returned
image's. `canvas` and `document` with no active document SHALL return
`no_document`. A non-positive `max_dim` SHALL be `invalid_param`.

#### Scenario: A window screenshot is a PNG

- **WHEN** a client invokes `screenshot` with `scope` `window`
- **THEN** the result's `mime` is `image/png`, its `base64` decodes to a PNG, and `width`/`height` match the decoded image

#### Scenario: A canvas screenshot with no document is reported

- **WHEN** a client invokes `screenshot` with `scope` `canvas` and no document is open
- **THEN** the response has error code `no_document`

#### Scenario: max_dim downscales and reports the source size

- **WHEN** a client invokes `screenshot` with a `max_dim` smaller than the source long edge
- **THEN** the returned image's long edge is at most `max_dim`, `width`/`height` are the returned image's dimensions, and `source_width`/`source_height` are the original ones

#### Scenario: A non-positive max_dim is rejected

- **WHEN** a client invokes `screenshot` with `max_dim` 0 or negative
- **THEN** the response has error code `invalid_param`

### Requirement: UI tree readback

The server SHALL provide a `ui_tree` method with optional `max_depth` (default
12) and `max_children` (default 64) limits. It SHALL return a depth- and
breadth-capped array of nodes, each `{class, objectName, rect:{x,y,w,h},
visible, enabled, text, tooltip}`, where `rect` is in window (frame) coordinates.
A default call SHALL include the layers panel's object name so an agent can
locate it (the panel sits ~9 widget levels below the frame, well within the
default depth).

#### Scenario: The default tree contains the layers panel

- **WHEN** a client invokes `ui_tree` with no parameters
- **THEN** some node's `objectName` is `layersPanel` and its `rect` is a window-local rectangle

#### Scenario: The walk is bounded

- **WHEN** a client invokes `ui_tree` with small `max_depth`/`max_children`
- **THEN** the returned tree does not exceed those bounds and the server responds without error

### Requirement: Layer thumbnail readback

The server SHALL provide a `layer_thumbnail` method with `index` and an optional
`size` (default 64). For a layer that has a thumbnail it SHALL return the same
image shape as `screenshot` (`mime`, `base64`, `width`, `height`,
`source_width`, `source_height`); for a group, an adjustment layer, or an
out-of-range index it SHALL return `invalid_param`.

#### Scenario: A pixel layer has a thumbnail

- **WHEN** a client invokes `layer_thumbnail` for a pixel layer
- **THEN** the result's `mime` is `image/png` and its `base64` decodes to a PNG of at most `size` on the long edge

#### Scenario: A group layer has no thumbnail

- **WHEN** a client invokes `layer_thumbnail` for a group layer
- **THEN** the response has error code `invalid_param`

### Requirement: Vision verification

The self-test SHALL include a block that exercises the three vision methods over
the control socket, using the next append-only check codes. It SHALL assert that
a `screenshot` is a PNG and that its reported dimensions match the decoded image,
that a downscaled `screenshot` reports the source dimensions, that `ui_tree`
contains the layers panel, and that a pixel layer's `layer_thumbnail` is a PNG
while a group's is `invalid_param`.

#### Scenario: The vision block proves the methods offscreen

- **WHEN** the self-test runs the vision block under `--headless`
- **THEN** every vision assertion passes and the block reports pass, using no display
