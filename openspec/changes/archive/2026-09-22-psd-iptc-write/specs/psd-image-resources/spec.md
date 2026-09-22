## ADDED Requirements

### Requirement: An image-resource block is framed from typed fields

`pictura-codec` SHALL expose `frame_image_resource(id: u16, name: &str, data:
&[u8]) -> ImageResource` that produces a block's `raw` bytes as a 4-byte `8BIM`
signature, the big-endian `u16` id, a Pascal name padded to an even length, the
big-endian `u32` data length, and the data padded to an even length, and that
sets the record's `id`, `name`, and `data`. A name longer than 255 bytes SHALL be
truncated to 255 bytes (the Pascal length is a `u8`), and the record's `name`
SHALL be the truncated form. Framing a record and then decoding the resulting
block SHALL recover the same `id`, `name`, and `data`.

#### Scenario: A framed odd-length value round-trips

- **WHEN** a record is framed from an id, a name, and odd-length data and the resulting block is decoded
- **THEN** the decoded record has the same id, name, and data

#### Scenario: A long name is truncated to the length byte

- **WHEN** a record is framed with a 300-byte name
- **THEN** framing and decoding recover the first 255 bytes as the name
