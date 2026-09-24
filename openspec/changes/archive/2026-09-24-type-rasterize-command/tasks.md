# Tasks: type-rasterize-command

## 1. Dispatch

- [x] 1.1 In `crates/pictura-app/src/cxxqt_object/impl_layers_rasterize.rs`, make `rasterize_layer` try `pictura_render::render_text_layer` first (label `Rasterize Type`), else the existing `rasterize_fill_content` path (label `Rasterize Layer`); leave `rasterize_fill_content`/`rasterize_path` fill-only. Update the method doc comment.
- [x] 1.2 `cargo check -p pictura-app` / `cargo clippy -p pictura-app --all-targets -- -D warnings`; `cargo nextest run -p pictura-render text_render`; `openspec validate type-rasterize-command --strict`.
