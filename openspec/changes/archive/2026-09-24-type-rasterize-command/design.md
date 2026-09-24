# Design: type-rasterize-command

## Context

`PictureView::rasterize_layer` (`crates/pictura-app/src/cxxqt_object/impl_layers_rasterize.rs`)
calls `rasterize_path`, which calls `pictura_render::rasterize_fill_content`.
`pictura_render::render_text_layer` (archived `text-rasterize-bundled`)
materializes a type layer and returns `false` without mutating for any other
layer.

## Decisions

### D1. Dispatch in `rasterize_layer`, leave the fill command alone

`rasterize_layer` tries `render_text_layer` first; since it is a no-op for a
non-type layer, the existing `rasterize_fill_content` path is unchanged for fill
layers. The shared `rasterize_path` is left as the fill-only helper so
`Rasterize Fill Content` never touches a type layer. The success label is
`Rasterize Type` when the type path rendered, else `Rasterize Layer`, so the
History panel reads correctly.

### D2. No test hook needed for the thin branch

The non-trivial work is `render_text_layer`, already unit-tested in
`pictura-render`. The app change is a two-branch dispatch, so it is covered by
the app crate compiling and the engine tests.

## Risks / Trade-offs

- [Menu gating] → the C++ `Layer > Rasterize > Layer` action is not
  fill-content gated, so a type layer reaches the new branch; a fill layer
  reaches the old one.
- [Rasterize All Layers] → still fill-content only; extending it to type layers
  is a follow-up.
