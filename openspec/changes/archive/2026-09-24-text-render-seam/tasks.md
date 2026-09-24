# Tasks: text-render-seam

## 1. Model

- [x] 1.1 `pictura-core::text_render`: `ShapedGlyph { id: u16, advance: f32 }`, `TextAlign { Left, Center, Right }` + `from_justification(u8)`, `LayoutParams`, `PlacedGlyph`, `LayoutLine`, `TextLayout`.
- [x] 1.2 `layout_lines(lines: &[Vec<ShapedGlyph>], params: &LayoutParams) -> TextLayout` per design D2 (advance scaling, tracking, leading, alignment, empty input).
- [x] 1.3 `RasterRequest`, `GlyphMask`, `trait Rasterizer`.
- [x] 1.4 `FontPolicy`, `TextProvenance { requested_family, resolved_family, font_hash: [u8;32], backend, backend_version }` + `new` + stable record form (`Display` or `fn record`).
- [x] 1.5 Export all of the above from `pictura-core`.

## 2. Tests

- [x] 2.1 Layout goldens: advance accumulation; tracking; leading; center/right alignment inside a wrap width; unknown justification → Left; empty/blank lines.
- [x] 2.2 A `TestRasterizer` implementing `Rasterizer` returning a synthetic mask; assert coverage length `width*height` and `None` for an unknown glyph.
- [x] 2.3 Provenance: substitution record exposes requested vs resolved; the three policies are distinct.

## 3. Gates

- [x] 3.1 `cargo nextest run -p pictura-core`, `cargo fmt --all --check`, `cargo clippy -p pictura-core --all-targets -- -D warnings`, `openspec validate text-render-seam --strict`.
