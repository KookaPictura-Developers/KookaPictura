## MODIFIED Requirements

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

## ADDED Requirements

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
