## MODIFIED Requirements

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
