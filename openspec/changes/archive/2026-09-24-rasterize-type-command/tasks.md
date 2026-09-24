# Tasks: rasterize-type-command

## 1. Rust app

- [x] 1.1 `impl_layers_rasterize.rs`: add `pub fn rasterize_type(path) -> bool` (render_text_layer + recomposite + record "Rasterize Type") and `pub fn layer_is_type(path) -> bool`.

## 2. C++

- [x] 2.1 `commands.h`: add `LayerRasterizeType = "layer.rasterize.type"`.
- [x] 2.2 `command_tree.cpp`: register the Type entry with the command id and enabled default (replace the disabled leaf).
- [x] 2.3 `frame_menus.cpp`: handler + enabled provider gated on `layer_is_type(currentPath)`.
- [x] 2.4 `selftest_layers_controls.cpp`: check 238 drops `layer.rasterize.type` from the kind-less list and asserts `rasterize_type` refuses a plain layer without history.

## 3. Gates

- [x] 3.1 `cargo check -p pictura_app`, `cargo clippy -p pictura_app --all-targets -- -D warnings`, CMake build + `./build/pictura --headless --self-test`, `openspec validate rasterize-type-command --strict`.
