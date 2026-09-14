## Context

M6 shipped `pictura-filters` as the destructive filter math over a planar 8-bit
`PixelBuffer`, with `Filter` and `FilterError` and no knowledge of documents,
layers, or Qt. M2 built `pictura-render` as the CPU compositor, where
`composite_pixels` already owns a layer's `rect`/channel layout, and M4 made
render depend on `pictura-adjust` so adjustment-layer payloads decode into
destructive ops. M5-C added a `selection_to_mask` helper in the app that turns a
`Selection` into a full-frame `LayerMask`, and the app now has a
`--self-test` that already proves a masked adjustment is confined.

What is missing is the bridge: a render-level `apply_filter` that applies a
finished filter to a pixel layer under a mask, and an app command that chooses
the layer and feeds it the selection mask. `docs/dev/m6c-filter-integration.md`
freezes the signature and semantics; this change records them as requirements.
The code does not exist yet: M6-C is forward-looking.

## Goals / Non-Goals

**Goals:**

- Freeze `pictura_render::apply_filter(layer, filter, mask)`: destructive
  application to color channels `0,1,2`, gated by an optional document-coordinate
  `LayerMask`, alpha (`-1`) untouched, metadata unchanged, errors not panics.
- Add `pictura-render` → `pictura-filters` so render can call the finished M6
  filters, mirroring the existing `pictura-render` → `pictura-adjust` edge.
- Wire a `PictureView::apply_filter(kind)` command that targets the topmost pixel
  layer, uses the M5-C selection mask, recomposites, and emits `changed`.
- Add the dock filter combo + "Apply Filter" button and a headless self-test that
  proves confinement to the selection.

**Non-Goals:**

- Non-destructive Smart Filters and the Smart Filter effect stack.
- Filter dialogs, preview panes, parameter editing, and Fade.
- Per-layer filter targeting (any explicit layer picker) and the Filter menu tree.
- Undo/history for a filter apply; the app records no history state yet.
- Filters on adjustment layers or group layers.
- 16-bit and 32-bit pixel math; M6-C is 8-bit only.
- GPU compute; the CPU render result stays the oracle.

## Decisions

**`apply_filter` lives in `pictura-render`, not `pictura-filters`.** Render
already owns the layer layout: `composite_pixels` reads `layer.rect` and the
channel planes, and `pictura-filters` deliberately knows nothing about
documents or layers (it takes a `PixelBuffer`). Putting the bridge in render
keeps the filter crate pure and the layer/rect/mask semantics next to the code
that already interprets them. Alternative rejected: a layer-aware `apply_filter`
inside `pictura-filters`, which would invert the dependency and drag
`pictura-core::Layer` into the math crate.

**New dependency `pictura-render` → `pictura-filters`.** Render calls
`Filter`/`FilterError` directly. This is the same shape as the existing
`pictura-render` → `pictura-adjust` dependency added for M4-B; both are
in-workspace and need no external crates.

**Extract to a planar 3-channel buffer, filter, then gate back.** The function
builds a `lw × lh × 3` `PixelBuffer` from the layer's id `0,1,2` channels, calls
`pictura_filters::apply`, then writes `round(orig + (filtered − orig) ×
coverage/255)` per pixel. This is the Fade-composite form from the filters
overview applied through the selection, and it is what lets the same filter run
whether or not a mask exists (`None` = coverage 255). Alternative rejected:
pre-intersecting the region with the mask and filtering a sub-rect, which would
lose the apron that neighborhood filters read at selection edges and add a
second region model.

**Reuse the M5-C selection→`LayerMask` path.** The app already converts a
`Selection` into a full-frame `LayerMask` in `selection_to_mask`, and the
compositor already consumes `LayerMask`. `apply_filter` takes
`Option<&LayerMask>`, so the command passes that same mask with no new mask
type or coverage conversion; `None` means no selection and a full-frame apply.

**Topmost pixel layer as the target.** The command scans `doc.layers`
bottom-first for the last entry with `adjustment.is_none() && !is_group`, which
matches the app's existing `layer_kind` classification and needs no selection
model for layers.

**Destructive now, Smart Filters later.** M6-C bakes the filter into the layer
pixels, so there is no effect node, no filter mask, and no re-edit path. Smart
Filters (a persistent effect stack on a Smart Object) are the documented CS6
model and are explicitly deferred; keeping the destructive path in render means
the later non-destructive path can be added as a separate evaluator without
changing this function's contract.

## Risks / Trade-offs

- **Selection-edge contamination.** Filters read outside the selection as an
  apron, so a neighborhood blur can smear unselected color into the selected
  edge. This matches the documented Gaussian/Box/Motion behavior; the gate only
  constrains writes. Accepted for M6-C and called out by filter, not hidden.
- **Closed-kernel approximation.** The filter output is only a behavioral
  approximation where no oracle exists (already documented by M6). M6-C adds no
  new approximation; it composes whatever the filter returns.
- **Channel-id assumptions.** `apply_filter` assumes ids `0,1,2` are color and
  `-1` is transparency. A layer with a different layout returns
  `FilterError::InvalidParams` rather than guessing; an explicit layout error is
  the safe failure at this boundary.
- **Full-layer allocation.** The extracted buffer and the filtered copy are
  `lw × lh × 3` bytes, in addition to the compositor's own canvas. Acceptable at
  current sizes; tiled filtering is the named ceiling for PSB-scale documents.
- **No undo.** A destructive apply is not reversible in the app yet; the brief
  defers undo, so a mis-apply is not recoverable until history lands. Accepted
  and bounded by the task scope.
- **Self-test coupling.** The confinement assertion depends on the layered-PSD
  fixture geometry, like the M5-C masked-adjustment check; a fixture change
  requires updating the self-test coordinates.
