## ADDED Requirements

### Requirement: Decode a preserved `vscg` block into typed fill content

The compositor SHALL decode a layer's preserved `'vscg'` additional-layer-info
block into the matching `pictura_adjust::Adjustment` variant when the layer has
no modeled adjustment block. The block SHALL be a big-endian 4-byte fill key
followed by a big-endian `u32` version that MUST equal `16` and a descriptor. The
descriptor SHALL be decoded by the same decoders the top-level `'SoCo'`,
`'GdFl'`, and `'PtFl'` blocks use, dispatching on its content: a `Grad` object
SHALL decode to `Adjustment::GradientFill`, a `Ptrn` object SHALL decode to
`Adjustment::PatternFill`, and a `Clr ` object SHALL decode to
`Adjustment::SolidFill`. The 4-byte key SHALL be consumed but SHALL NOT be the
dispatch authority, so a block whose key disagrees with its descriptor content
still decodes by content.

A missing key, a version other than `16`, a non-object descriptor, a descriptor
with none of those contents, or a malformed descriptor SHALL decode to `None`,
SHALL NOT change the backdrop, and SHALL NOT panic. The preserved `'vscg'` block
SHALL remain in `Layer.extra_blocks` and SHALL be re-emitted verbatim; the decode
SHALL be derived only.

#### Scenario: A `vscg` solid fill decodes to `SolidFill`

- **WHEN** a layer carries a `vscg` block whose descriptor holds a `Clr ` object
  with `Rd `/`Grn `/`Bl  ` doubles
- **THEN** the decoded fill is `Adjustment::SolidFill` with those components and
  alpha 255

#### Scenario: A `vscg` gradient fill reuses the gradient decoder

- **WHEN** a layer carries a `vscg` block whose descriptor holds `Angl`, a `Type`
  enum, and a custom-stop `Grad` object
- **THEN** the decoded fill is `Adjustment::GradientFill` with the same kind,
  stops, and angle a top-level `GdFl` block would yield

#### Scenario: A malformed `vscg` block is a no-op

- **WHEN** a `vscg` block has a version other than 16, a non-object descriptor,
  or a descriptor with none of `Grad`/`Ptrn`/`Clr `
- **THEN** the decode yields `None` and does not panic

#### Scenario: The decoded block is not serialized

- **WHEN** a document carrying a `vscg` block is written and read back
- **THEN** the block's bytes are preserved and the reconstructed document equals
  the input

### Requirement: A vector fill composites and is clipped by the vector mask

A layer whose decoded adjustment comes from `'vscg'` SHALL composite as
generative fill content over the layer rectangle clamped to the canvas, through
the normal layer path, so the layer's mask, opacity, fill, and blend mode apply
and pixels outside the rectangle are unchanged. The fill SHALL be clipped by the
layer's vector-mask coverage when the layer carries an enabled `'vmsk'`, and
SHALL cover the layer rectangle when the layer has no vector mask. A modeled
layer adjustment block SHALL take precedence, so `'vscg'` SHALL be consulted only
when the layer has no adjustment block; an adjustment block that does not decode
SHALL leave the layer a no-op and SHALL NOT fall through to its pixels.

A scene containing a vector fill SHALL composite through the CPU path: the GPU
compositor SHALL fall back rather than render a layer carrying a `vscg` block, so
the CPU/GPU parity contract holds. Layer effects SHALL gate their coverage by the
same decoded vector fill a shape layer's content uses.

#### Scenario: A `vscg` solid fill composites inside its vector mask

- **WHEN** a shape layer carrying a `vscg` solid fill and a closed-rectangle
  `vmsk` is composited over an opaque base
- **THEN** the pixels inside the rectangle carry the fill and the pixels outside
  it carry the base

#### Scenario: A vector fill without a vector mask fills the layer rect

- **WHEN** a layer carrying a `vscg` solid fill has no vector mask
- **THEN** its fill covers the layer rectangle and leaves pixels outside it
  unchanged

#### Scenario: A modeled adjustment block takes precedence over `vscg`

- **WHEN** a layer carries both a decodable `SoCo` adjustment block and a `vscg`
  block
- **THEN** the layer composites the adjustment block's fill

#### Scenario: A malformed `vscg` fill does not change the composite

- **WHEN** a layer's `vscg` block does not decode
- **THEN** the composite equals the same document with the `vscg` block absent

#### Scenario: The GPU falls back for a vector fill

- **WHEN** a document containing a visible layer with a `vscg` block is
  composited with GPU acceleration enabled
- **THEN** the compositor reports the CPU backend and produces the CPU result
