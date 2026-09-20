## ADDED Requirements

### Requirement: Alt content-move previews the duplicated pixels

The Move tool SHALL show the duplicated pixels in the live preview during an Alt
content move with a selection, and they SHALL follow the pointer during the drag,
not only the selection outline. Releasing SHALL copy the
selected pixels to a new layer offset by the drag, record exactly one
`"Move Selection"` undo state, and leave the new layer active. Cancelling SHALL
restore the document bit-identically with no history state.

#### Scenario: The clone follows the cursor [lcm_alt_preview_pixels]

- **WHEN** a selection exists and an Alt Move drag is in progress before release
- **THEN** the duplicated pixels are drawn at the dragged offset during the drag,
  not only the selection outline

#### Scenario: Commit makes the clone active with one state [lcm_alt_preview_commit]

- **WHEN** the Alt content-move drag is released after a non-zero offset
- **THEN** the copy is a new active layer and exactly one `"Move Selection"`
  state was recorded

#### Scenario: Cancel is side-effect free [lcm_alt_preview_cancel]

- **WHEN** the Alt content-move drag is cancelled before release
- **THEN** the document is bit-identical to the pre-drag state and no history
  state was added
