## ADDED Requirements

### Requirement: Smart-filter list editing on the preserved descriptor

The codec SHALL expose operations to remove the smart filter at a given index
from a layer, to move an entry to another position, and to clear every entry.
Each operation SHALL rewrite the layer's preserved `SoLd`/`SoLE`
`filterFX.filterFXList` in place, keeping every other descriptor key, and SHALL
keep the typed `SmartObject::smart_filters` view in step. When a layer has no
preserved block the operation SHALL change only the typed view so the writer
authors the block on save. Removing the last entry SHALL drop the whole
`filterFX` object from the descriptor. An out-of-range index SHALL return an
error and SHALL NOT mutate the layer.

#### Scenario: A deleted entry disappears and survives a write

- **WHEN** a smart filter is deleted from a layer whose preserved descriptor carries two entries and the document is written and read back
- **THEN** the removed filter no longer appears in the re-read list and the surviving entry keeps its bytes and its position

#### Scenario: Deleting the last filter drops the filterFX object

- **WHEN** the only smart filter is deleted
- **THEN** the typed list is empty and a re-read document authors no `filterFX` list

#### Scenario: Reordering changes the applied order and persists

- **WHEN** the filter at index 0 is moved to index 1
- **THEN** the typed list order changes, the preserved `filterFXList` order changes to match, and the new order is unchanged by a write and re-read

#### Scenario: Clearing removes every filter

- **WHEN** clear is called on a layer with one or more smart filters and the document is written and read back
- **THEN** the re-read layer has an empty `smart_filters` list

#### Scenario: An out-of-range index is refused

- **WHEN** delete or reorder is called with an index outside the list
- **THEN** the call returns an error and the typed and preserved lists are unchanged
