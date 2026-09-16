## ADDED Requirements

### Requirement: Layer fill opacity folds into effective source alpha

Each layer's `fill` (0..=255) SHALL multiply the effective source alpha, in
addition to its `opacity`, in both the CPU oracle and the GPU compositor: a
sample's effective alpha SHALL be `opacity/255 × fill/255` (with the raster mask
and clipping folded in as before). When `fill == 255` the composite MUST be
byte-identical to the layer's contribution without a fill factor, so existing
documents are unchanged. When `fill != 255` the GPU result MUST remain within
the existing ±1 LSB CPU parity. A group's `fill` SHALL have no effect on
compositing (CS6 exposes no group Fill), but the stored value SHALL round-trip
through PSD.

#### Scenario: Default fill is byte-identical

- **WHEN** a document whose layers all have fill 255 is composited
- **THEN** the result is byte-identical to compositing the same document without any fill factor

#### Scenario: Fill scales the layer contribution

- **WHEN** a full-alpha source layer with fill 128 is composited over an opaque backdrop
- **THEN** the result is the source-over mix with effective source alpha `opacity/255 × 128/255`

#### Scenario: Fill combines with opacity and mask

- **WHEN** a masked layer with opacity 200 and fill 128 is composited
- **THEN** its contribution is scaled by `200/255 × 128/255` and the mask sample, in the same order as the CPU oracle

#### Scenario: GPU matches the CPU oracle with fill

- **WHEN** a scene with a non-default fill is composited on the GPU and on the CPU
- **THEN** the two results agree within ±1 LSB per channel

#### Scenario: Group fill is inert

- **WHEN** a group carries a non-default fill value
- **THEN** the group composites as if its fill were 255, while the stored value survives a PSD round-trip
