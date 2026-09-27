# edit-clipboard Specification

## ADDED Requirements

### Requirement: Clipboard copy stores the selection bounding box

`pictura_render` SHALL expose `Clip { rect, rgba, mask }`, `copy_layer(doc,
path, selection)`, and `copy_merged(doc, composite, selection)`, where
`selection` is an optional document-sized coverage plane. A copy SHALL store only
the bounding box of the nonzero selection coverage intersected with the canvas
(and, for `copy_layer`, with the layer rectangle): `rgba` SHALL hold the source
pixels over that box and `mask` the selection coverage over it. Without a
selection the box SHALL be the layer rectangle ∩ canvas (`copy_layer`) or the
canvas (`copy_merged`) with full coverage. A copy SHALL return `None` for a
group, a layer without color channels, an empty box, or a box whose masked alpha
is zero everywhere.

#### Scenario: A small selection copies a small clip

- **WHEN** `copy_layer` copies a 2×2 selection from a 4×4 opaque layer
- **THEN** the clip's `rect` is that 2×2 box, `rgba` holds 16 bytes, and `mask` holds the selection coverage

#### Scenario: Copy without a selection takes the layer on the canvas

- **WHEN** `copy_layer` runs with no selection on a layer that extends past the canvas
- **THEN** the clip covers the layer rectangle clipped to the canvas with full coverage

#### Scenario: Nothing to copy refuses

- **WHEN** `copy_layer` runs on a group, with an all-zero selection, or over only transparent pixels
- **THEN** it returns `None`

#### Scenario: Copy Merged reads the composite

- **WHEN** `copy_merged` runs with a selection over a 4-plane document-sized composite
- **THEN** the clip holds the composite's RGBA over the selection box, and a composite that is not 4-plane document-sized refuses

### Requirement: Clear erases the selection and respects locks

`clear_layer(doc, path, selection)` SHALL scale each covered pixel's alpha by
`(255 - coverage) / 255`, clearing the whole layer without a selection. A layer
without an alpha channel (a Background) SHALL blend its color channels toward
white by coverage instead. It SHALL refuse (return false, unchanged) a group, a
layer without color channels, a pixel-locked layer, and a transparency-locked
layer that has an alpha channel, and SHALL return false when no pixel changes.

#### Scenario: Partial coverage clears partially

- **WHEN** `clear_layer` runs with coverage 255 and 128 over an opaque layer
- **THEN** those pixels' alpha become 0 and 127 and pixels outside the selection keep 255

#### Scenario: Locked layers refuse

- **WHEN** `clear_layer` targets a pixel-locked layer or a transparency-locked layer with alpha
- **THEN** it returns false and the layer is unchanged

#### Scenario: A no-op clear reports no change

- **WHEN** `clear_layer` runs over pixels that are already transparent
- **THEN** it returns false

### Requirement: Paste inserts the clip as a new layer

`paste_clip(doc, selection_path, clip, origin, mode, selection)` SHALL insert a
raster layer named by `next_layer_name(doc, "Layer")`, sized to the clip with its
top-left at `origin`, by the New Layer placement rule relative to
`selection_path`, with alpha `clip alpha × mask / 255`, and return its path.
`PasteMode::Into` SHALL attach a document-sized layer mask equal to the selection
(default 0); `PasteMode::Outside` SHALL attach its inverse (default 255). Into and
Outside SHALL refuse without a selection, and a clip whose buffers do not match
its rectangle SHALL refuse; a refusal SHALL return an empty path and leave the
document unchanged.

#### Scenario: Plain paste adds a layer above the target

- **WHEN** a clip is pasted with `PasteMode::Plain` at `(2, 3)` relative to path `"0"`
- **THEN** a new `Layer 1` sits at path `"1"` with rect origin `(2, 3)`, no mask, and alpha scaled by the clip mask

#### Scenario: Paste Into and Paste Outside mask by the selection

- **WHEN** a clip is pasted with `PasteMode::Into` and then `PasteMode::Outside` over the same selection
- **THEN** the first layer's mask equals the selection with default 0 and the second's is its inverse with default 255

#### Scenario: Cut then paste at the origin restores the pixels

- **WHEN** a selection is copied, cleared from its layer, and the clip is pasted at its own `rect` origin
- **THEN** the new layer reproduces the cut pixels at the same document position

### Requirement: Edit clipboard commands

The Edit menu SHALL expose Cut (`Ctrl+X`), Copy (`Ctrl+C`), Copy Merged
(`Shift+Ctrl+C`), Paste (`Ctrl+V`), Paste Special ▸ Paste Into (`Shift+Ctrl+V`),
Paste Special ▸ Paste Outside, Clear, and Purge ▸ Clipboard as implemented
commands. One clipboard SHALL be shared by every open document. Copy and Copy
Merged SHALL replace the clipboard and record no history, and a refused copy
SHALL keep the previous clipboard. Cut SHALL copy then clear the Layers panel's
current layer as one "Cut" state and store the clip only when the clear succeeds.
Clear SHALL record one "Clear" state. Paste SHALL add the clip centred on the
canvas view as one "Paste" state; Paste Into SHALL centre it on the selection
bounds and Paste Outside on the canvas view, each as one "Paste Into" / "Paste
Outside" state that deselects, and the pasted layer SHALL become the current
layer. Purge ▸ Clipboard SHALL empty the clipboard without history. Cut, Copy,
and Clear SHALL be disabled without a document or current layer; Copy Merged
without a document; Paste without a document or clipboard contents; Paste Into
and Paste Outside also without a selection; and Purge ▸ Clipboard without
clipboard contents.

#### Scenario: Cut, Paste Into, Clear, and Purge through the menu

- **WHEN** the `edit_clipboard` self-test copies, cuts, pastes into, clears, pastes, and purges through the command handlers on an 8×8 document
- **THEN** Copy records no state, each other mutating command records exactly one state with its label, the cut region shows the layer below until Paste Into restores it with a mask and no selection, and after Purge the clipboard is empty and Paste is disabled

#### Scenario: Paste is disabled with an empty clipboard

- **WHEN** the clipboard has been purged
- **THEN** the Paste command is disabled
