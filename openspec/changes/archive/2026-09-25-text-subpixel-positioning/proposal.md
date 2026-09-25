# Proposal: text-subpixel-positioning

## Why

The layout seam carries `RasterRequest.subpixel_x/subpixel_y` and
`PlacedGlyph` positions are `f32`, but `BundledRasterizer` ignores the offsets
and `render_text_buffer` rounds every glyph to a whole pixel. The bundled
`fontdue` rasterizer cannot do better: its `rasterize_indexed_subpixel` renders
LCD subpixel **anti-aliasing** at 3× width (phase fixed at 0), and
`GlyphRasterConfig { glyph_index, px, font_hash }` has no offset field. Text with
fractional advances (kerning, tracking) therefore shifts unevenly and loses
positioning accuracy. The roadmap's "subpixel placement" gap is exactly this.

## What Changes

- **Rasterize the bundled glyphs with `swash`** (pure Rust, MIT, RazrFalcon),
  whose `Scaler`/`Render::offset(Vector)` honors a fractional `(x, y)` offset,
  and feed `RasterRequest.subpixel_x/subpixel_y` from the layout.
- **`render_text_buffer` passes the fractional part** of each glyph's x (and y)
  and places the mask at the floored integer position, so a glyph at `x = 12.5`
  gets the half-pixel phase instead of being rounded to 13.
- **`fontdue` is retained for metrics only** (ascent, glyph count, char
  lookup) — a `ponytail:` ceiling to fold into swash later. Shaping stays
  `rustybuzz`; provenance backend becomes `rustybuzz/swash`.
- **BREAKING**: none at the seam; rendered pixels change (a different, finer
  rasterizer/AA), but there is no text golden to preserve.

## Capabilities

### New Capabilities

- `text-subpixel-positioning`: the bundled rasterizer honors fractional offsets
  and the renderer passes the layout's fractional pen positions through.

### Modified Capabilities

<!-- None. -->

## Impact

- `crates/pictura-render/Cargo.toml`: add `swash` (features `scale`, `render`).
- `crates/pictura-render/src/text_render.rs`: swash-based `BundledRasterizer`
  and the fractional pen in `render_text_buffer`.
- No app UI, no codec/PSD change.
- Ceiling: fontdue remains for metrics; vertical subpixel is applied if the
  placement is verified, else marked.
