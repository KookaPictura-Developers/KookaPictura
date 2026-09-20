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

A layer whose lock state includes `TRANSPARENCY` SHALL preserve each pixel's
alpha. An edit SHALL change a pixel's color channels when that pixel's
pre-existing alpha is greater than zero and SHALL write the pre-existing alpha
back unchanged, so a pixel with alpha `180` keeps alpha `180`; a pixel whose
alpha is zero SHALL be left untouched. An edit whose purpose is to lower alpha —
an erase or a `Clear` paint mode — SHALL be refused for the whole operation
because it mutates transparency, and the layer's alpha channel SHALL be
bit-identical after a refusal. This per-pixel rule SHALL be applied in the shared
write path and by filter application, not by a per-call-site exception.

#### Scenario: An opaque-preserving color edit keeps alpha [llk_transparency_color]

- **WHEN** a paint or filter edit runs on a transparency-locked layer at a pixel
  whose alpha is `180`
- **THEN** that pixel's color channels change and its alpha remains `180`

#### Scenario: A fully transparent pixel is untouched [llk_transparency_clear_pixel]

- **WHEN** an edit runs on a transparency-locked layer at a pixel whose alpha is
  zero
- **THEN** the pixel's color and alpha are unchanged

#### Scenario: An erase is refused [llk_transparency]

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

### Requirement: Nesting lock prevents reparenting

A move that would change a layer's structural parent SHALL be refused when either
side of the move is nesting-locked: a drop into a nesting-locked group and a drop
out of a nesting-locked group SHALL each leave the document unchanged with no
history state. A drop that keeps the layer in the same container — a
within-container reorder — SHALL remain allowed. This SHALL be enforced at the
shared move entry points (the group, ungroup, and move-to-container operations),
not only on the dragged source, so the destination group and the source parent are
both checked. Until artboards and frames exist, the rule applies to groups.

#### Scenario: Dropping into a nesting-locked group is refused [llk_nesting_in]

- **WHEN** a row is dragged and dropped onto a nesting-locked group
- **THEN** the move is refused, the layer's parent is unchanged, and no history
  state is added

#### Scenario: Dropping out of a nesting-locked group is refused [llk_nesting_out]

- **WHEN** a row whose parent group is nesting-locked is dragged out of that group
- **THEN** the move is refused, the parent is unchanged, and no history state is
  added

#### Scenario: A within-container reorder is still allowed [llk_nesting_reorder]

- **WHEN** a row is reordered among its siblings inside a nesting-locked group
- **THEN** the reorder applies and both layers keep the same parent

### Requirement: Transparent-pixel lock is per-pixel across the bridge

With the transparent-pixel lock enabled on the active layer, a paint stroke SHALL
modify the RGB of pixels whose alpha is non-zero while preserving each pixel's
existing alpha value exactly, and SHALL leave fully transparent (`A=0`) pixels
untouched. A pixel with `A=180` SHALL remain `A=180` after editing while its RGB
may change. The filter path SHALL follow the same rule. Content move keeps its
existing separate transparency refusal, recorded as a documented divergence.

#### Scenario: Alpha is preserved while RGB changes [llk_transparency_preserve]

- **WHEN** a stroke is painted over a pixel with `A=180` on a transparency-locked
  layer
- **THEN** the pixel's RGB may change and its alpha remains exactly 180

#### Scenario: Fully transparent pixels are untouched [llk_transparency_zero]

- **WHEN** a stroke is painted over a pixel with `A=0` on a transparency-locked
  layer
- **THEN** the pixel stays fully transparent

