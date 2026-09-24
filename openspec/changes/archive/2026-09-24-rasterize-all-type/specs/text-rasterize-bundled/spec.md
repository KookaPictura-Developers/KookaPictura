# Specs delta: rasterize-all-type

## ADDED Requirements

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
