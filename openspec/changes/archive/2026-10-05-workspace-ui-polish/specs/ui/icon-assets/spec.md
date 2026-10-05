## MODIFIED Requirements

### Requirement: Panel and path glyph set

The system SHALL provide standalone SVG assets for the Layers-panel glyphs
`layers.search`, `layers.kindShape`, `layers.kindSmartObject`, and
`layers.reset`, for the Paths-panel glyphs `path.thumbnail`, `path.fill`,
`path.stroke`, `path.loadSelection`, `path.makeWorkPath`, `path.newPath`, and
`path.delete`, and for the panel-group corner glyph `panel.menu`. Each asset SHALL
use the 24×24 grid, SHALL use `stroke="currentColor"` so it is tinted at render
time and needs no baked colour, SHALL be registered in the Qt resource
`assets/pictura.qrc`, and SHALL resolve through `icon(id)` like every other
bundled icon.

#### Scenario: The new Layers glyphs resolve [plg_layers_resolve]

- **WHEN** `icon(id)` is called for each of `layers.search`,
  `layers.kindShape`, `layers.kindSmartObject`, and `layers.reset`
- **THEN** each call returns a non-null icon whose pixmap is non-null at 16 px

#### Scenario: The Paths glyphs resolve [plg_path_resolve]

- **WHEN** `icon(id)` is called for each of `path.thumbnail`, `path.fill`,
  `path.stroke`, `path.loadSelection`, `path.makeWorkPath`, `path.newPath`, and
  `path.delete`
- **THEN** each call returns a non-null icon whose pixmap is non-null at 16 px

#### Scenario: The corner menu glyph resolves [plg_panel_menu_resolve]

- **WHEN** `icon("panel.menu")` is called
- **THEN** it returns a non-null icon whose pixmap is non-null at 16 px and the
  asset is recorded in `assets/icons/lucide-map.json`

#### Scenario: An unknown id stays null [plg_unknown_null]

- **WHEN** `icon(id)` is called for an id with no bundled asset
- **THEN** it returns a null icon without crashing
