# Tasks: text-shaping-rustybuzz

## 1. Dependency

- [x] 1.1 Add `rustybuzz = "0.20"` to `crates/pictura-render/Cargo.toml` with a one-line comment naming the pure-Rust HarfBuzz port.

## 2. Shaping

- [x] 2.1 `BundledText`: add `face: rustybuzz::Face<'static>` built in `new()` via `rustybuzz::Face::from_slice(FONT_BYTES, 0)`; `None` if either parse fails.
- [x] 2.2 Rewrite `shape_line` to shape via `rustybuzz` (UnicodeBuffer, LTR, `guess_segment_properties`) and return `ShapedGlyph { id: u16, advance: px }`, scaling `x_advance` by `font_size / units_per_em` and dropping glyph ids above `u16::MAX`.
- [x] 2.3 Update the module doc (drop the "no GSUB/GPOS" ceiling) and `BACKEND`/`BACKEND_VERSION` to name shaper + rasterizer; update the `composite_type_source` "no kerning" note.

## 3. Tests

- [x] 3.1 Update `provenance_records_the_bundled_substitution` for the new backend string.
- [x] 3.2 Add a kerning test: a kerned pair's shaped total advance is strictly less than the two glyphs shaped alone and than their standalone `fontdue` advances; an unkerned control pair is not reduced. Discover the pair empirically; if none kerns, STOP and report.
- [x] 3.3 Confirm existing shape/render/composite text tests pass unchanged.

## 4. Gates

- [x] 4.1 `cargo nextest run -p pictura-render -p pictura-core`, `cargo test -p pictura-render --doc`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `openspec validate text-shaping-rustybuzz --strict`, `bash scripts/check-file-size.sh`.
- [x] 4.2 Confirm no exact text-pixel golden broke; if one did, report it.
