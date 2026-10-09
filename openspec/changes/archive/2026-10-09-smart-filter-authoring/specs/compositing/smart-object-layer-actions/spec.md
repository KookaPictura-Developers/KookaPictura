## ADDED Requirements

### Requirement: Smart-filter list document operations

The engine SHALL expose `pictura_render::add_smart_filter(doc, path, filter_id,
name, options) -> bool`, `delete_smart_filter(doc, path, index) -> bool`,
`reorder_smart_filters(doc, path, from, to) -> bool`, and
`clear_smart_filters(doc, path) -> bool`. Each SHALL resolve `path` and SHALL act
only on a layer that is not a group, has no adjustment data, and carries a smart
object; otherwise it SHALL return `false` without mutating the document. On an
eligible layer the operations SHALL delegate to the codec descriptor edits so the
preserved `filterFX` list and the typed `smart_filters` view change together.
`add_smart_filter` SHALL attach the filter enabled, replacing an existing entry
with the same `filter_id`. A refusal from the codec SHALL return `false` without
mutating.

#### Scenario: Adding appends or replaces a filter

- **WHEN** `add_smart_filter` is called on an eligible smart-object layer with a `filter_id` not already present
- **THEN** it returns `true` and the typed list gains one enabled entry with that id

#### Scenario: Deleting and reordering follow the codec

- **WHEN** `delete_smart_filter` and `reorder_smart_filters` are called on an eligible layer
- **THEN** each returns `true` and the typed list reflects the removal or move

#### Scenario: Clearing empties the stack

- **WHEN** `clear_smart_filters` is called on an eligible layer with filters
- **THEN** it returns `true` and the typed list is empty

#### Scenario: A non-smart layer is refused

- **WHEN** any of the operations is called on a layer with no smart object, a group, or an adjustment layer
- **THEN** it returns `false` and the document is unchanged

#### Scenario: An out-of-range index is refused

- **WHEN** delete or reorder is called with an index outside the typed list
- **THEN** it returns `false` and the document is unchanged

### Requirement: The Clear Smart Filters command records exactly one undo state

The application SHALL expose `Layer > Smart Filter > Clear Smart Filters` with
the stable id `layer.smartFilters.clear`. The command SHALL be enabled only when
the current layer carries at least one smart filter. On dispatch it SHALL invoke
the engine operation on the current layer, recomposite, and record exactly one
history state labelled `"Clear Smart Filters"`. It SHALL record no history state
when disabled or when no filters are present.

#### Scenario: Success records one state and empties the stack

- **WHEN** the command runs on the current layer with one or more smart filters
- **THEN** exactly one history state labelled `"Clear Smart Filters"` is added and the layer reports no smart filters

#### Scenario: Disabled when the layer has no filters

- **WHEN** the current layer is not a smart object or has no smart filters
- **THEN** the command is disabled and records no history state

#### Scenario: No document records nothing

- **WHEN** no document is open
- **THEN** the command is disabled and records no history state
