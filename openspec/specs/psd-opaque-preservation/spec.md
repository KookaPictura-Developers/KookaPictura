# psd-opaque-preservation Specification

## Purpose
TBD - created by archiving change psd-opaque-preservation. Update Purpose after archive.
## Requirements
### Requirement: Opaque PSD blocks survive an open-save round trip
Reading a PSD SHALL retain, byte-for-byte, every section and block the engine
does not interpret, and writing a document SHALL re-emit those bytes so that an
open→save round trip loses nothing. The preserved data SHALL include the
color-mode-data section, the image-resource section, the global layer mask, the
trailing global additional-layer information, each layer's original blend key
when it is not a recognized mode, each layer's blending-ranges bytes, each
layer's unknown additional-layer-info tagged blocks, each layer's unmodeled
channels (for example the `-3` real user mask), and each layer mask's trailing
parameter bytes.

#### Scenario: Image resources and color-mode data round-trip
- **WHEN** a document read from a file carries a non-empty image-resource or color-mode-data section and is written back
- **THEN** reading the output yields the same resource and color-mode-data bytes

#### Scenario: Unknown tagged blocks round-trip
- **WHEN** a layer carries additional-layer-info blocks the engine does not model
- **THEN** those blocks are present with identical keys and payloads after read→write→read

#### Scenario: An unmodeled channel round-trips
- **WHEN** a layer carries a `-3` real-user-mask channel
- **THEN** the channel's bytes are present after read→write→read

#### Scenario: An unknown blend key round-trips
- **WHEN** a layer's blend key is not one of the recognized modes
- **THEN** the original 4-byte key is written back after read→write→read

#### Scenario: Blending ranges and mask parameters round-trip
- **WHEN** a layer carries non-empty blending ranges or a mask block longer than the fixed fields
- **THEN** those bytes are present after read→write→read

### Requirement: The document model defaults to empty preservation
A document created by the engine (not read from a file) SHALL have empty
preservation storage, so `write_psd` output for it is unchanged from before
preservation existed.

#### Scenario: A constructed document serializes as before
- **WHEN** a default document is written
- **THEN** its bytes equal the pre-preservation serialization for the same document

#### Scenario: Preservation does not affect unrelated edits
- **WHEN** a file with preserved blocks is read, an unrelated layer is edited, and it is written
- **THEN** the preserved blocks of the untouched layers are still emitted

