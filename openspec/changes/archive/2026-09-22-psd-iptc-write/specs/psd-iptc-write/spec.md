## ADDED Requirements

### Requirement: IPTC-IIM records can be set, removed, and encoded

`pictura-codec` SHALL expose `Iptc::set(record: u8, dataset: u8, value: &[u8])
-> bool`, `Iptc::remove(record: u8, dataset: u8) -> bool`, and
`encode_iptc(&Iptc) -> Vec<u8>`. `set` SHALL replace the value of an existing
`record:dataset` in its stored position, or append a new record, and SHALL
return whether the record set changed. `remove` SHALL drop a `record:dataset`
and return whether one was present. `encode_iptc` SHALL be the exact inverse of
`parse_iptc`: each record is the `0x1C` marker, the record number, the dataset
number, a big-endian `u16` length, and the value bytes, concatenated in order. A
value longer than the `u16` length SHALL be truncated to its first `65535` bytes
so the stream stays parseable and no following record is lost.

#### Scenario: Setting an existing record replaces it in place

- **WHEN** `set` is called for a `record:dataset` already present
- **THEN** its value is replaced and its position among the records is unchanged

#### Scenario: Setting an absent record appends it

- **WHEN** `set` is called for a `record:dataset` not present
- **THEN** a new record is appended and `set` returns true

#### Scenario: Removing a record drops it

- **WHEN** `remove` is called for a present `record:dataset`
- **THEN** the record is gone from `records` and `remove` returns true; for an absent one it returns false and changes nothing

#### Scenario: Encode then parse round-trips

- **WHEN** a decoded `Iptc` is re-encoded with `encode_iptc` and parsed again
- **THEN** the two record lists are equal

#### Scenario: An over-long value cannot corrupt the stream

- **WHEN** a record whose value exceeds 65535 bytes is encoded
- **THEN** the value is truncated to 65535 bytes and every following record still parses

### Requirement: A document's IPTC core fields can be updated in place

`pictura-codec` SHALL expose `set_iptc_fields(document: &mut Document, fields:
&[(u8, u8, String)]) -> bool` that decodes the document's image resources, parses
resource 1028 (or starts empty), applies each field with `Iptc::set` (or
`Iptc::remove` when the value is empty), frames a new resource 1028 block
carrying the encoded IIM, replaces the existing block or appends it, and
reassigns `Document.image_resources`. Every other resource SHALL keep its
original raw bytes. The function SHALL return whether the section changed, SHALL
be a no-op returning false when every field already has its requested value, and
SHALL be a no-op returning false when the preserved section does not decode
losslessly (so a re-encode cannot drop bytes the parser could not frame),
leaving `Document.image_resources` untouched. Clearing a field that is already
absent or already empty SHALL be a no-op.

#### Scenario: An edit updates resource 1028 and preserves the others

- **WHEN** `set_iptc_fields` is called with a new Object Name on a document that also carries EXIF and XMP resources
- **THEN** `read_metadata`'s IPTC Object Name is the new value and the EXIF and XMP resources are byte-identical to before

#### Scenario: An empty value removes the record

- **WHEN** `set_iptc_fields` is called with an empty value for a field that is present
- **THEN** the field is absent from `read_metadata`'s IPTC afterwards

#### Scenario: An unchanged field is a no-op

- **WHEN** `set_iptc_fields` is called with values the document already has
- **THEN** it returns false and the image-resource section is unchanged

#### Scenario: A malformed section is left untouched

- **WHEN** the image-resource section ends in bytes that do not decode losslessly and a field is edited
- **THEN** `set_iptc_fields` returns false and `Document.image_resources` is unchanged

#### Scenario: Clearing an already-empty record is a no-op

- **WHEN** a field that is present with an empty value is cleared
- **THEN** `set_iptc_fields` returns false

### Requirement: An edited document's IPTC survives a save

The system SHALL commit a test that edits a fixture document's IPTC core fields
with `set_iptc_fields`, writes it with `write_psd`, re-reads it, and asserts the
edited values are present and every untouched resource survives; a second check
SHALL confirm the independent `exiftool` decoder reads the edited values from the
saved file. The oracle SHALL self-skip with a clear message when `exiftool` is
unavailable and SHALL NOT fail the suite in that case.

#### Scenario: Edit, save, re-read keeps the edit

- **WHEN** a fixture is edited, saved, and read back
- **THEN** `read_metadata`'s IPTC Object Name and By-line are the edited values and the document's other image resources are unchanged

#### Scenario: exiftool agrees on the saved edit

- **WHEN** `exiftool` decodes the saved file
- **THEN** its IPTC Object Name, By-line, and Copyright Notice match the edited values

#### Scenario: The oracle self-skips without exiftool

- **WHEN** the `exiftool` binary is not available
- **THEN** the oracle reports a skip and the suite passes
