# Proposal: text-shaping-rustybuzz

## Why

The bundled text backend shapes a line by looking up one glyph per `char` and
using the glyph's standalone advance (`text_render.rs::shape_line`), so it never
kerns or substitutes — the module doc calls "no GSUB/GPOS shaping" a marked
ceiling (roadmap P3, "complex shaping/kerning"). The layout seam already expects
positioned glyphs with device-pixel advances (`ShapedGlyph`, `layout_lines`), so
the missing piece is a real shaper.

## What Changes

- **Shape each line with HarfBuzz's algorithm** via `rustybuzz` — a pure-Rust
  port of HarfBuzz (no C, no system library), the natural companion to the
  existing pure-Rust `fontdue` rasterizer. `shape_line` returns the shaped glyph
  ids and their device-pixel advances, so kerning and other default GPOS/GSUB
  substitutions are applied. The backward-compatible `ShapedGlyph { id, advance }`
  contract is unchanged.
- **Provenance records the shaper + rasterizer** as the backend.
- **A kerning test**: a pair the bundled Liberation Sans kerns has a smaller
  total advance than the two glyphs shaped separately.
- **No UI change, no PSD change.**
- **BREAKING**: none for the engine; the rendered advances change (kerning), so
  any exact-position text expectation would shift — none exists.

## Capabilities

### New Capabilities

- `text-shaping`: the bundled backend shapes lines with a HarfBuzz-compatible
  pure-Rust shaper, applying the font's kerning by default.

### Modified Capabilities

<!-- None. -->

## Impact

- `crates/pictura-render/Cargo.toml`: add `rustybuzz` (pure Rust, MIT).
- `crates/pictura-render/src/text_render.rs`: build a `rustybuzz::Face` from the
  bundled bytes, shape in `shape_line`, update the backend constant and docs.
- No app UI, no codec, no byte-layout change.
- Ceiling unchanged: axis-aligned placement, no transform/warp.
