## MODIFIED Requirements

### Requirement: Alpha and layer metadata preservation

On a layer that is not transparency-locked, `apply_filter` SHALL filter the transparency channel (`id == -1`) only for a kernel that moves, spreads, or ranks samples independently of their value (the blurs, the sharpens, Median, Despeckle, Maximum, Minimum, Dust & Scratches, Offset, Mosaic, Crystallize, Fragment, and the geometric Distort filters), by running that kernel over the alpha plane, so the layer edge moves with the colour. Every other filter MUST leave the transparency channel bit-identical. Offset without wrap SHALL make the area it exposes fully opaque on the transparency channel, as CS6's "Set to Background" does. `apply_filter` MUST NOT change `layer.rect`, `layer.mask`, `layer.opacity`, or `layer.blend`.

#### Scenario: Transparency is bit-identical

- **WHEN** `apply_filter` runs Solarize, Clouds, Difference Clouds, Fibers, Lens Flare, Find Edges, Trace Contour, Tiles, Emboss, High Pass, or Add Noise runs on an unlocked layer that is opaque or partly transparent
- **THEN** the channel with id `-1` equals the input bit for bit

#### Scenario: A moving kernel keeps an opaque layer opaque

- **WHEN** a blur, Median, Mosaic, a geometric Distort filter, or Offset with wrap off and a black Background runs on an unlocked layer whose alpha is 255 everywhere
- **THEN** every alpha sample is still 255

#### Scenario: Offset moves the layer edge

- **WHEN** Offset with wrap off shifts a layer whose right half is transparent to the right
- **THEN** the transparent half moves right with the colour and the exposed left columns are opaque

#### Scenario: Layer metadata is unchanged

- **WHEN** `apply_filter` returns success
- **THEN** `rect`, `mask`, `opacity`, and `blend` equal their pre-call values
