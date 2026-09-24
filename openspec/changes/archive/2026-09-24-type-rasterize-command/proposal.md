# Proposal: type-rasterize-command

## Why

`text-rasterize-bundled` shipped `render_text_layer` but nothing calls it, so a
type layer still cannot be rasterized from the UI. C++ `Layer > Rasterize >
Layer` already invokes `PictureView::rasterize_layer`, which only handles
fill-content layers.

## What Changes

- **`PictureView::rasterize_layer` handles type layers**: it tries
  `render_text_layer` first (which is a no-op for non-type layers), then the
  existing fill-content path, and records the state as `Rasterize Type` when a
  type layer was rendered.
- **No C++ change**: the existing `Layer > Rasterize > Layer` menu command
  becomes functional for type layers.
- **No new dependency.**
- **BREAKING**: none.

## Capabilities

### Modified Capabilities

- `text-rasterize-bundled`: the Rasterize Layer command rasterizes a type layer
  through the bundled backend.

## Impact

- `crates/pictura-app`: `impl_layers_rasterize.rs` dispatch.
- Tests: the engine renderer is already covered; the dispatch is a thin branch
  over it.
