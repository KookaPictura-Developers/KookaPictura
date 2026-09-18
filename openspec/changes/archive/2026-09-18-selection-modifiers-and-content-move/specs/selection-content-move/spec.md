## ADDED Requirements

### Requirement: Move selected pixels
When a selection is active, the Move tool SHALL drag the selected pixels rather
than the whole layer: the covered pixels are cleared from the source layer and
re-created at the dragged offset inside that same layer, and the selection mask
moves with them by the same offset. With no active selection, the Move tool
SHALL continue to move the whole layer.

#### Scenario: Drag moves the selected pixels
- **WHEN** a selection is active and the Move tool is dragged by `(dx, dy)`
- **THEN** the selected pixels appear offset by `(dx, dy)`, the source area is cleared, and the layer count is unchanged

#### Scenario: The selection follows the pixels
- **WHEN** the Move tool commits a selected-pixel drag by `(dx, dy)`
- **THEN** the active selection mask is translated by `(dx, dy)`

#### Scenario: No selection still moves the layer
- **WHEN** no selection is active and the Move tool is dragged
- **THEN** the whole layer moves, as before

### Requirement: Duplicate selected pixels to a new layer
Holding Alt while the Move tool drags a selection SHALL copy the selected pixels
into a new layer directly above the source and move that new layer, leaving the
source layer intact. A selection tool with Ctrl+Alt held SHALL perform the same
duplicate. The duplicate SHALL be a single layer inserted above the source.

#### Scenario: Move tool Alt-drag duplicates
- **WHEN** a selection is active and the Move tool is dragged with Alt held
- **THEN** the selected pixels are copied to a new layer that moves with the drag and the source layer keeps its pixels

#### Scenario: Select tool Ctrl+Alt duplicates
- **WHEN** a selection exists and a selection tool is dragged inside it with Ctrl+Alt held
- **THEN** the selected pixels are copied to a new layer that moves with the drag

### Requirement: Temporary content move with a selection tool
Holding Ctrl while a selection tool drags inside the active selection SHALL move
the selected pixels exactly as the Move tool does, without changing the active
tool. Without Ctrl (and without Shift or Alt) a press inside the selection SHALL
keep the existing behaviour of moving only the selection outline.

#### Scenario: Ctrl moves the pixels
- **WHEN** a selection is active, a selection tool is active, and Ctrl is held while dragging inside the selection
- **THEN** the selected pixels move by the drag offset and no new selection is started

#### Scenario: Plain drag still moves only the outline
- **WHEN** a selection is active, a selection tool is active, and no modifier is held during a drag inside the selection
- **THEN** only the selection outline moves and the pixel content is unchanged

#### Scenario: A quick-mode modifier does not move
- **WHEN** a selection is active and a selection tool drags inside it with Shift or Alt held but not Ctrl
- **THEN** no content move occurs and the gesture combines a new selection per the quick mode

### Requirement: Content move records one undo state
A completed content move (move or duplicate) SHALL record exactly one history
state labelled `Move Selection`; a zero-offset drag, a missing selection, or a
missing document SHALL record nothing.

#### Scenario: One state per move
- **WHEN** a selected-pixel move or duplicate completes
- **THEN** exactly one `Move Selection` state is added to the history

#### Scenario: No-op records nothing
- **WHEN** a content-move gesture ends with a zero offset, or there is no selection
- **THEN** no history state is recorded and the document is unchanged
