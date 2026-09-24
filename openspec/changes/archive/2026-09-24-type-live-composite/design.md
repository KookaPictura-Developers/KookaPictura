# Design: type-live-composite

## Context

`composite.rs::composite_layer` dispatches a non-group layer to
`decode_layer_fill` (fill), `composite_smart_source` (channel-less embedded
smart object), or `composite_pixels`. `composite_pixels` paints an opaque black
rect for a channel-less layer, so a proxy-less type layer currently renders
black. `composite_smart_source` is the precedent: it renders a source buffer
and blends it per pixel over the canvas-clipped region.

`text_render.rs::render_text_layer` already shapes, lays out, rasterizes, and
paints coverage into `0/1/2/-1` channel planes, then mutates the layer. The
compositor needs the same pixels without mutating the document.

## Goals / Non-Goals

**Goals:**

- A proxy-less type layer composites its live text.
- A type layer with a proxy is unchanged.
- The GPU declines the proxy-less case to the CPU oracle.
- Rust tests.

**Non-Goals:**

- Re-rendering a type layer that already has a proxy.
- Text editing, transform/warp, or per-run styling.
- Any app change.

## Decisions

### D1. Extract `render_text_buffer`

Move the shape/layout/paint body of `render_text_layer` into
`pub(crate) fn render_text_buffer(type_tool: &TypeTool, width: i32, height: i32)
-> Option<PixelBuffer>`, returning a straight-alpha RGBA buffer of `width ×
height` (layer-rect local), `None` when nothing was painted or the size is
non-positive. `render_text_layer` calls it and writes the four channel planes.
This is a pure refactor: `render_text_layer`'s output and tests are unchanged.

### D2. Composite branch mirrors smart source

In `composite_layer`, before `composite_pixels`, when the layer is not a group,
has `type_tool.is_some()`, and `channel(layer, 0).is_none()`, call a new
`composite_type_source(canvas, layer) -> bool`. It computes the canvas-clipped
region, calls `render_text_buffer(type_tool, layer.rect.width(),
layer.rect.height())`, and `blend_into`s each region pixel from the buffer's
corresponding local offset (1:1, no scaling). Returns false when the buffer is
`None`, so the existing path runs.

### D3. Proxy wins

The branch requires `channel(layer, 0).is_none()`, so a type layer read from a
PSD (which carries a raster proxy) is untouched; only a proxy-less layer renders
live.

### D4. GPU declines

Add `GpuError::UnsupportedText` and, in `check_supported`'s walk, return it for a
layer with `type_tool.is_some()` and no channel `0`, matching the existing
channel-less smart-object decline (`UnsupportedSmartObject`).

## Risks / Trade-offs

- [Refactor drift] → `render_text_layer`'s six existing tests must stay green
  unchanged.
- [Region indexing] → the buffer is layer-rect-local; the compositor maps
  `(canvas x - rect.left, canvas y - rect.top)`, the same mapping the pixel path
  uses.
- [UnsupportedText fallback] → the CPU composite is the oracle, so parity is
  preserved.
