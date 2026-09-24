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

