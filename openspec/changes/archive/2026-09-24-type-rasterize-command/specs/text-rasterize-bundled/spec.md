# Specs delta: type-rasterize-command

## ADDED Requirements

### Requirement: Rasterize Layer handles type layers

`PictureView::rasterize_layer` SHALL rasterize a type layer through the bundled
backend: it SHALL attempt `render_text_layer` first and, when that renders,
recomposite, record exactly one `Rasterize Type` history state, and return true.
For a non-type layer it SHALL fall back to the fill-content rasterizer and
record as before. A layer that neither path can rasterize SHALL return false and
record nothing.

#### Scenario: A type layer rasterizes through the command

- **WHEN** `rasterize_layer` is called on a type layer with a decoded style
- **THEN** its text is painted into its channels, it is no longer a type layer, and one `Rasterize Type` state is recorded

#### Scenario: A fill layer still rasterizes as before

- **WHEN** `rasterize_layer` is called on a decodable fill-content layer
- **THEN** the fill is baked and one `Rasterize Layer` state is recorded

#### Scenario: An un-rasterizable layer records nothing

- **WHEN** `rasterize_layer` is called on a layer that has neither a decodable fill nor a type tool
- **THEN** it returns false and records no history state
