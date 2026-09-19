## ADDED Requirements

### Requirement: Embedded smart-object source resolution
The system SHALL resolve, for a layer carrying a smart-object config descriptor,
the document-level linked source record whose `uuid` matches the descriptor, and
SHALL expose the matched record's embedded payload, filename, and filetype. A
matched record whose kind is external or alias SHALL be reported as
non-embedded without reading any path from disk.

#### Scenario: Embedded source resolved
- **WHEN** a layer's smart-object descriptor `uuid` matches an embedded linked record
- **THEN** `read_psd` exposes that record's payload bytes, filename, and filetype on the layer

#### Scenario: External source is not read
- **WHEN** the matching linked record is external
- **THEN** the source is reported as non-embedded and no file is read from disk

#### Scenario: Unresolved object does not fail
- **WHEN** a layer carries a smart-object descriptor with no matching linked record
- **THEN** `read_psd` succeeds and the block stays preserved as unresolved

### Requirement: Smart-object blocks round-trip byte-for-byte
The system SHALL preserve through read and write the smart-object config
descriptors (`SoLd`/`SoLE`, legacy `plLd`), the document-level linked records
(`lnkD`/`lnk2`/`lnk3`/`lnkE`), the embedded payload, and the `uuid` link, so that
a read→write→read round-trip reproduces them exactly.

#### Scenario: Embedded smart object survives
- **WHEN** a PSD with an embedded smart object is read and then written
- **THEN** the config descriptor, linked record, payload, and `uuid` are unchanged after re-reading

#### Scenario: Payload integrity
- **WHEN** the embedded payload is non-empty
- **THEN** its bytes after read→write→read equal the original

#### Scenario: External record preserved
- **WHEN** a document carries an external linked record
- **THEN** read→write→read preserves the record without dereferencing it

#### Scenario: Unrecognized descriptor version is preserved
- **WHEN** a layer carries a `SoLE` descriptor or an unrecognized descriptor block version
- **THEN** the preserved block re-emits unchanged and no error is raised

### Requirement: Embedded smart-object authoring
Given an embedded source payload and a composite, the writer SHALL emit a
smart-object layer with a config descriptor and a linked source record whose
`uuid` matches, and SHALL place the payload as the record's embedded data. When
authoring a new object with no input descriptor, the emitted descriptor SHALL be
`SoLd` with outer version 4 and descriptor block version 16, so Photoshop CS6 can
reopen it. The written file SHALL be readable by the codec and by an independent
PSD reader as a smart object carrying the same payload.

#### Scenario: Author and re-read
- **WHEN** a document with an embedded source is written and read back
- **THEN** the smart object resolves with the same payload and filename

#### Scenario: Independent reader agrees
- **WHEN** the written file is parsed by `psd-tools`
- **THEN** it is reported as a smart object whose embedded data equals the payload

#### Scenario: No payload corruption
- **WHEN** the payload is a fixed-seed random byte string
- **THEN** its hash after write and read is unchanged

### Requirement: Camera Raw crs settings are preserved
The system SHALL expose the `crs:` XMP settings associated with an embedded
source on read and SHALL re-emit them byte-exact on write. The system SHALL NOT
provide an API to edit `crs:` settings; the packet is preserve-only.

#### Scenario: Settings preserved
- **WHEN** an embedded source carries `crs:` settings
- **THEN** read exposes them and a write re-emits them byte-exact

#### Scenario: No edit API
- **WHEN** a caller wants to change a `crs:` setting
- **THEN** no edit API exists and the packet is preserved unchanged

### Requirement: Adobe Photoshop interop is fixture-validated
The system SHALL round-trip Photoshop-produced reference PSDs containing an
embedded smart object so that the config descriptor, linked record, and payload
are preserved. The interop acceptance SHALL be the `psd-tools` oracle parsing
the written file; a manual Photoshop CC reopen MAY be recorded as a deferred
follow-up where Photoshop is available. Reference fixtures SHALL be
self-produced and their provenance recorded.

#### Scenario: Photoshop embedded smart object round-trips
- **WHEN** `test_with_smart_object01.psd` is read and written
- **THEN** the smart object parses identically in the output and its payload is unchanged

#### Scenario: Shared source across layers
- **WHEN** two layers reference the same linked record by `uuid`
- **THEN** both resolve to the same payload and the output preserves the single record

#### Scenario: Automated interop acceptance
- **WHEN** the written file is parsed by psd-tools
- **THEN** the smart object, its uuid, and its embedded payload match the source fixture
