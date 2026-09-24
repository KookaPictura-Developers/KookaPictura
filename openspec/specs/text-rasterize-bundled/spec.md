# text-rasterize-bundled Specification

## Purpose
TBD - created by archiving change text-rasterize-bundled. Update Purpose after archive.
## Requirements
### Requirement: A bundled-font, pure-Rust glyph rasterizer

`pictura-render` SHALL bundle one permissively-licensed font and implement
`pictura_core::Rasterizer` over it, so a `RasterRequest` for a glyph yields a
`GlyphMask` whose coverage length is `width * height`, with no Qt and no C
dependency. The bundled font and its license SHALL be recorded in the
repository.

#### Scenario: A glyph rasterizes to coverage

- **WHEN** the bundled rasterizer is asked for a glyph present in the bundled font
- **THEN** it returns a non-empty `GlyphMask` whose coverage length equals `width * height`

#### Scenario: An absent glyph is a no-op

- **WHEN** the requested glyph index has no rasterizable outline
- **THEN** the rasterizer returns `None` and no panic

### Requirement: A type layer materializes into pixels

`pictura-render` SHALL expose `render_text_layer(document, path)` that shapes and
lays out the layer's `TypeTool` text and style with the bundled font, paints the
glyph coverage in the style's fill colour into the layer's colour and alpha
channels, removes the `TySh` block, and clears the type-tool view. It SHALL
return `false` without mutating when the layer has no type tool or style, when
its rect has no area, or when no glyph rendered, so the caller records exactly
one history state on success.

#### Scenario: A type layer becomes a pixel layer

- **WHEN** a type layer with a style is rendered
- **THEN** its channels contain non-empty coverage, `type_tool` is cleared, and the `TySh` block is gone

#### Scenario: A missing style is a no-op

- **WHEN** a layer has a type tool but no decoded style
- **THEN** `render_text_layer` returns `false` and the channels and blocks are unchanged

#### Scenario: Provenance records the substitution

- **WHEN** provenance is produced for a requested family other than the bundled one
- **THEN** it records the requested family, the bundled resolved family, the bundled font hash, and the backend

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

### Requirement: Rasterize All Layers covers type layers

`pictura-render` SHALL expose `rasterize_all_layers(document) -> usize` that
rasterizes every flattened layer through the fill-content path or, failing that,
the bundled text path, and returns how many were rasterized. The command SHALL
record exactly one history state when the count is positive.

#### Scenario: A fill layer and a type layer both rasterize

- **WHEN** a document holds one fill-content layer and one type layer
- **THEN** `rasterize_all_layers` returns 2, the fill is baked, and the type layer is materialized

#### Scenario: Already rasterized layers are skipped

- **WHEN** no layer is fill content or a type layer
- **THEN** `rasterize_all_layers` returns 0

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

