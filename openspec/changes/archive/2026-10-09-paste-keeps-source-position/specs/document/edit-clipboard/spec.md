## MODIFIED Requirements

### Requirement: Edit clipboard commands

The Edit menu SHALL expose Cut (`Ctrl+X`), Copy (`Ctrl+C`), Copy Merged
(`Shift+Ctrl+C`), Paste (`Ctrl+V`), Paste Special ▸ Paste Into (`Shift+Ctrl+V`),
Paste Special ▸ Paste Outside, Clear, and Purge ▸ Clipboard as implemented
commands. One clipboard SHALL be shared by every open document. Copy and Copy
Merged SHALL replace the clipboard and record no history, and a refused copy
SHALL keep the previous clipboard. Cut SHALL copy then clear the Layers panel's
current layer as one "Cut" state and store the clip only when the clear succeeds.
Clear SHALL record one "Clear" state. Paste SHALL add the clip with its top-left
at the clip's own document rect as one "Paste" state; Paste Into SHALL centre it
on the selection bounds and Paste Outside on the canvas view, each as one "Paste
Into" / "Paste Outside" state. Every successful paste SHALL drop the active
marquee by moving the selection into the reselect store, and the pasted layer
SHALL become the current layer. Purge ▸ Clipboard SHALL empty the clipboard
without history. Cut, Copy, and Clear SHALL be disabled without a document or
current layer; Copy Merged without a document; Paste without a document or
clipboard contents; Paste Into and Paste Outside also without a selection; and
Purge ▸ Clipboard without clipboard contents.

#### Scenario: Cut, Paste Into, Clear, and Purge through the menu

- **WHEN** the `tst_edit_clipboard` Qt Test copies, cuts, pastes into, clears,
  pastes, and purges through the command handlers on an 8×8 document
- **THEN** Copy records no state, each other mutating command records exactly one
  state with its label, the cut region shows the layer below until Paste Into
  restores it with a mask and no selection, and after Purge the clipboard is
  empty and Paste is disabled

#### Scenario: Paste is disabled with an empty clipboard

- **WHEN** the clipboard has been purged
- **THEN** the Paste command is disabled

#### Scenario: A plain paste keeps the copied position and deselects

- **WHEN** a 3×3 selection at `(2, 2)` is copied and a plain Paste runs while the
  marquee is live
- **THEN** one "Paste" state adds a layer whose rect starts at `(2, 2)` and the
  marquee is dropped
