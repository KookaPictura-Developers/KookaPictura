# Specs delta: text-qt-backend

## ADDED Requirements

### Requirement: The Rasterize Type command can render through the Qt backend

`PictureView::rasterize_type` SHALL attempt a Qt `QFont`/`QPainter` text render
of the layer's text and style and, when it yields a non-empty raster, replace
the layer's channels with it, drop `TySh`, clear the type tool, and record one
`Rasterize Type` state. When the Qt render is empty it SHALL fall back to the
bundled backend. A non-type layer SHALL still be refused without mutation.

#### Scenario: A type layer rasterizes through Qt

- **WHEN** `rasterize_type` is called on a type layer
- **THEN** the layer's channels hold the Qt-rendered text, it is no longer a type layer, and one state is recorded

#### Scenario: An empty Qt render falls back to the bundled face

- **WHEN** the Qt render yields no pixels
- **THEN** the bundled backend materializes the layer instead

#### Scenario: A non-type layer is refused

- **WHEN** `rasterize_type` is called on a plain pixel layer
- **THEN** it returns false and records no state

### Requirement: The Qt text helper is directly testable

`pictura-render` SHALL expose `materialize_text_rgba(document, path, rgba)` that
replaces a layer's colour and alpha channels from a packed RGBA buffer of the
layer-rect size, drops `TySh`, and clears the type tool, returning false without
mutating for a wrong-size buffer or a missing layer.

#### Scenario: A wrong-size buffer is refused

- **WHEN** `materialize_text_rgba` is called with a buffer that is not `width*height*4` bytes
- **THEN** it returns false and the layer is unchanged
