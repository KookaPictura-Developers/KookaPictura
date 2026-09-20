# brush-tools Specification

## Purpose
TBD - created by archiving change m21-paint-engine. Update Purpose after archive.
## Requirements
### Requirement: Brush and Pencil tools paint the foreground colour

The system SHALL provide Brush and Pencil as selectable tools that paint the
current foreground colour into the topmost raster layer of the active document.
Painting SHALL be a no-op on a document with no raster layer and SHALL NOT
corrupt the document.

#### Scenario: Painting marks the topmost raster layer

- **WHEN** the Brush tool drags over the canvas and the active document has a raster layer
- **THEN** the composite updates with the foreground colour along the drag

#### Scenario: No raster layer

- **WHEN** a paint tool drags on a document with no raster layer
- **THEN** nothing changes and the document is not corrupted

### Requirement: Paint options bar

The options bar SHALL expose, for Brush and Pencil, a size, hardness, opacity,
flow, and paint-mode control, and for Pencil an Auto Erase toggle. Changing a
control SHALL affect the next stroke and SHALL NOT change the active tool. The
numeric paint controls SHALL use the shared numeric field control, each with a
scrubbing label and a slider popup, and a change to the size SHALL be reflected
in every control bound to the brush size, including the `[`/`]` shortcuts and the
brush-size outline.

#### Scenario: Options follow the paint tool

- **WHEN** the Brush or Pencil tool becomes active
- **THEN** the options bar shows the paint controls

#### Scenario: Changed option applies to the next stroke

- **WHEN** the size is changed before a stroke
- **THEN** the next stroke is painted at the new size

#### Scenario: The size control and shortcuts stay in sync [lbt_size_sync]

- **WHEN** the brush size is changed from the options bar or with `[`/`]`
- **THEN** the other bound control and the brush-size outline show the new size

### Requirement: Canvas pointer routing to the stroke engine

The system SHALL map canvas pointer press, move, and release into a stroke: press
SHALL begin a stroke, move SHALL add samples, and release SHALL commit it. The
image-space coordinates of every sample SHALL be passed to the engine.

#### Scenario: A drag paints one stroke

- **WHEN** the user presses, drags, and releases with a paint tool active
- **THEN** exactly one stroke is committed covering the drag path

### Requirement: Brush size and hardness shortcuts

The system SHALL change the brush diameter with `[` and `]` and the hardness
with `Shift+[` and `Shift+]` for the Brush tool, clamped to the documented
ranges.

#### Scenario: Diameter shortcut

- **WHEN** `]` is pressed with the Brush tool active
- **THEN** the brush diameter increases by one step, clamped to the maximum

#### Scenario: Hardness shortcut

- **WHEN** `Shift+[` is pressed with the Brush tool active
- **THEN** the brush hardness decreases by one step, clamped to zero

### Requirement: Brush-size outline overlay

While the Brush or Pencil tool is active over the canvas, the system SHALL draw a
circle outline at the pointer position sized to the current brush diameter in
image space, so the outline scales with zoom, and SHALL update it as the brush
size or the pointer position changes. The outline SHALL be a drawn overlay, not an
OS cursor pixmap, and SHALL be hidden when a paint tool is not active or the
pointer leaves the canvas.

#### Scenario: The outline tracks the brush size [lbt_outline_size]

- **WHEN** the brush size changes while a paint tool hovers the canvas
- **THEN** the drawn circle's diameter changes to match

#### Scenario: The outline scales with zoom [lbt_outline_zoom]

- **WHEN** the canvas is zoomed while a paint tool hovers the canvas
- **THEN** the circle is drawn at the brush diameter in image space, so its
  on-screen size scales with the zoom

#### Scenario: No outline for other tools [lbt_outline_hidden]

- **WHEN** a tool other than Brush or Pencil is active, or the pointer leaves the
  canvas
- **THEN** no brush-size circle is drawn

