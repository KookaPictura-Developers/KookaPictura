# Tasks: type-live-composite

## 1. Refactor

- [x] 1.1 `crates/pictura-render/src/text_render.rs`: extract `pub(crate) fn render_text_buffer(type_tool: &TypeTool, width: i32, height: i32) -> Option<PixelBuffer>` (RGBA, straight alpha); `render_text_layer` reuses it (its output and tests unchanged).

## 2. Compositor

- [x] 2.1 `crates/pictura-render/src/composite.rs`: `composite_type_source(canvas, layer) -> bool` (region-clipped blend of the buffer, design D2) and call it in `composite_layer` for a non-group layer with `type_tool` and no channel `0`.

## 3. GPU

- [x] 3.1 `crates/pictura-render/src/gpu/mod.rs`: `GpuError::UnsupportedText` + `Display` arm; decline a type layer with no channel `0`.

## 4. Tests and gates

- [x] 4.1 A proxy-less type layer composites non-empty coverage (not uniform black); a type layer with a channel still composites from it (and the live path is not taken).
- [x] 4.2 An existing smart-object channel-less composite test still passes (no regression).
- [x] 4.3 `cargo nextest run -p pictura-render`, `cargo fmt --all --check`, `cargo clippy -p pictura-render --all-targets -- -D warnings`, `openspec validate type-live-composite --strict`.
