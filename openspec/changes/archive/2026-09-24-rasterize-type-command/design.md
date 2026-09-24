# Design: rasterize-type-command

## Context

`command_tree.cpp` registers `Layer > Rasterize > Type` as a `leaf(...)`
(disabled, no command id). `frame_menus.cpp` wires `LayerRasterizeLayer` to
`PictureView::rasterize_layer`, which already dispatches type layers to
`render_text_layer`. The C++ self-test (check 238) asserts that
`layer.rasterize.type` is among the kind-less commands that stay disabled.

## Decisions

### D1. Enable the leaf and gate it on a type layer

Replace the disabled leaf with `registry.add(command_ids::LayerRasterizeType,
..., true)` and register an enabled provider that is true only when the current
layer is a type layer. Add `PictureView::layer_is_type(path) -> bool`
(`resolve_path(doc, path).is_some_and(Layer::is_type)`), mirroring
`layer_is_fill_content`. The handler calls `rasterize_type` and refreshes.

### D2. `rasterize_type` is type-only

`PictureView::rasterize_type(path) -> bool` calls
`pictura_render::render_text_layer(doc, path)`, and on success clears link sets,
recomposites, and records `Rasterize Type`. It never touches fill content, so
the two commands are distinct.

### D3. Self-test

Check 238 drops `layer.rasterize.type` from the kind-less list (it is now
enabled but provider-disabled on a plain pixel layer) and adds
`rasterize_type("0")` returning false with no history. The positive path stays
covered by the `pictura-render` unit tests, since the self-test has no
type-layer fixture.

## Risks / Trade-offs

- [Enablement] → on a plain pixel layer the provider keeps Type disabled, so the
  existing "disabled" assertion would still hold; the check is tightened to test
  refusal instead of absence.
- [No positive self-test] → acceptable; the renderer is unit-tested and adding a
  type-layer fixture is a separate task.
