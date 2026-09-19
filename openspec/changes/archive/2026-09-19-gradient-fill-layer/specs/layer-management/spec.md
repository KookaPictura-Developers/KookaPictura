## MODIFIED Requirements

### Requirement: Rasterize subset

The system SHALL provide a `Rasterize` submenu. `Fill Content` SHALL be
implemented: for a fill-content layer (a layer whose opaque adjustment block is
a fill-content key — `SoCo` or `GdFl` — and whose payload decodes to
`Adjustment::SolidFill` or `Adjustment::GradientFill`) it SHALL render the fill
content to a full-layer pixel node and clear the fill/adjustment data. A solid
fill SHALL bake its decoded colour across the layer rect; a gradient fill SHALL
bake the generated five-kind ramp with opaque alpha across the layer rect.
`Rasterize Layer` SHALL rasterize the active layer only when it is a fill-content
layer, and SHALL otherwise refuse without changing the document. `Rasterize All
Layers` SHALL rasterize every fill-content layer in the document. The `Type`,
`Shape`, `Vector Mask`, `Smart Object`, `Video`, and `3D` variants SHALL remain
visible and disabled, with a documented reason that those layer kinds do not
exist in the model. A rasterization whose content is not decodable SHALL be
disabled and SHALL refuse without changing the document, and every applied
rasterization SHALL recomposite and record exactly one undo state.

#### Scenario: Fill Content becomes pixels

- **WHEN** Rasterize Fill Content runs on a solid fill-content layer
- **THEN** the layer becomes a pixel layer holding the rendered fill, its fill /
  adjustment data is cleared, and the change is one undo step

#### Scenario: A descriptor-form fill is rasterizable

- **WHEN** Rasterize Fill Content runs on a layer whose `SoCo` payload is the
  standard Photoshop descriptor
- **THEN** the layer becomes a pixel layer holding the descriptor's colour and
  the change is one undo step

#### Scenario: A gradient fill is rasterizable

- **WHEN** Rasterize Fill Content runs on a layer whose `GdFl` payload decodes to
  `Adjustment::GradientFill`
- **THEN** the layer becomes a pixel layer holding the generated gradient over
  its rect, its fill / adjustment data is cleared, and the change is one undo
  step

#### Scenario: A kind-less variant stays disabled

- **WHEN** the Rasterize submenu is shown
- **THEN** Type, Shape, Vector Mask, Smart Object, Video, and 3D are visible but
  disabled, and their reason is documented

#### Scenario: Rasterize Layer refuses a non-fill layer

- **WHEN** Rasterize Layer runs on a group, adjustment, or plain pixel layer
- **THEN** the document is unchanged and no undo state is recorded
