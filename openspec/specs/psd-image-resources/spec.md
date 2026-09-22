# psd-image-resources Specification

## Purpose
TBD - created by archiving change psd-image-resources. Update Purpose after archive.
## Requirements
### Requirement: Image resources are decoded into typed records

`pictura-codec` SHALL expose `decode_image_resources(document: &Document) ->
Vec<ImageResource>` that parses the document's preserved image-resource section
into typed records in stored order. Each record SHALL carry the resource `id`
(`u16`), its `name` (the Pascal string with the length byte removed and a
trailing NUL stripped, decoded lossily as text), and its `data` bytes verbatim.
A block SHALL be read as a 4-byte signature, a `u16` id, a Pascal name padded to
an even length, a `u32` data length, and that many data bytes padded to an even
length. The signatures `8BIM`, `8B64`, `MeSa`, `AgHg`, `PHUT`, and `DCSR` SHALL
be accepted; any other signature, a truncated block, or trailing bytes too short
to frame a block SHALL end parsing and the records decoded so far SHALL be
returned. The parser SHALL NOT panic and SHALL NOT allocate without bound on a
malformed section.

#### Scenario: A resource section decodes to typed records

- **WHEN** the document carries an image-resource section holding an EXIF resource and an XMP packet
- **THEN** `decode_image_resources` returns records carrying those ids, names, and exact data bytes in stored order

#### Scenario: The ICC profile is exposed by id

- **WHEN** the decoded records include resource id `1039`
- **THEN** the record's `data` is the embedded ICC profile bytes and the well-known id constant equals `1039`

#### Scenario: A malformed block is skipped without panicking

- **WHEN** the section ends with a truncated block or an unrecognized signature
- **THEN** `decode_image_resources` returns the records decoded before it and does not panic

#### Scenario: Byte preservation is unchanged

- **WHEN** a document read from a file with a non-empty image-resource section is written back
- **THEN** the output's image-resource bytes are identical to the input's, because the raw section is still re-emitted verbatim

### Requirement: Image-resource data is parsed with the shared codec reader

The image-resource parser SHALL use the existing bounds-checked `pictura-codec`
`Reader`, so every read is length-checked and a truncated section yields a
recoverable end-of-parse rather than an index panic. It SHALL NOT introduce a
second byte cursor.

#### Scenario: Truncation is recoverable

- **WHEN** a resource block's declared data length exceeds the remaining bytes
- **THEN** parsing stops, the prior records are returned, and no panic occurs

