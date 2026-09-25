# Proposal: text-backend-drop-fontdue

## Why

`text-subpixel-positioning` left `fontdue` in `Cargo.toml` for metrics only — a
marked `ponytail:` ceiling. But `fontdue` is no longer used for rasterization
(swash is) or shaping (rustybuzz is); it is kept for `glyph_count`,
`lookup_glyph_index`, and ascent. `rustybuzz` re-exports `ttf-parser`, which
provides all three (`number_of_glyphs`, `glyph_index`, `ascender`/`units_per_em`),
so the dependency is redundant and the backend can be one pure-Rust stack.

## What Changes

- **Read glyph count, char→glyph lookup, and ascent from `ttf-parser`** (reached
  through `rustybuzz::ttf_parser`, no new dependency).
- **Remove the `fontdue` dependency** and the temporary swash-vs-fontdue
  placement test; the `ponytail:` three-way-split ceiling is deleted.
- **No behavior change intended**: swash still rasterizes, rustybuzz still
  shapes, the metrics are the same font tables (`maxp`/`cmap`/`hhea`).
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `text-rasterize-bundled`: add a requirement that the bundled backend uses one
  pure-Rust font stack and no second rasterizer/metrics library.

## Impact

- `crates/pictura-render/Cargo.toml` + `Cargo.lock`: drop `fontdue`.
- `crates/pictura-render/src/text_render.rs`: `BundledText` keeps a
  `ttf_parser::Face` instead of a `fontdue::Font`; tests updated.
- No app UI, no codec/PSD change.
- Ceiling unchanged otherwise (transform/warp, Qt live-composite).
