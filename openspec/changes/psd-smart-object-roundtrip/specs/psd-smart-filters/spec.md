## ADDED Requirements

### Requirement: Smart-filter blocks round-trip byte-for-byte
The system SHALL preserve through read and write a smart-object layer's
`filterFX` list inside its `SoLd` descriptor, the document-level filter-effects
blocks (`FEid`/`FXid`), and the filter mask (`FMsk`), so that a read→write→read
round-trip reproduces them exactly.

#### Scenario: filterFX survives
- **WHEN** a smart-object layer carries a `filterFX` list
- **THEN** read→write→read preserves the list entries, their order, and their bytes

#### Scenario: Filter effects and mask survive
- **WHEN** a document carries `FEid`/`FXid` or `FMsk` blocks
- **THEN** read→write→read preserves those blocks

#### Scenario: A filter we do not model is preserved
- **WHEN** a smart filter's `filterID` is not the Camera Raw Filter
- **THEN** its options descriptor bytes are preserved unchanged

### Requirement: Camera Raw Filter settings are read and written
The system SHALL expose, for a smart filter whose `filterID` is 2683, its options
descriptor named `Fltr`, and SHALL re-emit it on write. Keys the system does not
model SHALL be preserved. The `Fltr` keys SHALL be read with the mapping
recorded in `docs/dev/camera-raw-cc-notes.md`.

#### Scenario: Settings exposed
- **WHEN** a layer carries a Camera Raw Filter with a `Fltr` descriptor
- **THEN** read exposes the filter options keyed by their short names

#### Scenario: Settings edit round-trips
- **WHEN** a Camera Raw Filter option is changed through the API
- **THEN** the written `Fltr` reflects the change and unmodeled keys are retained

#### Scenario: Filter identity is preserved
- **WHEN** a Camera Raw Filter is written
- **THEN** its `filterID` is 2683 and its display name is present in the entry

### Requirement: Smart-filter interop is fixture-validated
The system SHALL round-trip the supplied Photoshop CC reference PSD containing a
Camera Raw Filter so that the smart object, the `filterFX` block, the `Fltr`
settings, and the `FEid`/`FMsk` blocks are preserved, and SHALL record a manual
Photoshop CC reopen as the interop acceptance.

#### Scenario: Camera Raw Filter fixture round-trips
- **WHEN** `test_with_smart_object02.psd` is read and written
- **THEN** the smart-filter blocks and `Fltr` settings parse identically in the output

#### Scenario: Manual reopen recorded
- **WHEN** the fixture oracle is in place
- **THEN** a manual check that Photoshop CC reopens the written file with the smart object and Camera Raw Filter editable is recorded in the task result

#### Scenario: Sixteen-bit variant is deferred
- **WHEN** a 16-bit fixture is produced
- **THEN** it is added to the oracle once roadmap G4 depth support lands
