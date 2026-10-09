# Design: paint-on-layer-mask

## Context

The paint (`Stroke`), fill (`bucket::fill`), and filter (`apply_filter`) engines
all edit a `Layer`'s colour channels. The raster mask is a single coverage plane
on `LayerMask`, sized to its own rectangle, so none of them can target it
directly. CS6's observable contract is in `docs/05-layers/layer-masks.md`:
clicking the mask thumbnail activates the mask (a border appears around it);
with the mask active, black hides, white reveals, gray is partial.

## Decisions

**The mask as a grayscale document.** `mask_document` builds a mask-sized
`Document` whose single layer holds the coverage replicated across R/G/B with a
transparency lock (opacity stays 255), and `write_mask_back` copies the edited
layer's red plane into the real mask, materialising a data-less mask from its
default colour first. The existing stroke, fill, and filter engines then run
unchanged, so the mask gains their full behaviour through one adapter instead of
three new kernels. Ceiling (`ponytail:`): a mask-sized four-plane copy per
operation; a dedicated single-plane mask stroke is the follow-up if huge masks
make the copy matter.

**Brush.** `begin_paint` with the mask target builds the document and starts an
ordinary `Stroke` on it. Each dab samples in mask-local coordinates, copies the
stroke's dirty region back into the real mask, and presents the document-space
region; the GPU and reduced-level previews are skipped because the exact CPU
stroke already paints the mask in place. `end_paint` finishes as usual and
records the normal "Brush"/"Pencil" label; `cancel_paint` restores from the
stroke's saved tiles and copies them back.

**Fill and filters.** `Edit ▸ Fill` builds the document, crops the active
selection to the mask rectangle, and fills the coverage with the fill colour's
luma (a pattern's per-pixel luma). A destructive filter runs on the document's
layer; a dialog preview snapshots the original coverage and re-filters from it,
a commit writes back and clears the snapshot, and cancel restores it.

**Grayscale conversion.** Rec.601 luma (0.299/0.587/0.114), the weights
`pictura-adjust` already uses. Adobe's mask conversion is closed, so this is an
approximation.

**Activation.** The bridge stores the target as a layer path; the Layers panel
sets it on a plain mask-thumbnail click and clears it on a layer-thumbnail click
or another row's selection. The delegate draws the border from a row role.

## Risks / Trade-offs

- Only Normal (and the opacity/flow accumulation) maps cleanly to coverage;
  Dissolve scatters the luma and Behind is a no-op on an opaque plane. These
  stay approximations, as the engine has no mask-specific blend modes.
- Fill and filter previews operate on the whole mask, not a viewport section
  (`ponytail:`).
- A vector mask cannot be the paint target; its thumbnail click still does
  nothing beyond selection, as CS6 paths are edited with the path tools.
