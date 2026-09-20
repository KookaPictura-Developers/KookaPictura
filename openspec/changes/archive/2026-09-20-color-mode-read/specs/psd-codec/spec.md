## MODIFIED Requirements

### Requirement: Header validation for the composite subset
The system SHALL parse the channel count, height, width, bit depth, and color
mode, and SHALL accept 1–56 channels, non-zero dimensions within the PSD/PSB
dimension limit, bit depth 8 (or bit depth 1 only for Bitmap mode), and color
mode Bitmap (0), Grayscale (1), Indexed (2), RGB (3), CMYK (4), or Lab (9). It
SHALL return `PsdError::Unsupported` for Multichannel (7), Duotone (8), any
other color-mode code, and any bit depth other than 8 (other than depth 1 for
Bitmap); it SHALL return `PsdError::Invalid` for a channel count or dimension
outside the accepted range.

#### Scenario: Zero dimension
- **WHEN** the header declares a width or height of zero
- **THEN** it returns `PsdError::Invalid`

#### Scenario: Depth other than 8-bit
- **WHEN** the header declares a bit depth other than 8 for a mode other than Bitmap, or a bit depth other than 1 for Bitmap
- **THEN** it returns `PsdError::Unsupported`

#### Scenario: Bitmap depth 1 is accepted
- **WHEN** the header declares Bitmap mode with bit depth 1
- **THEN** the header is accepted and the composite is decoded as 1-bit rows

#### Scenario: Channel count out of range
- **WHEN** the header declares zero channels or more than 56 channels
- **THEN** it returns `PsdError::Invalid`

#### Scenario: A non-RGB color mode is accepted
- **WHEN** the header declares Indexed (2), CMYK (4), or Lab (9)
- **THEN** the header is accepted and the document is normalized to RGB

#### Scenario: Multichannel and Duotone are unsupported
- **WHEN** the header declares Multichannel (7) or Duotone (8)
- **THEN** it returns `PsdError::Unsupported`

### Requirement: Composite image read with raw and RLE compression
The system SHALL read the color-mode data and image-resource sections by their
declared lengths and SHALL retain their bytes verbatim on the document for
lossless re-save (instead of discarding them), except that a color-mode-data
section belonging to an Indexed document SHALL be interpreted as the 768-byte
palette (see `psd-color-modes`) and SHALL NOT be retained on the normalized
document. It SHALL read the image-data section and decode the composite color
planes for compression `0` (raw), `1` (PackBits RLE), `2` (ZIP), and `3`
(ZIP-with-prediction), returning `PsdError::Unsupported` for any other
compression method. ZIP is a zlib-framed deflate stream (a raw-deflate stream
SHALL also be accepted); ZIP-with-prediction additionally SHALL invert the
byte-wise per-scanline delta after inflating. For a depth-1 Bitmap document the
raw and RLE row stride SHALL be `ceil(width / 8)` bytes per row and ZIP
compression SHALL be `PsdError::Unsupported`. A malformed deflate stream SHALL
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

#### Scenario: Read a depth-1 Bitmap RLE composite
- **WHEN** a depth-1 Bitmap document stores compression 1
- **THEN** each scanline's RLE decode writes `ceil(width / 8)` bytes and the row expands to black and white pixels

#### Scenario: Color-mode and image-resource bytes are retained
- **WHEN** a non-Indexed PSD carries a non-empty color-mode-data or image-resource section
- **THEN** those exact bytes are exposed on the returned document

#### Scenario: An Indexed palette is interpreted, not retained
- **WHEN** an Indexed PSD carries a 768-byte color-mode-data palette
- **THEN** the palette expands the composite to RGB and the normalized document's color-mode-data is empty

#### Scenario: Reject an unsupported compression method
- **WHEN** the image-data compression method is none of 0, 1, 2, or 3
- **THEN** it returns `PsdError::Unsupported`

#### Scenario: Reject ZIP at depth 1
- **WHEN** a depth-1 Bitmap document declares compression 2 or 3
- **THEN** it returns `PsdError::Unsupported`
