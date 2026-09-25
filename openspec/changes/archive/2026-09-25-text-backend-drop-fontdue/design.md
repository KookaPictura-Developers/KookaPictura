# Design: text-backend-drop-fontdue

## Context

`crates/pictura-render/src/text_render.rs` now uses:
- `rustybuzz` for shaping (`shape_line`),
- `swash` for rasterization (`BundledRasterizer::rasterize`),
- `fontdue` only for `glyph_count` (rasterize guard), `lookup_glyph_index`
  (tests), and `horizontal_line_metrics().ascent` (`BundledText::ascent`), plus
  the kerning test's standalone-advance baseline.

`rustybuzz` re-exports `ttf_parser` (`pub use ttf_parser;`). `ttf_parser::Face`
offers `number_of_glyphs() -> u16`, `glyph_index(char) -> Option<GlyphId>`,
`ascender() -> i16`, `units_per_em() -> u16`, and `glyph_hor_advance(GlyphId)`
(for the kerning baseline if wanted).

## Goals / Non-Goals

**Goals:**

- Delete `fontdue`; keep shaping/raster metrics on the pure-Rust stack.
- Behavior-identical text (same font tables).

**Non-Goals:**

- Changing the shaper/rasterizer, layout, or output.
- Folding `swash` metrics in where `ttf-parser` already answers.

## Decisions

### D1. One `ttf_parser::Face` in `BundledText`

`BundledText` replaces `font: fontdue::Font` with `ttf: ttf_parser::Face<'static>`
parsed in `new()` via `ttf_parser::Face::parse(FONT_BYTES, 0).ok()?`
(`rustybuzz::ttf_parser`). `BundledRasterizer` keeps a `&ttf_parser::Face` for
the guard.

### D2. Replacements

- Guard: `if request.glyph >= self.ttf.number_of_glyphs() { return None; }`.
- `lookup_glyph_index(c)` (tests): `self.ttf.glyph_index(c).map(|g| g.0)`; add a
  small helper `BundledText::glyph_for(char) -> u16` for the tests.
- `ascent(size)`: `self.ttf.ascender() as f32 * size / self.ttf.units_per_em() as f32`.
- Kerning test: drop the `fontdue_standalone` closure/assertion; keep the
  rustybuzz-based `pair_total < alone(pair)` and the unkerned control.
- Delete `swash_placement_matches_fontdue_within_one_pixel` (its purpose was the
  migration check; fontdue is gone) and the `ponytail:` three-way-split comment.

### D3. Dependency

Remove `fontdue = "0.9"` from `crates/pictura-render/Cargo.toml`; `cargo`
updates `Cargo.lock`.

## Risks / Trade-offs

- [Ascent from a different table accessor] → `ttf-parser` `ascender()` reads
  `hhea`/`OS/2` exactly as fontdue did; if a test asserts absolute placement it
  is relative here, so it is safe. Verify the render tests still pass.
- [Test churn] → several tests reference `bundled.font`; the subagent migrates
  them to the helper.
