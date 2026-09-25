# Specs delta: text-subpixel-positioning

## ADDED Requirements

### Requirement: The bundled rasterizer honors subpixel offsets

The bundled glyph rasterizer SHALL rasterize a glyph at the fractional offset in
`RasterRequest.subpixel_x/subpixel_y`, so a glyph positioned at a fractional pen
coordinate renders at that subpixel phase rather than being rounded to a whole
pixel. The rasterizer SHALL remain pure Rust with no C dependency. Rendering a
glyph at an integer position SHALL place its mask within one pixel of the
position the previous rasterizer chose.

#### Scenario: A fractional offset changes the raster

- **WHEN** the same glyph is rasterized at `subpixel_x = 0.0` and at `subpixel_x = 0.5`
- **THEN** the two masks differ (coverage or placement), proving the offset reaches the rasterizer

#### Scenario: A fractional pen position is not rounded away

- **WHEN** a glyph is painted at a pen x of `12.5`
- **THEN** its mask is not identical to the same glyph painted at `12.0` or `13.0`

#### Scenario: Integer placement is preserved

- **WHEN** a glyph is rasterized at an integer position
- **THEN** its mask bounding box is within one pixel of the previous rasterizer's

#### Scenario: No C dependency

- **WHEN** the crate is built
- **THEN** the rasterizer is a pure-Rust dependency that links no C library
