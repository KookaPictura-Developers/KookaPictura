# text-shaping Specification

## Purpose
A bundled HarfBuzz-compatible shaper that shapes lines for the text backend.
## Requirements
### Requirement: The bundled backend shapes lines with a HarfBuzz-compatible shaper

`pictura-render`'s bundled text backend SHALL shape each line with a
HarfBuzz-compatible shaping algorithm implemented in pure Rust, so the glyph
sequence and the device-pixel advances reflect the font's default `kern`/GPOS
and GSUB substitutions. `pictura_core::ShapedGlyph` SHALL carry the shaper's
glyph id, horizontal advance, and x/y offsets, each scaled to the requested pixel
size; `layout_lines` SHALL place a glyph at `pen + x_offset` and
`baseline - y_offset` and SHALL advance the pen by the glyph's advance alone, so
the marked glyph positions are honoured without changing line width. A line with
no renderable characters SHALL yield no glyphs. The backend SHALL introduce no C
dependency.

#### Scenario: A kerned pair advances less than the glyphs separately

- **WHEN** a pair the bundled face kerns is shaped
- **THEN** its total advance is strictly less than the sum of the two glyphs shaped alone, while an unkerned control pair is not reduced

#### Scenario: An offset shifts a glyph without moving the pen

- **WHEN** a shaped glyph carries a non-zero x-offset or y-offset
- **THEN** the glyph is placed at `pen + x_offset` and `baseline - y_offset`, and the following glyph's pen is unchanged

#### Scenario: Simple text still shapes one glyph per character

- **WHEN** a two-letter unkerned string is shaped
- **THEN** it yields two glyphs with positive advances

#### Scenario: Provenance records the shaper and rasterizer

- **WHEN** provenance is produced
- **THEN** it records the bundled resolved family, the font hash, and a backend naming both the shaper and the rasterizer

#### Scenario: No C dependency

- **WHEN** the crate is built
- **THEN** the shaper is a pure-Rust dependency that links no C library

