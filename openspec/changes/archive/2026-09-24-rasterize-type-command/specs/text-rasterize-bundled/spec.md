# Specs delta: rasterize-type-command

## ADDED Requirements

### Requirement: A dedicated Rasterize Type command

`PictureView::rasterize_type(path)` SHALL materialize the type layer at `path`
through the bundled backend, recomposite, record exactly one `Rasterize Type`
history state, and return true; it SHALL return false and record nothing for a
layer that is not a type layer. The `Layer > Rasterize > Type` command SHALL be
enabled only when the selected layer is a type layer.

#### Scenario: A type layer rasterizes through the Type command

- **WHEN** `rasterize_type` is called on a type layer with a decoded style
- **THEN** its text is painted into its channels, the layer is no longer a type layer, and one `Rasterize Type` state is recorded

#### Scenario: A non-type layer is refused

- **WHEN** `rasterize_type` is called on a plain pixel layer
- **THEN** it returns false and records no history state

#### Scenario: The command is gated to type layers

- **WHEN** the selected layer is not a type layer
- **THEN** the `layer.rasterize.type` action is disabled
