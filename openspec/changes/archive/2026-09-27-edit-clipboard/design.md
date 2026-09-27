# Design: edit-clipboard

## Context

photorust implements Copy as "selection bounds × coverage over the active
layer or composite", Paste as "a new layer, masked by the selection for
Into/Outside", and Clear as "alpha × (1 − coverage)". Kooka already has the
pieces those need: a document-sized selection coverage plane, layer-tree paths
with the New Layer placement rule (`insert_node`), layer locks, and the
full-document composite used by the display.

## Goals / Non-Goals

**Goals**

- The seven Edit clipboard commands plus `Purge ▸ Clipboard`, each one history
  step (Copy/Purge none), respecting locks, disabled without a document/layer.
- A clipboard sized to what was copied, not to the document.

**Non-Goals**

- OS clipboard interop, Paste in Place, vector-mask Paste Into, the background
  swatch.

## Decisions

**Clip model.** `Clip { rect, rgba, mask }`: `rect` is the document-space box,
`rgba` the unmasked source pixels over it, `mask` the selection coverage. The
box is the selection bounds ∩ layer rect ∩ canvas (Copy) or ∩ canvas (Copy
Merged); without a selection it is the layer rect ∩ canvas or the canvas. A
region that is empty, or whose masked alpha is all zero, refuses the copy and
keeps the previous clipboard (CS6: "the selected area is empty").

**Paste.** A new `Layer N` raster layer sized to the clip, alpha =
`clip alpha × mask / 255`, inserted by `insert_node` relative to the Layers
panel's current path. A plain paste centres on the canvas widget's centre in
document space; Paste Into centres on the selection bounds; Paste Outside on the
canvas centre. Into/Outside attach a document-sized layer mask from the
selection (inverted, default 255 for Outside) and move the selection into
`deselected_selection`, since it now lives on as the mask.

**Clear.** Alpha scales by `(255 − coverage) / 255`; without a selection the
whole layer clears. A Background (no alpha channel) blends toward white — the
app has no background swatch yet (`ponytail:`). A pixel lock refuses; a
transparency lock refuses on a layer with alpha. Clear reports whether a pixel
changed, so a no-op records no state. Cut is Copy then Clear under one "Cut"
state and stores the clip only when the clear succeeds.

**Bridge placement.** `cxxqt_object.rs` is at its allowlisted ceiling, so the
commands live in a separate `#[cxx_qt::bridge]` (`cxxqt_object/clipboard.rs`)
whose `extern "Rust"` free functions take `Pin<&mut PictureView>` through a cxx
type alias of the generated QObject. The module stays a child of
`cxxqt_object`, so it reuses `record`/`recomposite`/`clear_link_sets` without
widening their visibility.

**One clipboard.** A process-wide `Mutex<Option<Clip>>`, so a copy in one tab
pastes into another, as in CS6.

## Risks / Trade-offs

- Copy Merged renders the composite on the active backend (GPU when enabled),
  the same frame the canvas shows; golden checks run on CPU.
- A grayscale layer copies as replicated RGB and pastes back through the shared
  0/1/2/−1 channel layout (the existing `add_raster_layer_from_rgba` ceiling).
