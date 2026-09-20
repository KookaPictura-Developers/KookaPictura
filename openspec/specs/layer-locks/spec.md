# layer-locks Specification

## Purpose
TBD - created by archiving change fix-app-ui-issues. Update Purpose after archive.
## Requirements
### Requirement: Position lock prevents layer movement

A layer whose lock state includes `POSITION` SHALL NOT be moved by the Move tool
drag, by a content move, or by any translate entry point. The refusal SHALL occur
at the shared mutation entry points rather than in one tool, and SHALL leave the
document unchanged with no history state. Structural panel reordering (Move Layer
Up / Down) SHALL NOT be blocked by a position lock, and a position lock SHALL NOT
block pixel edits such as a filter.

#### Scenario: The Move tool refuses a position-locked layer [llk_position_drag]

- **WHEN** the Move tool is dragged and the target layer is position-locked
- **THEN** the layer does not move, the composite is unchanged, and no history
  state is added

#### Scenario: A content move refuses a position-locked layer [llk_position_content]

- **WHEN** a selection content move is committed on a position-locked layer
- **THEN** the move is refused and the layer's pixels and rect are unchanged

#### Scenario: A filter is not blocked by a position lock [llk_position_not_pixels]

- **WHEN** a filter is applied to a position-locked layer that is not
  pixel-locked
- **THEN** the filter applies and the position lock does not refuse it

#### Scenario: Panel reordering is not blocked [llk_position_reorder]

- **WHEN** Move Layer Up is run on a position-locked layer
- **THEN** the layer reorders within its container in one undo step

### Requirement: Pixel lock prevents pixel mutation

A layer whose lock state includes `PIXELS` SHALL NOT have its pixels mutated by
painting, a filter application, a content move, or a fill. The refusal SHALL be
checked at each mutation entry point and SHALL return a typed refusal without
partially writing the layer, leaving its channels bit-identical.

#### Scenario: A brush stroke is refused on a pixel-locked layer [llk_pixels_paint]

- **WHEN** a paint stroke begins on a pixel-locked layer
- **THEN** the stroke is refused, no pixels change, no history state is added,
  and the document is not marked dirty

#### Scenario: A filter is refused on a pixel-locked layer [llk_pixels_filter]

- **WHEN** a filter is applied to a pixel-locked layer
- **THEN** the call returns a refusal error and the layer's channels are
  bit-identical to their input

#### Scenario: A fill is refused on a pixel-locked layer [llk_pixels_fill]

- **WHEN** a fill is applied to a pixel-locked layer
- **THEN** the fill is refused and the layer is unchanged

### Requirement: Transparency lock prevents alpha mutation

A layer whose lock state includes `TRANSPARENCY` SHALL refuse an edit that would
change its transparency, including an erase or a `Clear` paint mode, while still
allowing an opaque-preserving color edit where the contract permits it.

#### Scenario: An erase is refused on a transparency-locked layer [llk_transparency]

- **WHEN** a `Clear`-mode paint or an erase would reduce a transparency-locked
  layer's alpha
- **THEN** the edit is refused and the layer's alpha channel is bit-identical

### Requirement: Refusal is visible to the user

When a lock refuses an action, the system SHALL surface the refusal to the user
rather than silently doing nothing, and while a pixel-editing tool is active over
a pixel-locked layer the canvas cursor SHALL indicate the layer is not editable.

#### Scenario: A locked pixel edit shows a refusal [llk_refusal_ui]

- **WHEN** the user attempts a pixel edit on a pixel-locked layer
- **THEN** the system reports the refusal (for example in the status bar or a
  warning) and the document is unchanged

#### Scenario: The cursor indicates a pixel lock [llk_locked_cursor]

- **WHEN** a pixel-editing tool is active over a pixel-locked layer
- **THEN** the canvas cursor indicates the action is not allowed

