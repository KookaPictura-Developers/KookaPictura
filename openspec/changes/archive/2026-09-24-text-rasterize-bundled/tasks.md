# Tasks: text-rasterize-bundled

## 1. Dependency and assets

- [x] 1.1 Add `fontdue` to `crates/pictura-render/Cargo.toml` (state why in the commit).
- [x] 1.2 Add `crates/pictura-render/assets/LiberationSans-Regular.ttf` (from `/usr/share/fonts/liberation/`) and `assets/LICENSES/LiberationSans-OFL.txt` (from `/usr/share/licenses/ttf-liberation/LICENSE`), with a short `assets/README.md` recording provenance.

## 2. Backend

- [x] 2.1 `crates/pictura-render/src/text_render.rs`: `BundledText` (`include_bytes!`, parse once), `shape_line(text, font_size, tracking) -> Vec<ShapedGlyph>` (design D2).
- [x] 2.2 `BundledRasterizer` implementing `pictura_core::Rasterizer` (design D3).
- [x] 2.3 `render_text_layer(doc, path) -> bool` (design D4) + `BundledText::provenance(requested)` (design D5).
- [x] 2.4 Register the module and re-export the public items from `pictura-render`.

## 3. Tests

- [x] 3.1 Rasterizer returns coverage of length `width*height` for a bundled glyph and `None`/no-panic for an absent index.
- [x] 3.2 `shape_line` maps a known string to the expected glyph count and non-zero advances.
- [x] 3.3 `render_text_layer` on a synthetic type layer yields non-empty alpha, clears `type_tool`, and removes `TySh`; a layer without a style returns false and is unchanged; provenance names the substitution.
- [x] 3.4 `cargo nextest run -p pictura-render`, `cargo fmt --all --check`, `cargo clippy -p pictura-render --all-targets -- -D warnings`, `openspec validate text-rasterize-bundled --strict`.
