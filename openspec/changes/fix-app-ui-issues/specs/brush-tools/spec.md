## MODIFIED Requirements

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

## ADDED Requirements

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
