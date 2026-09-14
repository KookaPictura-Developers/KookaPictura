## Context

M5 added a `pictura-select` engine (`Selection`, `magic_wand`) but selections were
process-local: `PictureView` had no selection state, the app had no way to use a
selection as a mask, and `Document` had no place to store an alpha channel. The
PSD codec already parsed the composite image-data section but treated every plane
as a color plane. `docs/08-selection/save-and-load-selections.md` (SEL-012) and
`channel-based-masking.md` (SEL-013) define saved selections as grayscale
document-level alpha channels; `docs/05-layers/layer-masks.md` (LAY-004) defines a
layer mask as an alpha channel that confines a layer's contribution. This change
implements the parts of those specs needed to persist and use a selection.

## Goals / Non-Goals

**Goals:**
- Store document-level extra channels on `Document`, separate from layer channels.
- Round-trip those channels through PSD so a saved selection survives save/open,
  verified by an independent reader (psd-tools).
- Convert between `Selection` and `Channel` in both directions.
- Let a live selection mask an adjustment layer in the app, with UI controls.
- Prove confinement end to end in the headless self-test.

**Non-Goals:**
- Refine Edge, quick-selection brush, marquee tool, channel-thumbnail UI.
- Mask sense (`Masked Areas` / `Selected Areas`), spot channels, the 56-channel cap
  policy, cross-image Save/Load dialogs, and the Save/Load Selection dialogs.
  These stay in the SEL-012/SEL-013 specs, unbuilt.

## Decisions

**Document-level channel list, not per-layer.** SEL-012/SEL-013 describe alpha
channels as document-scoped (they have no owning layer), so `Document.channels:
Vec<Channel>` is a sibling of `Document.layers`, and layer records keep their own
`Layer.channels` for masks. Merging the two would make a layer mask indistinguishable
from a saved selection on export.

**Extra planes appended after the color planes in the image-data section.** The PSD
header channel count is bumped to `color_channels + extra.len()`, and the extra
planes are read/written after the color planes. This matches the on-disk planar
layout the reader already assumes and keeps composite/layer parsing untouched:
`split_planes` returns the composite (first `color_channels` planes) plus the extra
channels. The alternative — a separate image resource — would require
reimplementing the alternate-channel section and would not be visible to
psd-tools' header count.

**Selection → mask reuses `Selection.data` verbatim.** `selection_to_mask` builds a
full-frame `LayerMask` whose `data` is the selection coverage; the compositor
already multiplies layer contribution by mask coverage. No second mask format, no
coverage conversion. `Selection::to_channel`/`from_channel` likewise clone the same
byte plane, so a saved selection and a layer mask are the same representation.

**Byte-length validation at both edges.** `from_channel` and `write_psd` reject
length mismatches so a malformed mask/channel fails loudly instead of compositing
garbage.

## Risks / Trade-offs

- **Alpha convention.** psd-tools exposes the extra plane as the composite alpha;
  our document treats it as an independent channel. Accepted: no composite used by
  the app has < 4 planes, and the oracle only checks the raw bytes and count.
- **One extra channel per write.** The writer emits all `Document.channels`; the
  psd-tools oracle only exercises one. Multiple channels are covered by the Rust
  round-trip test.
- **Mask covers the whole frame.** `selection_to_mask` allocates a plane the size
  of the document even when the selection is small; acceptable at current sizes,
  revisit with tiling for PSB.
- **Self-test coupling.** The quadrant assertion depends on the `two_layers.psd`
  fixture geometry; a fixture change requires updating the coordinates.
