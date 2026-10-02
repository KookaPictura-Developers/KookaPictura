# icon-assets Specification

## Purpose

Add the remaining panel and path line-art glyphs to the bundled icon set, so the
Layers and Paths panel buttons resolve through the same `icon(id)` lookup as
every other asset.

## ADDED Requirements

### Requirement: Panel and path glyph set

The system SHALL provide standalone SVG assets for the Layers-panel glyphs
`layers.search`, `layers.kindShape`, `layers.kindSmartObject`, and
`layers.reset`, and for the Paths-panel glyphs `path.thumbnail`, `path.fill`,
`path.stroke`, `path.loadSelection`, `path.makeWorkPath`, `path.newPath`, and
`path.delete`. Each asset SHALL use the 24×24 grid, SHALL be pre-tinted so it
needs no render-time colour substitution, SHALL be registered in the Qt
resource `assets/pictura.qrc`, and SHALL resolve through `icon(id)` like every
other bundled icon.

#### Scenario: The new Layers glyphs resolve [plg_layers_resolve]
- **WHEN** `icon(id)` is called for each of `layers.search`,
  `layers.kindShape`, `layers.kindSmartObject`, and `layers.reset`
- **THEN** each call returns a non-null icon whose pixmap is non-null at 16 px

#### Scenario: The Paths glyphs resolve [plg_path_resolve]
- **WHEN** `icon(id)` is called for each of `path.thumbnail`, `path.fill`,
  `path.stroke`, `path.loadSelection`, `path.makeWorkPath`, `path.newPath`, and
  `path.delete`
- **THEN** each call returns a non-null icon whose pixmap is non-null at 16 px

#### Scenario: An unknown id stays null [plg_unknown_null]
- **WHEN** `icon(id)` is called for an id with no bundled asset
- **THEN** it returns a null icon without crashing
