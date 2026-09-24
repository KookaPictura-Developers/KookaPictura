# Tasks: text-shaping-offsets

## 1. Core seam

- [x] 1.1 `pictura-core/src/text_render.rs`: add `x_offset: f32, y_offset: f32` to `ShapedGlyph`; derive `Default`; add `ShapedGlyph::new(id, advance)` (offset `0.0`). Update the doc comment.
- [x] 1.2 `layout_lines`: place at `x = pen + device(x_offset)`, `y = baseline - device(y_offset)`; advance the pen by `device_advance` only. Update the `PlacedGlyph.y` doc.
- [x] 1.3 Update every `ShapedGlyph { .. }` literal in this file's tests to the new fields (or `new`), preserving the existing assertions; add the offset test (D5).

## 2. Renderer

- [x] 2.1 `shape_line`: set `x_offset`/`y_offset` from `pos.x_offset`/`pos.y_offset` scaled by `font_size / units_per_em`.
- [x] 2.2 `render_text_buffer`: use `glyph.y` (not `line.baseline`) for the paint baseline.
- [x] 2.3 Update any `ShapedGlyph` literals in `pictura-render` tests; add the offset-paint and zero-offset tests (D5).

## 3. Gates

- [x] 3.1 `cargo nextest run -p pictura-core -p pictura-render`, `cargo test -p pictura-render --doc`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `openspec validate text-shaping-offsets --strict`, `bash scripts/check-file-size.sh`.
- [x] 3.2 Confirm the kerning test and all render/composite text tests still pass (zero offsets must be byte-identical). If a combining-mark probe yields no offset in the bundled face, report it instead of asserting.
