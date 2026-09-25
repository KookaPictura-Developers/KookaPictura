# Specs delta: text-backend-drop-fontdue

## ADDED Requirements

### Requirement: The bundled text backend is one pure-Rust stack

The bundled text backend SHALL depend only on pure-Rust font crates: `rustybuzz`
for shaping, `ttf-parser` for glyph count, character lookup, and line metrics,
and `swash` for rasterization. It SHALL NOT keep a second rasterizer or metrics
library that nothing uses. The backend SHALL remain free of any C dependency.

#### Scenario: No redundant font library

- **WHEN** the crate's dependency list is inspected
- **THEN** shaping is `rustybuzz`, rasterization is `swash`, metrics/lookup are `ttf-parser`, and no other font rasterizer or metrics crate is present

#### Scenario: Metrics still resolve from the bundled face

- **WHEN** the backend needs the glyph count, the glyph id for a character, or the ascent
- **THEN** each is answered by the bundled `ttf-parser` face

#### Scenario: Text rendering is unchanged

- **WHEN** the existing shape, raster, render, kerning, and subpixel tests run
- **THEN** they pass with the same assertions
