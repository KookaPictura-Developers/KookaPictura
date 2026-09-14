## ADDED Requirements

### Requirement: PSD/PSB signature and version detection
The system SHALL read the 4-byte `8BPS` signature and SHALL reject any other
signature with `PsdError::BadSignature`; it SHALL accept version 1 (PSD) and
version 2 (PSB) and SHALL reject any other version with `PsdError::Unsupported`.

#### Scenario: Wrong signature
- **WHEN** `read_psd` is given bytes that do not start with `8BPS`
- **THEN** it returns `PsdError::BadSignature` and does not panic

#### Scenario: Unsupported version marker
- **WHEN** the version field is neither 1 nor 2
- **THEN** it returns `PsdError::Unsupported`

### Requirement: Header validation for the composite subset
The system SHALL parse the channel count, height, width, bit depth, and color
mode, and SHALL accept only 1–56 channels, non-zero dimensions within the
PSD/PSB dimension limit, 8-bit depth, and RGB or Grayscale color mode, returning
`PsdError::Invalid` or `PsdError::Unsupported` for anything else.

#### Scenario: Zero dimension
- **WHEN** the header declares a width or height of zero
- **THEN** it returns `PsdError::Invalid`

#### Scenario: Depth other than 8-bit
- **WHEN** the header declares a bit depth other than 8
- **THEN** it returns `PsdError::Unsupported`

#### Scenario: Channel count out of range
- **WHEN** the header declares zero channels or more than 56 channels
- **THEN** it returns `PsdError::Invalid`

#### Scenario: Color mode outside RGB and Grayscale
- **WHEN** the header declares a color mode that is neither Grayscale (1) nor RGB (3)
- **THEN** it returns `PsdError::Unsupported`

### Requirement: Composite image read with raw and RLE compression
The system SHALL skip the color-mode data and image-resource sections by their
declared lengths, read the image-data section, and decode the composite color
planes for compression 0 (raw) and compression 1 (PackBits RLE), returning
`PsdError::Unsupported` for any other compression method.

#### Scenario: Read a raw RGB composite
- **WHEN** an RGB PSD stores the image data section with compression 0
- **THEN** `read_psd` returns a document whose composite planes equal the stored bytes

#### Scenario: Read a PackBits RLE composite
- **WHEN** the image-data section uses compression 1 with per-scanline byte counts
- **THEN** `read_psd` decodes the PackBits literal and repeat runs into the composite planes

#### Scenario: Reject an unsupported compression method
- **WHEN** the image-data compression method is neither 0 nor 1
- **THEN** it returns `PsdError::Unsupported`

### Requirement: PSB uses 64-bit lengths and 4-byte RLE counts
The system SHALL read version-2 (PSB) files using 8-byte section lengths and
4-byte per-scanline RLE byte counts, versus 4-byte and 2-byte respectively for
version-1 (PSD) files.

#### Scenario: RLE PSB
- **WHEN** a PSB header declares compression 1
- **THEN** the scanline byte-count table is read as 4-byte entries and the composite decodes correctly

### Requirement: Composite PSD write and round-trip
The system SHALL serialize an 8-bit RGB or Grayscale `Document` to a valid PSD
(version 1) containing the header, empty color-mode and image-resource sections,
an empty layer/mask section, and a raw composite image-data section; reading the
output with `read_psd` SHALL produce a document equal to the input.

#### Scenario: Round-trip RGB and Grayscale
- **WHEN** a document is written with `write_psd` and read back with `read_psd`
- **THEN** the resulting document equals the original

#### Scenario: Randomized composite round-trip
- **WHEN** a fixed-seed sequence of random small RGB and Grayscale documents is written and read back
- **THEN** every document round-trips equal

#### Scenario: Reject an unwritable document
- **WHEN** `write_psd` is given a document that is not 8-bit or not RGB/Grayscale
- **THEN** it returns `PsdError::Unsupported`

### Requirement: Malformed input returns errors, never panics
The system SHALL validate every length and offset before use and SHALL return a
typed `PsdError`, never a panic or an out-of-bounds read, for truncated files,
impossible counts, malformed RLE runs, and size arithmetic overflow.

#### Scenario: Truncated file
- **WHEN** any required field or section is missing bytes
- **THEN** `read_psd` returns `PsdError::Truncated`

#### Scenario: PackBits run overrun
- **WHEN** an RLE control byte would decode past the scanline or past the input
- **THEN** `read_psd` returns `PsdError::Invalid`

### Requirement: Typed codec errors
The system SHALL define a `PsdError` enum with `thiserror` whose variants
distinguish bad signature, unsupported features, truncation, and invalid data.

#### Scenario: Failure class is matchable
- **WHEN** a caller matches on the error returned by `read_psd` or `write_psd`
- **THEN** `BadSignature`, `Unsupported`, `Truncated`, and `Invalid` are distinguishable
