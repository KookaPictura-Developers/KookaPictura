# Specs delta: knko-blend-if-model

## ADDED Requirements

### Requirement: Advanced-blending tagged blocks decode into layer fields

The system SHALL parse a layer's `knko`, `clbl`, and `infx` additional-layer-info blocks into first-class layer fields when present: `knko` into a knockout mode (`None`, `Shallow`, or `Deep` from bytes 0, 1, or 2; any other byte SHALL yield `None`), `clbl` into a blend-clipping boolean, and `infx` into a blend-interior boolean. When a block is absent, the fields SHALL default to knockout `None`, blend-clipping `true`, and blend-interior `true`. An empty or otherwise unusable payload SHALL leave the field at its default and MUST NOT fail the document read. These keys SHALL NOT be retained in the opaque `extra_blocks` list after a successful read.

#### Scenario: A deep-knockout block decodes

- **WHEN** a layer carries `knko` with payload byte 0 equal to 2
- **THEN** the layer's knockout mode is `Deep` and `extra_blocks` does not contain a `knko` entry

#### Scenario: Absent blocks use CS6 defaults

- **WHEN** a layer has no `knko`, `clbl`, or `infx` blocks
- **THEN** knockout is `None`, blend-clipping is `true`, and blend-interior is `true`

#### Scenario: Empty knko payload does not fail the document

- **WHEN** a layer carries `knko` with zero-length data
- **THEN** document read succeeds and knockout remains `None`

### Requirement: Advanced-blending fields write back only when non-default

On write, the system SHALL emit `knko` only when knockout is not `None`, `clbl` only when blend-clipping is `false`, and `infx` only when blend-interior is `false`. Each emitted payload SHALL be four bytes: the value byte followed by three padding zeros. A document whose fields are all at their defaults SHALL not contain those keys after a write.

#### Scenario: Deep knockout round-trips

- **WHEN** a document with knockout `Deep` is written and read back
- **THEN** the written file contains a `knko` block with value 2 and the layer's knockout is `Deep`

#### Scenario: Default fields are omitted

- **WHEN** every layer is at the default advanced-blending values and the document is written
- **THEN** the output contains no `knko`, `clbl`, or `infx` keys

### Requirement: Layer blending ranges decode into a typed Blend If view

The system SHALL parse a layer's non-empty blending-ranges payload into a typed Blend If view: composite gray source and destination ranges as two `(u16, u16)` pairs each, then zero or more channel groups of source and destination ranges in the same layout, all big-endian. An empty payload SHALL leave the typed view `None` while still round-tripping the raw field. A payload whose length is not a multiple of eight bytes, or that is otherwise truncated, SHALL leave the view `None` and MUST NOT fail the document read. The raw `blending_ranges` bytes SHALL remain the serialization source for an unmodified open→save.

#### Scenario: Non-empty ranges decode

- **WHEN** a layer carries a blending-ranges body with a composite source pair and at least one channel group
- **THEN** the typed view exposes those `u16` ranges and the raw field is unchanged

#### Scenario: Empty ranges keep the raw field

- **WHEN** a layer's blending-ranges length is zero
- **THEN** document write emits an empty blending-ranges field and read succeeds

#### Scenario: Truncated ranges leave the view unset

- **WHEN** the blending-ranges body has a length that is not a valid multiple of range groups
- **THEN** document read succeeds, the typed view is `None`, and the raw bytes still round-trip on write

### Requirement: encode_blend_if rebuilds a well-formed payload

`pictura-codec` SHALL expose `encode_blend_if(&BlendIf) -> Vec<u8>` that emits the composite source pair, composite destination pair, and each channel group as big-endian `u16` pairs. Parsing the encoded bytes SHALL recover the same ranges.

#### Scenario: Encode then parse preserves ranges

- **WHEN** `encode_blend_if` output is parsed back as a blending-ranges body
- **THEN** the composite and channel ranges equal the input view
