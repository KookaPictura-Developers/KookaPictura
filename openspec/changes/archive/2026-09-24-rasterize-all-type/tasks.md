# Tasks: rasterize-all-type

## 1. Engine

- [x] 1.1 Rename `rasterize_all_fill_content` to `rasterize_all_layers` in `crates/pictura-render/src/document_ops/layer_ops/rasterize.rs`; dispatch each flattened path to `rasterize_fill_content` or `render_text_layer`. Update the re-exports in `layer_ops/mod.rs`, `document_ops/mod.rs`, and `lib.rs`.
- [x] 1.2 Update `crates/pictura-app/src/cxxqt_object/impl_layers_rasterize.rs` to call `rasterize_all_layers`.
- [x] 1.3 Update `crates/pictura-render/src/tests/rasterize.rs` call sites; add a test that a proxy-less type layer with a style plus a fill layer both rasterize (count 2), and the type layer's `type_tool` is cleared.

## 2. Gates

- [x] 2.1 `cargo nextest run -p pictura-render`, `cargo check -p pictura_app`, `cargo fmt --all --check`, `cargo clippy -p pictura-render -p pictura_app --all-targets -- -D warnings`, `openspec validate rasterize-all-type --strict`.
