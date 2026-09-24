# Proposal: text-rasterize-bundled

## Why

Roadmap P3: `text-render-seam` shipped the deterministic layout and the POD
`Rasterizer` port, but no backend, so text still cannot reach pixels. The ADHD
decision chose a pure-Rust rasterizer over a bundled metric-compatible fallback
font as the deterministic default.

## What Changes

- **`fontdue` backend**: add `fontdue` to `pictura-render` and bundle
  `LiberationSans-Regular.ttf` (SIL OFL 1.1, license included) via
  `include_bytes!`.
- **Shape + rasterize**: shape a line (char → glyph index + advance) with the
  bundled font and implement `pictura_core::Rasterizer` over
  `fontdue::Font::rasterize_indexed`, returning a `GlyphMask`.
- **Materialize a type layer**: `render_text_layer(doc, path)` shapes and lays
  out the layer's `TypeTool` text via `layout_lines`, paints the glyph coverage
  in the style's fill colour into the layer's `0/1/2/-1` channels, and drops the
  `TySh`/`type_tool` so the layer becomes raster.
- **Provenance**: the render records the requested family, the bundled
  resolved family, the bundled font's digest, and the backend/version.
- **No Qt, no UI** in this change; the app command and the Qt backend follow.
- **BREAKING**: none (a new dependency and a new asset only).

## Capabilities

### New Capabilities

- `text-rasterize-bundled`: a bundled-font, pure-Rust glyph rasterizer and a
  type-layer materializer.

## Impact

- `crates/pictura-render`: `fontdue` dependency, the bundled font + license
  assets, a `text_render` module, and Rust tests.
- No codec/core/app change beyond consuming the shipped seam types.
- Ceiling: no complex shaping (kerning/ligatures), no warp/rotation, and the
  bundled face substitutes for the named family (recorded as provenance).
