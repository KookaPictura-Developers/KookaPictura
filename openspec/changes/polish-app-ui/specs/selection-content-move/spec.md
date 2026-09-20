## MODIFIED Requirements

### Requirement: Duplicate selected pixels to a new layer

Holding Alt while the Move tool drags a selection SHALL copy the selected pixels
into a new layer directly above the source and move that new layer, leaving the
source layer intact. A selection tool with Ctrl+Alt held SHALL perform the same
duplicate. The duplicate SHALL be a single layer inserted above the source.
**The duplicate SHALL be previewed live during the drag** (the dragged content
moves under the cursor before release) **and SHALL become the active layer on
commit**, so the next edit targets the clone. The duplicate SHALL record exactly
one history state, on commit, and no state during the drag; a zero-offset
Alt-drag SHALL record nothing.

#### Scenario: Move tool Alt-drag duplicates

- **WHEN** a selection is active and the Move tool is dragged with Alt held
- **THEN** the selected pixels are copied to a new layer that moves with the drag and the source layer keeps its pixels

#### Scenario: The clone previews during the drag [lcm_alt_preview]

- **WHEN** a selection is active and the Move tool is dragged with Alt held before release
- **THEN** the duplicate content follows the cursor in the live preview and the source layer is unchanged until commit

#### Scenario: The clone becomes active [lcm_alt_active]

- **WHEN** the Alt-drag commits
- **THEN** the duplicated layer is the active layer and exactly one history state is recorded

#### Scenario: Select tool Ctrl+Alt duplicates

- **WHEN** a selection exists and a selection tool is dragged inside it with Ctrl+Alt held
- **THEN** the selected pixels are copied to a new layer that moves with the drag
