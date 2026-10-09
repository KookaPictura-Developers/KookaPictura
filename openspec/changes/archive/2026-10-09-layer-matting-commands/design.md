# Design

## Context

`crates/pictura-render` already has a destructive per-layer colour-op pattern:
`filter.rs::apply_filter_region(&mut Layer, &Filter, Option<&LayerMask>, ...)`
crops the layer's colour planes, runs an op, and blends the result back through
the mask coverage (`orig + (result − orig)·cov/255`). The bridge mirrors this in
`impl_filters::apply_op_active_region`, which resolves the single active pixel
layer by index, refuses a hidden/pixel-locked/non-pixel target, and builds the
selection mask with `helpers_composite::selection_to_mask`.

The `Layer > Matting` leaves are registered in `command_tree.cpp` as path-derived
stubs (`leaf(...)`) and therefore disabled. The `CommandRegistry` lights a leaf
up once it has both a handler and an enabled-provider; frozen ids live in
`commands.h`.

`doc.layers` is bottom-first; the active layer path is `PictureViewRust.active_layer`,
which is `None` unless exactly one top-level pixel layer is selected.

## Goals / Non-Goals

**Goals:**
- Add the two missing pixel ops (un-matte, defringe) in the engine, following the
  existing crop/blend/coverage layout.
- Reuse `selection_to_mask` and the active-layer resolution helpers; do not
  invent a second target resolver or a second selection path.
- Keep each command one undo state via `recomposite()` + `record(label)`.

**Non-Goals:**
- No post-CS6 "Color Decontamination" and no selection-based "Refine Edge".
- No change to `compositing/layer-management` (that capability owns stack
  structure: arrange, merge, delete, stamp — not per-layer pixel cleanup).
- No GPU path; matting is a small CPU pass (the repository's determinism rule:
  CPU is the oracle, GPU is an accelerator).

## Decisions

- **Un-matte is a per-pixel solve, not a blur.** The layer is assumed to hold
  `P = C·a + B·(1−a)` (straight-alpha storage of a colour composited over
  background `B`). Solving gives `C = (P − B·(1−a)) / a`. For black `B=0` this is
  `P/a`; for white `B=255` it is `(P − 255·(1−a)) / a`. Pixels with `a = 0` are
  left untouched (undefined), and the result is clamped to `0..=255`. Alpha is
  never changed. Alternative: premultiply-then-divide — mathematically the same
  and rejected as needless indirection.
- **Defringe recolours the edge band from the nearest interior pixel.** A pixel
  is in the edge band when it is solid (`a > 0`) and within `width` (Chebyshev)
  of a transparent (`a = 0`) pixel. The band's colour is replaced by the colour
  of the nearest pixel that is solid and outside the band ("interior"), found
  with a multi-source BFS seeded from the interior in row-major order, so ties
  are deterministic. A band pixel with no interior pixel reachable keeps its
  colour. Alternative: sample the average of interior neighbours — rejected as
  not "the nearest interior colour" and order-sensitive.
- **The layer's own rect bounds the matte; the selection only gates writes.**
  Transparency outside the layer rect is not treated as an edge, so an
  otherwise-opaque layer with no alpha channel yields a no-op rather than a
  recoloured rect border.
- **A local `MattingError`** (`Locked`, `Empty`) rather than reusing
  `pictura_filters::FilterError`, because "empty layer" has no equivalent there
  and adding a variant would touch the filters crate. The bridge maps
  `MattingError` to `false` and leaves the document unchanged.
- **Defringe dialog is a hand-rolled `QDialog`** with one `QSpinBox` row, in the
  style of `fill_dialog.cpp`; `FilterParamControls` / `FilterPreviewDialog` are
  bound to `pictura_filters::Filter` parameter specs and carry a preview the
  matting op does not have, so reusing them would be more code, not less.

## Risks / Trade-offs

- [Defringe on a fully opaque layer with no `-1` channel] → no transparent
  pixels, so the command is a no-op success (one history state, unchanged
  pixels). Matches "clean the edge of a layer with transparency".
- [Un-matte amplifies noise where alpha is small] → inherent to the command;
  clamping keeps the result in range. CS6 behaves the same.
- [BFS cost on a huge layer] → O(pixels); width only bounds the band, not the
  search work. Acceptable for a menu command.

## Open Questions

None.
