# Proposal: text-qt-backend

## Why

The bundled pure-Rust rasterizer substitutes Liberation Sans for the document's
named family. The ADHD decision keeps a Qt `QFont`/`QPainter` backend as the
optional system-font-fidelity path. This adds it for the explicit `Rasterize
Type` command, where the user asked for a one-off bake and fidelity matters; the
bundled face stays the deterministic default for headless and live compositing.

## What Changes

- **Engine**: `pictura_render::materialize_text_rgba(doc, path, rgba)` replaces a
  layer's `0/1/2/-1` channels from a packed RGBA buffer of the layer-rect size,
  drops `TySh`, and clears `type_tool` (the tail of `render_text_layer`, split
  out so the app can supply pixels from any backend).
- **C++ Qt helper** `render_text_rgba(...)` in a new `render_text.{h,cpp}` (added
  to `CMakeLists.txt`): draws the text with `QFont`/`QPainter` into an RGBA
  `QImage` of the requested size, resolving the named family from the system.
- **App**: `PictureView::rasterize_type` renders through the Qt helper and
  materializes the result; it falls back to the bundled `render_text_layer` when
  Qt yields nothing.
- **Self-test**: a check rasterizes text through the Qt helper directly and
  asserts non-empty coverage.
- **No new dependency**; the C++ side already links Qt.

## Capabilities

### Modified Capabilities

- `text-rasterize-bundled`: the Rasterize Type command can render through a Qt
  font backend.

## Impact

- `crates/pictura-render`: `materialize_text_rgba` + test.
- `crates/pictura-app`: `render_text.{h,cpp}`, `CMakeLists.txt`, the cxx bridge
  declaration, `impl_layers_rasterize.rs`, and a self-test check.
- No codec/core change.
