## MODIFIED Requirements

### Requirement: Composite image read with raw and RLE compression
The system SHALL skip the color-mode data and image-resource sections by their
declared lengths, read the image-data section, and decode the composite color
planes for compression `0` (raw), `1` (PackBits RLE), `2` (ZIP), and `3`
(ZIP-with-prediction), returning `PsdError::Unsupported` for any other
compression method. ZIP is a zlib-framed deflate stream (a raw-deflate stream
SHALL also be accepted); ZIP-with-prediction additionally SHALL invert the
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

#### Scenario: Reject an unsupported compression method
- **WHEN** the image-data compression method is none of 0, 1, 2, or 3
- **THEN** it returns `PsdError::Unsupported`

## ADDED Requirements

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
