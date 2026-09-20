## ADDED Requirements

### Requirement: Tool refusal follows the active layer

The tool refusal SHALL resolve the **active** single layer's lock state and
visibility — the same layer the paint and filter edits target — not the topmost
pixel layer. The brush/pencil blank-cursor affordance, the Block/Forbidden
refusal cursor, and the edit-refusal status message SHALL all follow this. When the active layer is pixel- or position-locked, or is
invisible, the refusal cursor and message SHALL appear even if the topmost pixel
layer is editable; when the topmost layer is locked but a lower layer is active
and editable, the cursor SHALL stay blank (brush) and the edit SHALL proceed. The
transient Alt-eyedropper precedence SHALL be unchanged.

#### Scenario: A locked active layer refuses with the Block cursor [tf_refusal_active_locked]

- **WHEN** the active layer is pixel-locked and is not the topmost pixel layer
- **THEN** the brush shows the Block/Forbidden cursor and refuses the edit

#### Scenario: A locked topmost layer does not block a lower active layer [tf_refusal_topmost_ignored]

- **WHEN** the topmost pixel layer is locked but the active layer is a lower,
  editable layer
- **THEN** the brush cursor stays blank and the edit targets the active layer

#### Scenario: A panel multi-selection prevents edits [tf_refusal_multi]

- **WHEN** the Layers panel has more than one row selected, or no row
- **THEN** tool edits are refused with no history state
