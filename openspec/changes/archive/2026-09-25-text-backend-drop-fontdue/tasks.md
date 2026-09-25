# Tasks: text-backend-drop-fontdue

## 1. Swap the metrics source

- [x] 1.1 In `crates/pictura-render/src/text_render.rs`, replace `BundledText.font: fontdue::Font` with `ttf: rustybuzz::ttf_parser::Face<'static>` parsed in `new()` (`Face::parse(FONT_BYTES, 0).ok()?`); `BundledRasterizer` holds `&'a ttf_parser::Face`.
- [x] 1.2 Guard: `request.glyph >= self.ttf.number_of_glyphs()`.
- [x] 1.3 `ascent(size)`: `self.ttf.ascender() as f32 * size / self.ttf.units_per_em() as f32`.
- [x] 1.4 Add `BundledText::glyph_for(&self, c: char) -> u16` using `self.ttf.glyph_index(c)`; migrate the tests that call `bundled.font.lookup_glyph_index`.

## 2. Tests

- [x] 2.1 Drop the `fontdue_standalone` closure and its assertion in the kerning test; keep `pair_total < alone(pair)` and the unkerned control.
- [x] 2.2 Delete `swash_placement_matches_fontdue_within_one_pixel` and the three-way-split `ponytail:` comment.
- [x] 2.3 Migrate remaining `bundled.font.*` test calls to `glyph_for`.

## 3. Dependency and gates

- [x] 3.1 Remove `fontdue = "0.9"` from `crates/pictura-render/Cargo.toml`; confirm `Cargo.lock` drops it and `rg fontdue crates` returns nothing.
- [x] 3.2 `cargo nextest run -p pictura-render -p pictura-core`, `cargo test -p pictura-render --doc`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `openspec validate text-backend-drop-fontdue --strict`, `bash scripts/check-file-size.sh`.
- [x] 3.3 If any render/kerning/subpixel assertion fails, STOP and report rather than weakening it.
