# Proposal: rasterize-type-command

## Why

`Layer > Rasterize > Type` is a permanently greyed-out leaf even though the
single-layer command already rasterizes a type layer; the renderer is reachable
only through `Rasterize Layer`. The Adobe menu has a dedicated Type entry.

## What Changes

- **Enable the `Layer > Rasterize > Type` command** (`layer.rasterize.type`),
  gated to a selected type layer.
- **New `PictureView::rasterize_type`** that materializes a type layer through
  the bundled text backend and records one `Rasterize Type` state; a refusal
  leaves the document unchanged.
- **New `PictureView::layer_is_type`** predicate for menu enablement.
- **Self-test** check 238 stops treating `layer.rasterize.type` as a kind-less
  disabled leaf and asserts it refuses a plain pixel layer without history.
- **No new dependency; no engine change.**

## Capabilities

### Modified Capabilities

- `text-rasterize-bundled`: a dedicated Rasterize Type command materializes a
  type layer.

## Impact

- `crates/pictura-app`: `impl_layers_rasterize.rs` (two methods),
  `commands.h`, `command_tree.cpp`, `frame_menus.cpp`, and the check in
  `selftest_layers_controls.cpp`.
- No codec/core/render change.
