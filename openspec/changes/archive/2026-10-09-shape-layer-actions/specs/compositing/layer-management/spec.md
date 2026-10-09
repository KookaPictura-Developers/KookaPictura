## ADDED Requirements

### Requirement: Copy and Paste Shape Attributes

The system SHALL provide `Copy Shape Attributes` and `Paste Shape Attributes`
for shape layers. Copy SHALL capture a shape layer's fill and stroke
appearance: whether the fill is on and its colour, and whether the stroke is on
with its colour, width, and alignment. Paste SHALL apply the captured
appearance to a shape layer, replacing its fill and stroke; a target that is not
a shape layer MUST be refused, leaving it unchanged and adding no history state.
Copy SHALL NOT change the document and SHALL add no history state. Paste that
changes the layer SHALL recomposite and record exactly one undo state; a paste
that changes nothing SHALL add no history state. Both SHALL be reachable from
the `Layer` menu and from the Layers row context menu for a shape layer.

#### Scenario: Copy then paste transfers fill and stroke

- **WHEN** Copy Shape Attributes runs on a shape layer with a fill and a
  stroke, and Paste Shape Attributes then runs on a different shape layer
- **THEN** the target's fill colour and stroke colour, width, and alignment
  equal the source's, and the paste is one undo step

#### Scenario: Pasting onto a non-shape is refused

- **WHEN** Paste Shape Attributes runs with a pixel, group, adjustment, type, or
  Background layer as the target
- **THEN** the target is unchanged and no history state is recorded

#### Scenario: Copying a shape changes nothing

- **WHEN** Copy Shape Attributes runs on a shape layer
- **THEN** the document is unchanged and no history state is recorded

#### Scenario: A paste with no change adds no history

- **WHEN** Paste Shape Attributes applies the same fill and stroke the target
  already has
- **THEN** the layer is unchanged and no history state is recorded

## MODIFIED Requirements

### Requirement: Rasterize subset

The system SHALL provide a `Rasterize` submenu. `Fill Content` SHALL be
implemented: for a fill-content layer (a layer whose opaque adjustment block is
a fill-content key — `SoCo`, `GdFl`, or `PtFl` — and whose payload decodes to
`Adjustment::SolidFill`, `Adjustment::GradientFill`, or
`Adjustment::PatternFill`) it SHALL render the fill content to a full-layer
pixel node and clear the fill/adjustment data. A solid fill SHALL bake its
decoded colour across the layer rect; a gradient fill SHALL bake the generated
five-kind ramp with opaque alpha across the layer rect; a pattern fill SHALL
bake the pattern the document's `Patt` resource tiles over the layer rect (with
the pattern's own alpha, or the grey placeholder when the referenced pattern is
absent or was skipped as non-8-bit/malformed). `Shape` SHALL be implemented: a
shape layer (a solid fill cut to a `vmsk` vector mask) SHALL have its rendered
appearance — the fill cut to the outline, plus its stroke — baked into ordinary
pixels, and its shape/vector definition dropped (the solid-fill adjustment, the
`vmsk` vector mask, the live-shape `vogk` block, and the stroke effect), while
keeping its name, rect, opacity, fill, blend mode, and layer mask. `Rasterize
Layer` SHALL rasterize the active layer only when it is a fill-content layer,
and SHALL otherwise refuse without changing the document. `Rasterize All
Layers` SHALL rasterize every fill-content layer and every shape layer in the
document. The `Type`, `Vector Mask`, `Smart Object`, `Video`, and `3D` variants
SHALL remain visible and disabled, with a documented reason that those layer
kinds do not exist or are handled elsewhere. A rasterization whose content is
not decodable SHALL be disabled and SHALL refuse without changing the document,
and every applied rasterization SHALL recomposite and record exactly one undo
state.

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

#### Scenario: A pattern fill is rasterizable

- **WHEN** Rasterize Fill Content runs on a layer whose `PtFl` payload decodes to
  `Adjustment::PatternFill`
- **THEN** the layer becomes a pixel layer holding the tiled pattern over its
  rect, its fill / adjustment data is cleared, and the change is one undo step

#### Scenario: A pattern fill with a missing pattern is still rasterizable

- **WHEN** Rasterize Fill Content runs on a `PtFl` layer whose `pattern_id` is
  not present in the document's `Patt` resource
- **THEN** the layer becomes a pixel layer holding the grey placeholder and
  the change is one undo step

#### Scenario: A shape layer rasterizes to pixels

- **WHEN** Rasterize Shape runs on a shape layer
- **THEN** the layer's fill cut to its outline (and its stroke) is baked into
  the layer's pixels, its fill/adjustment, vector mask, live-shape, and stroke
  data are dropped, its appearance is unchanged when recomposited, and the
  change is one undo step

#### Scenario: Rasterize Shape refuses a non-shape

- **WHEN** Rasterize Shape runs on a pixel, group, adjustment, type, or
  Background layer
- **THEN** the document is unchanged and no undo state is recorded

#### Scenario: A kind-less variant stays disabled

- **WHEN** the Rasterize submenu is shown
- **THEN** Type, Vector Mask, Smart Object, Video, and 3D are visible but
  disabled, and their reason is documented

#### Scenario: Rasterize Layer refuses a non-fill layer

- **WHEN** Rasterize Layer runs on a group, adjustment, or plain pixel layer
- **THEN** the document is unchanged and no undo state is recorded
