## ADDED Requirements

### Requirement: Space temporarily activates Hand panning

Holding the `Space` key over the canvas SHALL temporarily enable Hand panning
while the previous tool remains logically active: the cursor SHALL become
`Qt::OpenHand`, and pressing and dragging SHALL pan the canvas through the shared
range helper exactly as the Hand tool does. Releasing `Space` SHALL restore the
previous tool's cursor and pan state, so a Space pan does not change the active
tool, the tool's options, or any document state. The `Space` key SHALL NOT be
bound as a prefix of a key sequence, so it is delivered as a plain key and never
consumed waiting for a second key.

#### Scenario: Space pans without changing the tool [lct_space_pan]

- **WHEN** `Space` is held and the canvas is dragged with a non-Hand tool active
- **THEN** the canvas pans, the active tool is unchanged, and the offset is
  clamped by the shared range

#### Scenario: Releasing Space restores the cursor [lct_space_release]

- **WHEN** `Space` is released after a pan
- **THEN** the previous tool's cursor and pan state are restored

#### Scenario: Space is not a sequence prefix [lct_space_not_prefix]

- **WHEN** `Space` is pressed and released on its own
- **THEN** it is handled as the temporary pan key and is not held waiting for a
  second key in a sequence
