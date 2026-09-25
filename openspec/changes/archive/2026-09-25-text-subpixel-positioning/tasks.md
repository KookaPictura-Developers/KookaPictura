# Tasks: text-subpixel-positioning

## 1. Dependency

- [x] 1.1 `crates/pictura-render/Cargo.toml`: `swash = { version = "0.2", default-features = false, features = ["std", "scale", "render"] }` (or the features `cargo add` resolves) with a one-line comment.

## 2. Rasterizer

- [x] 2.1 `BundledRasterizer`: build a `swash::FontRef` from `FONT_BYTES` and rasterize via `ScaleContext`/`Scaler`/`Render::new(&[Source::Outline])`, applying `render.offset(Vector::new(request.subpixel_x, request.subpixel_y))`.
- [x] 2.2 Map `zeno::Placement` to `GlyphMask { left, top, width, height, coverage }`; settle the `top` sign empirically (D4 placement check) so the existing `baseline - (height + top)` formula stays correct.
- [x] 2.3 Return `None` for an invalid glyph or an empty image, as today. Keep fontdue for `glyph_count`/`lookup_glyph_index`/ascent; add the `ponytail:` ceiling.

## 3. Painter

- [x] 3.1 `paint_layout`: floor the pen and pass the fractional parts as `subpixel_x`/`subpixel_y`; place the mask at the floored integer position. Apply vertical subpixel only if D4 verifies; else mark it.

## 4. Tests and gates

- [x] 4.1 Add the subpixel tests (D4): offset changes the raster; `12.5` differs from `12.0`/`13.0`; swash-vs-fontdue placement within 1 px (may be temporary).
- [x] 4.2 Confirm the existing text/raster/render/kerning/offset tests pass; migrate the kerning test's fontdue baseline only if the API changed.
- [x] 4.3 `cargo nextest run -p pictura-render -p pictura-core`, `cargo test -p pictura-render --doc`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `openspec validate text-subpixel-positioning --strict`, `bash scripts/check-file-size.sh`.
- [x] 4.4 If swash placement cannot be made to agree within 1 px, STOP, report, and revert the `swash` dependency; do not leave a broken rasterizer. (Not triggered: placement agreed exactly.)

