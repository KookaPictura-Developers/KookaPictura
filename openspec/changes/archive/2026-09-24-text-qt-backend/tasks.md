# Tasks: text-qt-backend

## 1. Engine

- [x] 1.1 `crates/pictura-render/src/text_render.rs`: add `pub fn materialize_text_rgba(doc, path, rgba) -> bool` (design D1); refactor `render_text_layer` to use it; re-export.
- [x] 1.2 Test: a wrong-size buffer and a missing layer return false without mutation; a correct buffer sets the channels, drops `TySh`, clears `type_tool`.

## 2. C++ Qt helper

- [x] 2.1 `crates/pictura-app/cpp/render_text.h` + `render_text.cpp` (design D2); add both to `CMakeLists.txt`.
- [x] 2.2 Declare `render_text_rgba` in the cxx bridge `unsafe extern "C++"` block in `cxxqt_object.rs`.

## 3. App

- [x] 3.1 `impl_layers_rasterize.rs::rasterize_type`: read the layer's type tool/style/rect, call `render_text_rgba`, and `materialize_text_rgba` on success, else fall back to `render_text_layer`; record one state.
- [x] 3.2 Self-test: a new check (next free code) calls the Qt helper and asserts non-empty RGBA.

## 4. Gates

- [x] 4.1 `cargo check -p pictura_app`, `cargo fmt --all --check`, `cargo clippy -p pictura-render -p pictura_app --all-targets -- -D warnings`, CMake build + `./build/pictura --headless --self-test`, `openspec validate text-qt-backend --strict`.
