# psd-codec Specification

## Purpose
TBD - created by archiving change m0-walking-skeleton. Update Purpose after archive.
## Requirements
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
The system SHALL read the color-mode data and image-resource sections by their
declared lengths and SHALL retain their bytes verbatim on the document for
lossless re-save (instead of discarding them), read the image-data section, and
decode the composite color planes for compression `0` (raw), `1` (PackBits RLE),
`2` (ZIP), and `3` (ZIP-with-prediction), returning `PsdError::Unsupported` for
any other compression method. ZIP is a zlib-framed deflate stream (a raw-deflate
stream SHALL also be accepted); ZIP-with-prediction additionally SHALL invert the
byte-wise per-scanline delta after inflating. A malformed deflate stream SHALL
return a typed error, never a panic.

#### Scenario: Read a raw RGB composite
- **WHEN** an RGB PSD stores the image data section with compression 0
- **THEN** `read_psd` returns a document whose composite planes equal the stored bytes

#### Scenario: Read a PackBits RLE composite
- **WHEN** the image-data section uses compression 1 with per-scanline byte counts
- **THEN** `read_psd` decodes the PackBits literal and repeat runs into the composite planes

#### Scenario: Read a ZIP composite
- **WHEN** the image-data section uses compression 2 and holds a zlib deflate stream
- **THEN** `read_psd` decodes the composite planes from the inflated bytes

#### Scenario: Read a ZIP-with-prediction composite
- **WHEN** the image-data section uses compression 3
- **THEN** `read_psd` inflates the stream and inverts the byte-wise per-row delta to recover the planes

#### Scenario: Color-mode and image-resource bytes are retained
- **WHEN** a PSD carries a non-empty color-mode-data or image-resource section
- **THEN** those exact bytes are exposed on the returned document

#### Scenario: Reject an unsupported compression method
- **WHEN** the image-data compression method is none of 0, 1, 2, or 3
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
(version 1) containing the header, the document's color-mode-data and
image-resource bytes (empty for a freshly constructed document), the layer/mask
section, and a raw composite image-data section; reading the output with
`read_psd` SHALL produce a document equal to the input.

#### Scenario: Round-trip RGB and Grayscale
- **WHEN** a document is written with `write_psd` and read back with `read_psd`
- **THEN** the resulting document equals the original

#### Scenario: Randomized composite round-trip
- **WHEN** a fixed-seed sequence of random small RGB and Grayscale documents is written and read back
- **THEN** every document round-trips equal

#### Scenario: Preserved header sections are re-emitted
- **WHEN** a document read from a file with non-empty color-mode-data or image-resource sections is written
- **THEN** the output carries those same bytes in their sections

#### Scenario: A constructed document is unchanged
- **WHEN** a default document with empty preservation storage is written
- **THEN** the output is byte-identical to the pre-preservation serialization

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

### Requirement: Absent merged composite
The system SHALL still return a document carrying the parsed layer tree when the
image-data section is absent (a document saved with "Maximize Compatibility"
off), synthesizing a zero-filled composite of the header dimensions and mode
rather than failing with a truncation error. A header whose data ends before the
layer section SHALL still return an error.

#### Scenario: Layers parse without a composite
- **WHEN** a layered PSD's bytes end after the layer-and-mask section
- **THEN** `read_psd` returns the layers and a zero-filled composite of the header size

#### Scenario: Truncation before the layers is still an error
- **WHEN** the file ends before the layer-and-mask section completes
- **THEN** `read_psd` returns a typed error

