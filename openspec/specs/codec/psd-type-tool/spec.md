# psd-type-tool Specification

## Purpose
Decodes TySh into a typed TypeTool view, round-trips unmodified bytes, and rebuilds a well-formed block.
## Requirements
### Requirement: TySh decodes into a typed TypeTool view

The system SHALL parse a layer's preserved `TySh` additional-layer-info block
into a derived `TypeTool` view when the payload is well-formed. The view SHALL
carry: the six `f64` affine transform values (`xx`, `xy`, `yx`, `yy`, `tx`,
`ty`); the text string from the text descriptor's `Txt ` key when present; the
four `i32` bounds `left`, `top`, `right`, `bottom`; and the text and warp
descriptors as parsed descriptor objects. A payload whose version is not `1`,
whose text version is not `50`, whose warp version is not `1`, or whose
descriptors do not parse SHALL leave the view `None` and MUST NOT fail the
document read. Detection of type-layer kind SHALL remain presence of `TySh` in
`extra_blocks`.

#### Scenario: A well-formed TySh yields a TypeTool view

- **WHEN** a layer carries a `TySh` block with version 1, a 6-value transform,
  text version 50, parseable text and warp descriptors, and four bounds
- **THEN** `layer.type_tool()` is `Some` with that transform, the `Txt ` string,
  and those bounds

#### Scenario: Missing Txt yields an empty text string

- **WHEN** the text descriptor parses but has no `Txt ` key
- **THEN** the view's text string is empty and the other fields still decode

#### Scenario: Malformed TySh leaves the view unset without failing the document

- **WHEN** `TySh` has version other than 1 or a truncated descriptor
- **THEN** document read succeeds and `type_tool()` is `None`

### Requirement: TySh bytes round-trip on unmodified save

An unmodified document whose layer carries `TySh` SHALL re-emit the same
`TySh` key and payload bytes on write, because serialization SHALL continue to
use `Layer.extra_blocks` rather than the derived view.

#### Scenario: Open-save preserves TySh bytes

- **WHEN** a document with a well-formed `TySh` is written and read back
- **THEN** the layer's `TySh` payload is byte-identical to the input

### Requirement: encode_type_tool rebuilds a well-formed block

`pictura-codec` SHALL expose `encode_type_tool(&TypeTool) -> Vec<u8>` that
emits version 1, the six transform doubles, text version 50, the text
descriptor (version-16 block), warp version 1, the warp descriptor, and the
four `i32` bounds. Decoding the encoder's output SHALL yield a `TypeTool` equal
to the input for transform, text string (via `Txt `), and bounds.

#### Scenario: Encode then decode preserves fields

- **WHEN** a `TypeTool` is encoded and the bytes are decoded
- **THEN** transform, `Txt ` text, and bounds match the input view

#### Scenario: Encoder output is version 1 with text version 50

- **WHEN** `encode_type_tool` is called on any well-formed view
- **THEN** the first `u16` is 1 and the `u16` after the transform is 50

