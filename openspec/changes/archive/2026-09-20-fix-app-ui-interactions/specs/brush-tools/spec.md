## MODIFIED Requirements

### Requirement: Brush and Pencil tools paint the foreground colour

The system SHALL provide Brush and Pencil as selectable tools that paint the
current foreground colour into the active layer of the active document, where the
active layer is the exactly-one layer resolved by the shared active-layer
resolver. Painting SHALL be refused, leaving the document unchanged, when no layer
is active, when more than one layer is selected, or when the active layer is not
a raster layer; it SHALL NOT fall back to the topmost raster layer. Painting on a
layer whose lock state includes `TRANSPARENCY` SHALL change only the color of
pixels whose pre-existing alpha is greater than zero and SHALL preserve each such
pixel's alpha.

#### Scenario: Painting marks the active layer

- **WHEN** the Brush tool drags over the canvas and a single raster layer is
  active
- **THEN** the composite updates with the foreground colour along the drag on
  that layer

#### Scenario: No active layer

- **WHEN** a paint tool drags with no layer active or with more than one layer
  selected
- **THEN** the stroke is refused, nothing changes, and the document is not
  corrupted

#### Scenario: A transparency-locked layer keeps its alpha [lbt_paint_alpha]

- **WHEN** the Brush paints on a transparency-locked layer
- **THEN** each painted pixel's colour changes and its alpha equals its
  pre-stroke value

### Requirement: Brush-size outline overlay

While the Brush or Pencil tool is active over the canvas, the system SHALL draw a
circle outline at the pointer position sized to the current brush diameter in
image space, so the outline scales with zoom, and SHALL update it as the brush
size or the pointer position changes. The outline SHALL be a drawn overlay, not an
OS cursor pixmap, and SHALL be hidden when a paint tool is not active or the
pointer leaves the canvas. The outline SHALL NOT be clipped to the document's
image rectangle: it SHALL render outside the canvas bounds while remaining inside
the canvas widget, so it stays visible when the pointer is near or beyond the
document edge.

#### Scenario: The outline tracks the brush size [lbt_outline_size]

- **WHEN** the brush size changes while a paint tool hovers the canvas
- **THEN** the drawn circle's diameter changes to match

#### Scenario: The outline scales with zoom [lbt_outline_zoom]

- **WHEN** the canvas is zoomed while a paint tool hovers the canvas
- **THEN** the circle is drawn at the brush diameter in image space, so its
  on-screen size scales with the zoom

#### Scenario: The outline renders outside the document [lbt_outline_outside]

- **WHEN** the brush pointer is near or beyond the document edge
- **THEN** the portion of the circle outside the document bounds is still drawn,
  bounded only by the canvas widget

#### Scenario: No outline for other tools [lbt_outline_hidden]

- **WHEN** a tool other than Brush or Pencil is active, or the pointer leaves the
  canvas
- **THEN** no brush-size circle is drawn

## ADDED Requirements

### Requirement: Brush and Pencil hide the mouse cursor

While Brush or Pencil is active over the canvas, the system SHALL set a blank
mouse cursor (`Qt::BlankCursor`) rather than a brush/pencil cursor pixmap or an
unset cursor, so the drawn brush-size ring is the only pointer affordance. The
ring SHALL remain driven by the existing outline overlay.

#### Scenario: The paint cursor is blank [lbt_blank_cursor]

- **WHEN** Brush or Pencil is active over the canvas
- **THEN** the mouse cursor is blank and the brush-size ring is drawn

#### Scenario: The cursor is restored for other tools [lbt_blank_cursor_restore]

- **WHEN** a non-paint tool becomes active or the pointer leaves the canvas
- **THEN** the blank cursor is cleared and the tool's normal cursor is used
