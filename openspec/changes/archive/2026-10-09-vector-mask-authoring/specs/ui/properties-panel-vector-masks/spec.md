# properties-panel-vector-masks

## ADDED Requirements

### Requirement: Properties vector mask section

When the active layer carries a vector mask, the Properties panel SHALL show a
Vector Mask section containing a name row and the actions Enable, Disable, Link,
Unlink, Delete, and Rasterize, each wired to the vector-mask bridge and each one
undoable step, plus read-only Density and Feather rows that are disabled and
carry the `— not implemented yet` tooltip because the model does not store them.
The section SHALL be absent for a layer without a vector mask, and the existing
adjustment page and layer-mask section behavior SHALL be unchanged.

#### Scenario: A vector-masked layer shows the section [vmk_props]

- **WHEN** the active layer carries a vector mask
- **THEN** the Properties panel shows the Vector Mask section with the name row
  and the Enable/Disable, Link/Unlink, Delete, and Rasterize actions

#### Scenario: The section is absent without a vector mask [vmk_props_absent]

- **WHEN** the active layer has no vector mask
- **THEN** the Properties panel shows no Vector Mask section

#### Scenario: Delete clears the vector mask from the panel [vmk_props_delete]

- **WHEN** the user activates Delete in the Vector Mask section
- **THEN** the vector mask is removed from the active layer in one undo step and
  the section disappears

#### Scenario: Rasterize replaces the vector mask in the panel [vmk_props_rasterize]

- **WHEN** the user activates Rasterize in the Vector Mask section
- **THEN** the vector mask is converted to a layer mask in one undo step and the
  Vector Mask section disappears

#### Scenario: Density and Feather are disabled placeholders [vmk_props_stubs]

- **WHEN** the Vector Mask section is shown
- **THEN** its Density and Feather rows are disabled and carry the `— not
  implemented yet` tooltip
