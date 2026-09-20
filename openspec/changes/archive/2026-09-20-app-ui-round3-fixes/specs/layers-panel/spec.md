## ADDED Requirements

### Requirement: Reparent into a group resolves post-removal indices

A layer drag SHALL be able to reparent a layer into a group and out of a group
regardless of the dragged layer's position relative to the target. The
destination parent SHALL be resolved in post-removal coordinates, so a layer that
sits above the target group is not mis-resolved after its own removal. The move
SHALL insert the layer as the last child of the group and keep every other row at
its original position; a successful move is one undoable step. All existing
refusals SHALL remain: the Background, a fully- or nesting-locked source, a drop
onto the source or into its own descendant, and an Into target that is not a
group.

#### Scenario: A layer above a group can be dropped into it [lpr_drag_into_above]

- **WHEN** the root has `[A, G]` (G a group) and A is dragged onto G
- **THEN** A becomes G's last child and the move succeeds as one undo step

#### Scenario: A layer above a group with siblings below does not mis-nest [lpr_drag_into_above_siblings]

- **WHEN** the root has `[A, G, B]` and A is dragged onto G
- **THEN** A becomes G's child, A no longer appears at the root, and B stays at
  the root in its original position

#### Scenario: A group child can be moved out of its group [lpr_drag_out]

- **WHEN** a child of a group is dragged to the empty viewport below the last row
  or above/below a root sibling
- **THEN** it is reparented to the root (or the sibling's container) in one undo
  step and no other row changes position

#### Scenario: Invalid reparents are still refused [lpr_drag_into_refused]

- **WHEN** a layer is dropped into its own descendant, onto itself, or a
  Background/locked source is dragged
- **THEN** the move is refused and the document is unchanged
