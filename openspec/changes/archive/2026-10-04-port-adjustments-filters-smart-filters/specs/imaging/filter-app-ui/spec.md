# Spec Delta

## ADDED Requirements

### Requirement: Artistic filter family exposure

The `Filter` menu SHALL contain an `Artistic` submenu listing all fifteen CS6 Artistic filters in CS6 order (`Colored Pencil`, `Cutout`, `Dry Brush`, `Film Grain`, `Fresco`, `Neon Glow`, `Paint Daubs`, `Palette Knife`, `Plastic Wrap`, `Poster Edges`, `Rough Pastels`, `Smudge Stick`, `Sponge`, `Underpainting`, `Watercolor`). Each entry SHALL open its parameter dialog, map to the corresponding engine kind, and be enabled exactly when the active layer is a filter target; a filter with no parameters SHALL apply directly.

#### Scenario: Every Artistic entry opens its dialog and applies

- **WHEN** each `Filter ▸ Artistic` entry is chosen on a document with an editable pixel layer
- **THEN** it maps to its engine kind and either opens the parameter dialog or applies directly, and the layer's colour planes change

#### Scenario: Disabled without a filter target

- **WHEN** no editable pixel layer is active
- **THEN** the Artistic entries are disabled

### Requirement: New filter-kind mapping

The filter mapping SHALL additionally recognise `lighting-effects` (19 slots, default Spot light), `diffuse` (one mode slot, default Normal), and `glowing-edges` (width, brightness, smoothness; defaults 2, 6, 1), each with an empty slot list producing those defaults and a non-empty list of the wrong length refused. These kinds SHALL drive `Filter ▸ Render ▸ Lighting Effects`, `Filter ▸ Stylize ▸ Diffuse`, and `Filter ▸ Stylize ▸ Glowing Edges`.

#### Scenario: The new kinds resolve and default

- **WHEN** each new kind is resolved with an empty slot list
- **THEN** the corresponding `Filter` with the listed defaults is produced

#### Scenario: Wrong arity is refused

- **WHEN** a new kind is resolved with a slot list of the wrong length
- **THEN** no filter is produced and the command returns false without changing the document

### Requirement: Convert for Smart Filters command

`Filter ▸ Convert for Smart Filters` SHALL convert the active raster layer into a smart object so that the layer's content is carried as an embedded source, while leaving the layer's pixels unchanged. It SHALL be enabled only when the active layer can be converted to a smart object, and on success it SHALL recomposite and record one history state named `Convert for Smart Filters`.

#### Scenario: Converting preserves the pixels and records one state

- **WHEN** `Convert for Smart Filters` is invoked on a convertible raster layer
- **THEN** the layer becomes a smart object with the same composited pixels and exactly one history state is recorded

#### Scenario: Refused when not convertible

- **WHEN** the active layer cannot be converted to a smart object
- **THEN** the command is disabled or refused and the document is unchanged
