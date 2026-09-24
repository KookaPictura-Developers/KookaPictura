# Proposal: rasterize-all-type

## Why

`Layer > Rasterize > All Layers` still only bakes fill-content layers;
`pictura_render::rasterize_all_fill_content` filters to `is_fill_content_layer`,
so type layers are skipped even though a single-layer command now rasterizes
them.

## What Changes

- **Rename and extend** the engine function to `rasterize_all_layers(doc)`:
  iterate every flattened layer and rasterize it through the fill-content path
  or, failing that, the bundled text path; return the count.
- **App**: `PictureView::rasterize_all_layers` calls the renamed function.
- **No new dependency.**
- **BREAKING**: the public `rasterize_all_fill_content` symbol is renamed (only
  internal callers/tests). No PSD change.

## Capabilities

### Modified Capabilities

- `text-rasterize-bundled`: All Layers also rasterizes type layers.

## Impact

- `crates/pictura-render`: `document_ops/.../rasterize.rs` + re-exports; tests.
- `crates/pictura-app`: one call site rename.
- No codec/core change.
