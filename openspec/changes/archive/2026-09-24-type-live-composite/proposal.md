# Proposal: type-live-composite

## Why

`text-rasterize-bundled` can render a type layer, but the compositor still
treats a proxy-less type layer (a `TypeTool` with no colour channel) as a bare
channel-less layer and paints an opaque black rect over its bounds. Live text
should render from the model when there is no stored raster proxy, exactly as a
proxy-less smart object renders from its embedded source.

## What Changes

- **Factor the renderer**: `pictura-render::text_render` gains an internal
  `render_text_buffer(type_tool, width, height) -> Option<PixelBuffer>` (RGBA,
  the layer-rect size); `render_text_layer` reuses it.
- **Compositor renders proxy-less type live**: `composite_layer` gains a branch
  that, for a non-group layer carrying a `TypeTool` and no channel `0`, renders
  the text buffer and composites it over the canvas-clipped region with the
  layer's opacity/mask/blend — mirroring `composite_smart_source`.
- **A stored proxy still wins**: a type layer with a colour channel composites
  from its raster, unchanged, so existing documents are byte-identical.
- **GPU declines it**: `check_supported` returns a new `GpuError::UnsupportedText`
  for a type layer with no channel `0`, so the CPU oracle renders it.
- **BREAKING**: none.

## Capabilities

### New Capabilities

- `type-live-composite`: live rendering of a proxy-less type layer in the CPU
  compositor.

## Impact

- `crates/pictura-render`: `text_render.rs` refactor, `composite.rs` branch,
  `gpu/mod.rs` decline + `Display` arm; unit tests.
- No dependency, no codec/core change, no app change.
- Ceiling: axis-aligned placement at the layer rect (no transform/rotation),
  first-run style, no kerning — inherited from the bundled renderer.
