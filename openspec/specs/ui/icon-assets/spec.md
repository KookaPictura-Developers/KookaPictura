# icon-assets Specification

## Purpose
The original SVG icon set covering the app, tools, and menus, bundled through Qt resources and resolved by id.

## Requirements

### Requirement: Icon set coverage
The system SHALL provide an original SVG icon for the application, for each
implemented tool, and for each command that has a registered handler. Icon files
SHALL live under `assets/icons/` and SHALL be named exactly by their icon id
(the command id verbatim, or `tool.<tool>` for tools; the application icon is
`app`).

#### Scenario: Every implemented command has an icon
- **WHEN** the implemented-command id set is enumerated
- **THEN** an SVG file exists for each id

#### Scenario: Every tool has an icon
- **WHEN** the tool set is enumerated
- **THEN** an SVG file exists for each `tool.<tool>` id

### Requirement: Resource-bundled icon loading
The system SHALL bundle the SVG icons into the executable through a Qt resource
and SHALL expose `QIcon icon(const QString& id)` that resolves the id to a
non-null icon.

#### Scenario: A known id resolves
- **WHEN** `icon(id)` is called for a bundled id
- **THEN** it returns a non-null `QIcon`

#### Scenario: An unknown id is null
- **WHEN** `icon(id)` is called for an id with no asset
- **THEN** it returns a null `QIcon` without crashing

### Requirement: Application icon
The system SHALL set the application/window icon from the `app` asset at
startup.

#### Scenario: Window icon is set
- **WHEN** the application starts
- **THEN** the main window's icon is the application icon

### Requirement: Tool and menu icons are displayed
The system SHALL display the tool icon on each Tools-panel action and the command
icon on each implemented menu action.

#### Scenario: Tools panel shows icons
- **WHEN** the Tools panel is shown
- **THEN** every tool button carries its tool icon

#### Scenario: Menu actions show icons
- **WHEN** a menu containing an implemented command is opened
- **THEN** that action carries its command icon

### Requirement: Icons are scalable
The system SHALL load icons from vector SVG so they render crisply at the
application's device pixel ratio and at the sizes the widgets request.

#### Scenario: HiDPI request
- **WHEN** an icon is requested at 2x device pixel ratio
- **THEN** the returned icon provides a suitably sized pixmap rather than a blurry upscale

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
