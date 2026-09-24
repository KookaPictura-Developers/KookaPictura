# Design: rasterize-all-type

## Context

`rasterize_all_fill_content` flattens the layer paths, keeps the fill-content
ones, and calls `rasterize_fill_content`. `render_text_layer` materializes a type
layer and is a no-op for any other layer. The app's `rasterize_all_layers`
records one "Rasterize All Layers" state when the count is positive.

## Decisions

### D1. Rename and dispatch

`pub fn rasterize_all_layers(doc: &mut Document) -> usize` keeps the
flatten-then-filter shape but dispatches per path: `rasterize_fill_content(doc,
path) || render_text_layer(doc, path)`. Neither adds or removes layers, so the
path list stays valid. Rename the symbol (and its re-exports, the app call, and
the two tests) rather than keep two near-identical functions (`delete over
add`).

### D2. One pass, one state

The function still returns a count; the app records one history state when it is
positive, so a mixed fill+type document is one undo step.

## Risks / Trade-offs

- [Order] → the filter uses the same `flatten_rows` order as before; only the
  per-layer operation widens.
- [Count semantics] → the count now includes rasterized type layers; the
  existing fill-only test has no type layers, so it is unchanged.
