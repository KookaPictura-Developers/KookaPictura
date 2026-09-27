# Design: perspective-crop-and-slice

## Decisions

**Homography.** Kooka already solves projective maps for Free Transform
(`solve_homography`, with `bilinear` sampling), so the port reuses them instead
of photorust's separate solver. The output size is photorust's: the longer of
each pair of opposite edges (clamped to 1–30 000), keeping the detail nearest
the camera.

**What warps.** Every pixel layer's channels and layer mask are resampled onto
the new canvas (mask default colour outside the source, alpha 0 outside for a
layer with alpha, white for a Background — `ponytail:` no background swatch).
Channel-less layers (adjustments, fills) are left alone; document extra channels
warp too; the selection is dropped, as the existing Crop does.

**Refusals.** A pixel warp would leave live type, smart objects, and vector
masks out of register, and retained 16/32-bit planes cannot be warped yet, so
the command refuses with a reason (`perspective_crop_refusal`) instead of
silently rasterizing or discarding data. A degenerate quad is refused and the
quad stays staged.

**Slices on the document.** User slices are document state, so they live on
`Document::slices` and ride the existing history snapshot for undo. Auto slices
are derived on every query (photorust's grid-and-merge). `ponytail:` slices are
not yet written to or read from the PSD slices resource (1050), which stays
preserved verbatim.

**Interaction.** Perspective Crop: drag a box; drag a corner (8 screen px) to
pull it off-square; drag inside to move it; Enter or a double-click (a second
press on the same spot inside, never on a corner) commits; Escape discards.
Slice: drag to add; a click adds nothing. The slice overlay is shown while the
Slice tool is active and re-read on every frame refresh through
`ToolHandler::onDocumentRefreshed`.

## Non-Goals

- Slice Select (move/resize/delete slices), Save for Web export, slice options.
- Perspective Crop options (W/H/resolution, Front Image), live-type preservation.
