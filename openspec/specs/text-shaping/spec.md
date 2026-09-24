# text-shaping Specification

## Purpose
TBD - created by archiving change text-shaping-rustybuzz. Update Purpose after archive.
## Requirements
### Requirement: The bundled backend shapes lines with a HarfBuzz-compatible shaper

`pictura-render`'s bundled text backend SHALL shape each line with a
HarfBuzz-compatible shaping algorithm implemented in pure Rust, so the glyph
sequence and the device-pixel advances reflect the font's default `kern`/GPOS
and GSUB substitutions. The `ShapedGlyph { id, advance }` contract SHALL be
preserved: `id` is the shaper's glyph id and `advance` is the horizontal advance
scaled to the requested pixel size. A line with no renderable characters SHALL
yield no glyphs. The backend SHALL introduce no C dependency.

#### Scenario: A kerned pair advances less than the glyphs separately

- **WHEN** a pair the bundled face kerns is shaped
- **THEN** its total advance is strictly less than the sum of the two glyphs shaped alone, while an unkerned control pair is not reduced

#### Scenario: Simple text still shapes one glyph per character

- **WHEN** a two-letter unkerned string is shaped
- **THEN** it yields two glyphs with positive advances

#### Scenario: Provenance records the shaper and rasterizer

- **WHEN** provenance is produced
- **THEN** it records the bundled resolved family, the font hash, and a backend naming both the shaper and the rasterizer

#### Scenario: No C dependency

- **WHEN** the crate is built
- **THEN** the shaper is a pure-Rust dependency that links no C library

