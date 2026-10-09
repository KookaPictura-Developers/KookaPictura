# properties-panel-masks

## ADDED Requirements

### Requirement: Properties mask section

When the active layer carries a layer mask, the Properties panel SHALL show a
Mask section containing a mask name row and the actions Enable, Disable, Link,
Unlink, Delete, and Apply, each wired to the layer-mask bridge and each one
undoable step, plus read-only Density, Feather, and Invert rows that are
disabled and carry the `— not implemented yet` tooltip because the model does
not store them. The section SHALL be absent for a layer without a mask, and the
existing adjustment page behavior SHALL be unchanged.

#### Scenario: A masked layer shows the section [lmk_props]

- **WHEN** the active layer carries a mask
- **THEN** the Properties panel shows the Mask section with the mask name row
  and the Enable/Disable, Link/Unlink, Delete, and Apply actions

#### Scenario: The section is absent without a mask [lmk_props_absent]

- **WHEN** the active layer has no mask
- **THEN** the Properties panel shows no Mask section

#### Scenario: Delete clears the mask from the panel [lmk_props_delete]

- **WHEN** the user activates Delete in the Mask section
- **THEN** the mask is removed from the active layer in one undo step and the section disappears

#### Scenario: Density, Feather, and Invert are disabled placeholders [lmk_props_stubs]

- **WHEN** the Mask section is shown
- **THEN** its Density, Feather, and Invert rows are disabled and carry the
  `— not implemented yet` tooltip
