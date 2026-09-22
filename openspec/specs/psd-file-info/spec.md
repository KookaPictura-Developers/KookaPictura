# psd-file-info Specification

## Purpose
TBD - created by archiving change psd-file-info. Update Purpose after archive.
## Requirements
### Requirement: EXIF image resources are decoded into ordered typed tags

`pictura-codec` SHALL expose `parse_exif(data: &[u8]) -> Exif` that decodes a
PSD EXIF image resource into ordered `(tag, value)` entries. The input MAY carry
the 6-byte `Exif\0\0` header or be a bare TIFF stream; both SHALL be accepted. The
decoder SHALL read the `II`/`MM` byte order
and the TIFF magic, SHALL walk IFD0 and the Exif sub-IFD (`0x8769`), and SHALL
decode the TIFF value types ASCII (`2`), SHORT (`3`), LONG (`4`), RATIONAL
(`5`), and UNDEFINED (`7`), reading values longer than four bytes from their
offset. A SHORT, LONG, or RATIONAL tag whose `count` exceeds 1 SHALL be exposed
as its raw `Undefined` bytes rather than as a component list. The returned
`Exif` SHALL expose `get(tag: u16) -> Option<&ExifValue>`
and iterate entries in tag order. The decoder SHALL bound the entry count per
IFD and the total value bytes cloned to the resource size, SHALL NOT recurse,
and SHALL NOT panic: a truncated or malformed blob
SHALL yield the entries decoded so far.

#### Scenario: The fixture's EXIF decodes to its stored tags

- **WHEN** `parse_exif` decodes the fixture's EXIF resource
- **THEN** `get` returns the Make, Model, Software, and DateTime from IFD0 and the ExposureTime, FNumber, ISO, DateTimeOriginal, and FocalLength from the Exif sub-IFD

#### Scenario: Both the Exif-prefixed and bare-TIFF forms are accepted

- **WHEN** the same TIFF stream is passed with and without the `Exif\0\0` prefix
- **THEN** both decode to the same entries

#### Scenario: A big-endian stream is decoded

- **WHEN** the TIFF header is `MM` (big-endian)
- **THEN** tags and values are decoded big-endian

#### Scenario: A truncated EXIF blob does not panic

- **WHEN** the blob is cut short inside an IFD or an offset value
- **THEN** `parse_exif` returns the entries decoded so far and does not panic

### Requirement: IPTC-IIM image resources are decoded into ordered records

`pictura-codec` SHALL expose `parse_iptc(data: &[u8]) -> Iptc` that decodes a
PSD IPTC-NAA image resource (id 1028) into ordered records. A record SHALL be
read as the `0x1C` marker, a record number, a dataset number, a big-endian
`u16` length, and that many value bytes, per the IPTC-IIM stream format. The
returned `Iptc` SHALL expose `get(record: u8, dataset: u8) -> Option<&[u8]>`
and a text accessor that decodes a value as UTF-8 (lossy). The decoder SHALL
stop at the first malformed or truncated record and return the records decoded
so far, SHALL bound the record count, and SHALL NOT panic.

#### Scenario: The fixture's IPTC decodes to its core fields

- **WHEN** `parse_iptc` decodes the fixture's IPTC resource
- **THEN** the Object Name (`2:5`), By-line (`2:80`), Copyright Notice (`2:116`), and Caption-Abstract (`2:120`) values are present in record order

#### Scenario: A truncated record stops parsing without panicking

- **WHEN** a record's declared length exceeds the remaining bytes
- **THEN** `parse_iptc` returns the records decoded before it and does not panic

### Requirement: The XMP packet is exposed as raw text

`read_metadata` SHALL expose the document's XMP image resource (id 1060) as a
UTF-8 (lossy) string, without resolving XML entities, so XXE and entity-expansion
payloads have no effect. `pictura-codec` SHALL additionally expose the packet's
fixed typed properties (title, creator, description, subject, rights, credit,
source, headline, marked) parsed from the same resource, per
`psd-xmp-metadata`. The raw string is unchanged by the addition.

#### Scenario: The fixture's XMP packet is returned as text

- **WHEN** `read_metadata` reads the fixture
- **THEN** its XMP string equals the stored resource 1060 bytes decoded as text

#### Scenario: The parsed properties accompany the raw string

- **WHEN** the fixture's XMP is read
- **THEN** the parsed title, creator, description, and rights are available alongside the raw string

### Requirement: Metadata is gathered from a document's image resources

`pictura-codec` SHALL expose `read_metadata(document: &Document) ->
DocumentMetadata` that gathers the decoded EXIF (resource 1058 or 1059), IPTC
(1028), and XMP (1060) from the document's preserved image-resource section. A
missing resource SHALL yield an empty value, not an error.

#### Scenario: The fixture yields EXIF, IPTC, and XMP

- **WHEN** `read_metadata` reads the metadata fixture
- **THEN** its EXIF and IPTC are non-empty and its XMP string is non-empty

#### Scenario: A document with no metadata resources yields empty metadata

- **WHEN** `read_metadata` reads a document whose image-resource section carries no EXIF, IPTC, or XMP block
- **THEN** its EXIF and IPTC are empty and its XMP string is empty

### Requirement: The metadata fixture is proven by an independent decoder

The system SHALL commit the fixture
`crates/pictura-codec/tests/fixtures/metadata.psd`, regenerated byte-stably by
`scripts/generate-fixtures.py`, carrying a real EXIF IFD, an IPTC-IIM block, and
an XMP packet, and a test SHALL verify that `read_metadata`'s decoded EXIF and
IPTC values agree with the independent `exiftool` decoder for the same file. The
oracle SHALL self-skip with a clear message when `exiftool` is unavailable and
SHALL NOT fail the suite in that case.

#### Scenario: read_metadata agrees with exiftool

- **WHEN** `read_metadata` decodes `metadata.psd` and `exiftool` decodes the same file
- **THEN** the decoded Make, Model, Software, ExposureTime, FNumber, ISO, DateTimeOriginal, and FocalLength, and the IPTC Object Name, By-line, Copyright Notice, and Caption-Abstract match

#### Scenario: The oracle self-skips without exiftool

- **WHEN** the `exiftool` binary is not available
- **THEN** the oracle reports a skip and the suite passes

### Requirement: The application edits IPTC core fields in File Info

The application SHALL enable `File > File Info…` when a document is open and
SHALL open a dialog listing the decoded metadata under the categories Camera
Data (EXIF), Description (the parsed XMP properties), IPTC, and Raw Data (the raw
XMP packet). The Camera Data, Description, and Raw Data categories SHALL be
read-only. The Description category SHALL list the parsed XMP properties. The
IPTC category SHALL present the six IPTC-Core fields — Object Name, By-line,
Copyright Notice, Caption/Abstract, Credit, and Source — as editable text
prefilled from the active document, plus a read-only list of the remaining IPTC
records. On OK the application SHALL apply only the fields
the user changed (so an untouched value is never rewritten), write each changed
field to both the XMP packet and the IIM record so the two never conflict, record
exactly one undo state labelled for the edit, and mark the document dirty; OK
with no changed fields SHALL apply nothing and record no state; on Cancel it
SHALL apply nothing. The bridge SHALL expose `iptc_edit_fields()` returning
`"record:dataset\tLabel\tValue"` rows, `xmp_rows()` returning the parsed XMP
properties as label/value rows, and an apply method taking `"record:dataset\tValue"`
rows.

#### Scenario: File Info is disabled without a document

- **WHEN** no document is open
- **THEN** the `File > File Info…` command is disabled

#### Scenario: Opening File Info shows the editable categories

- **WHEN** the metadata fixture is open and `File > File Info…` is invoked
- **THEN** the dialog shows the Camera Data, Description, IPTC, and Raw Data categories, the Camera Data category lists the decoded EXIF Make, the Description category lists the parsed XMP title read-only, and the IPTC category shows the document's Object Name in an editable field

#### Scenario: Editing a field and accepting applies one undo state

- **WHEN** the Object Name field is changed and the dialog is accepted
- **THEN** the document's decoded IPTC Object Name and parsed XMP title are both the new value, the document is dirty, and one Undo restores the previous value in both

#### Scenario: Cancelling applies nothing

- **WHEN** the Object Name field is changed and the dialog is rejected
- **THEN** the document's decoded IPTC Object Name and parsed XMP title are unchanged and the document is not dirtied by the dialog

#### Scenario: Accepting with no changes records nothing

- **WHEN** the dialog is accepted without changing any field
- **THEN** no edit is applied, no undo state is recorded, and the document is not dirtied by the dialog

