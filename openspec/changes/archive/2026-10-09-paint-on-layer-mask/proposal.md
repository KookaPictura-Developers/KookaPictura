# Proposal

## Why

Issue #108's last open item: with a raster layer mask present, CS6 lets the user
click the mask thumbnail to make the mask the edit target and then paint, fill,
or filter its coverage (black hides the layer, white reveals it, gray is
partial). Kooka reads and authors masks, but a plain mask-thumbnail click is
consumed with no effect and every paint, fill, and filter path writes the layer's
pixels.

## What Changes

- New `pictura_render` mask-edit adapter: `mask_document(doc, path)` exposes a
  raster mask as a standalone mask-sized grayscale document (its coverage in the
  colour channels, opacity locked), and `write_mask_back` copies an edited
  document's colour plane into the real mask.
- Bridge: a per-view mask edit target (`set`/`clear`/`read`); a plain click on a
  mask thumbnail activates it and a click on the layer thumbnail (or another row)
  deactivates it.
- Brush strokes, `Edit ▸ Fill`, and destructive filters route through the adapter
  when the active layer's mask is the target, each recording its normal one-step
  history label. The foreground/fill colour becomes Rec.601 luma.
- The Layers panel delegate draws the active-mask border.

## Capabilities

### Modified Capabilities

- `compositing/layer-masks`: the mask edit target and its paint/fill/filter
  coverage semantics.
- `ui/layers-panel`: the mask activation click and the active-mask border.

## Impact

- `pictura-render` (new `mask_edit` module), `pictura-app` (bridge state and the
  paint/fill/filter routing, Layers panel click and delegate). No new
  dependency.
