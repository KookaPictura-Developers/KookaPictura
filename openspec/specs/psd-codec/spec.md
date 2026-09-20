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
section, and a PackBits-RLE (compression `1`) composite image-data section;
reading the output with `read_psd` SHALL produce a document equal to the input.

#### Scenario: Round-trip RGB and Grayscale

- **WHEN** a document is written with `write_psd` and read back with `read_psd`
- **THEN** the resulting document equals the original

#### Scenario: Randomized composite round-trip

- **WHEN** a fixed-seed sequence of random small RGB and Grayscale documents is written and read back
- **THEN** every document round-trips equal

#### Scenario: Preserved header sections are re-emitted

- **WHEN** a document read from a file with non-empty color-mode-data or image-resource sections is written
- **THEN** the output carries those same bytes in their sections

#### Scenario: A constructed document matches the RLE golden

- **WHEN** a default document with empty preservation storage is written
- **THEN** the output is byte-identical to the refreshed fixed PackBits-RLE golden fixture (the `default_document_bytes_are_unchanged` baseline updated by this change)

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

### Requirement: Engine-encoded PSD channels are written with PackBits RLE

`write_psd` SHALL encode every channel it owns with compression `1` (PackBits
RLE): the merged composite's color planes and any document extra channels in the
image-data section, and every engine-encoded layer channel — the layer's color
channels and its raster mask (`-2`). The encoding SHALL use the standard PSD
layout: the 2-byte compression word, then one 2-byte big-endian byte count per
scanline (PSD widths, so `u16` counts), then the PackBits-encoded rows in
channel-major, row-major order. Channels preserved verbatim on
`Layer.raw_channels` (unknown compressed streams) SHALL be re-emitted
byte-for-byte and SHALL NOT be re-encoded. The encoding SHALL be deterministic.
A row whose PackBits encoding would exceed the `u16` byte-count limit SHALL
return a `PsdError` and SHALL NOT be truncated; for the PSD maximum width this
limit is not reachable.

#### Scenario: Composite declares compression 1

- **WHEN** an 8-bit RGB or Grayscale document is written with `write_psd`
- **THEN** the image-data section's 2-byte compression word is `1` and `read_psd` recovers the composite pixels exactly

#### Scenario: Layer color channel and raster mask use RLE

- **WHEN** a document with a pixel layer and a raster mask is written
- **THEN** the layer's channel records declare compression `1` and `read_psd` reconstructs the channel and mask bytes exactly

#### Scenario: Document extra channels use RLE

- **WHEN** a document carrying a saved-selection extra channel is written
- **THEN** the extra plane shares the image-data section's compression `1` and round-trips unchanged

#### Scenario: Preserved verbatim channels are re-emitted byte-for-byte

- **WHEN** a layer carries a `Layer.raw_channels` stream with its own compression header
- **THEN** read→write→read preserves that stream's exact bytes and the encoder does not re-encode it

#### Scenario: RLE write is lossless

- **WHEN** a document read from disk is written and read back
- **THEN** the decoded pixel data of the composite, extra channels, layer channels, and raster mask equals the input

#### Scenario: RLE output is deterministic and standard-reader readable

- **WHEN** the same document is written twice, or a written document is opened by the `psd-tools` oracle
- **THEN** the two outputs are byte-identical and `psd-tools` decodes the composite and channels to the same pixels

#### Scenario: Oversized scanline errors instead of truncating

- **WHEN** a scanline's PackBits encoding exceeds the `u16` byte-count limit
- **THEN** `write_psd` returns a `PsdError` and writes no truncated count

### Requirement: PSB write

`write_psd` SHALL emit a version-1 PSD for a document whose width and height are
each at most 30 000 (`MAX_DIM_PSD`) and whose source was a PSD, leaving its
output bytes unchanged, and SHALL emit a version-2 PSB when the source document
was a PSB (`Document.is_psb`) or when either dimension exceeds 30 000, because a
PSD can neither represent those dimensions nor carry the widened length fields a
source PSB's preserved tagged blocks require. A new
`write_psb(doc: &Document) -> Result<Vec<u8>, PsdError>` SHALL always emit a
version-2 PSB; both entry points SHALL share one container writer. A PSB SHALL
write the header version word `2` and SHALL use 8-byte (`u64`) big-endian fields
for the layer-and-mask section length, the layer-info length, and each layer
record's per-channel data length, and 4-byte (`u32`) big-endian entries for the
RLE scanline byte-count table in the composite image-data section and in every
layer channel and mask stream. The global layer-mask info length SHALL remain a
4-byte (`u32`) field in both containers. A per-layer additional-layer-information
(tagged) block SHALL declare an even length with the pad byte included in that
length, so a pad byte is never emitted outside the declared length. A
document-level (global) additional-layer-information block SHALL declare its
exact data length and SHALL be padded externally to a 4-byte boundary. In a PSB,
such a block whose key is a PSB big key (the psd-tools `_BIG_KEYS` set,
including `lnk2`/`lnk3`/`lnkE`, `Lr16`/`Lr32`/`Layr`, `LMsk`, `Alph`,
`FMsk`, `PxSD`, `pths`, `Mtrn`/`Mt16`/`Mt32`, `cinf`, `extd`/`extn`, `artd`,
`FXid`/`FEid`/`FELS`) SHALL write an 8-byte length; non-big keys SHALL stay
4 bytes in both containers. A preserved document-level tagged block SHALL be
re-framed to the output container's length width (8 bytes in a PSB, 4 bytes in a
PSD) and re-padded to a 4-byte boundary before it is emitted. The `iOpa`
blend-fill-opacity block SHALL be written as a 4-byte block (`[fill, 0, 0, 0]`),
matching the Photoshop `B3x` layout. Dimensions SHALL be
accepted up to 300 000
(`MAX_DIM_PSB`), and a width or height above 300 000 SHALL return
`PsdError::Unsupported`. RLE (compression `1`) SHALL remain the only compression
written. A stream preserved on `Layer.raw_channels` SHALL be re-emitted
byte-for-byte with its own compression header, widening only its declared length
field, and repeated writes of the same document SHALL be byte-identical.

#### Scenario: Automatic PSB selection above the PSD limit

- **WHEN** `write_psd` is given a document whose width or height is above 30 000 and at most 300 000
- **THEN** the output header version word is `2` and `read_psd` recovers a document equal to the input

#### Scenario: write_psb always emits version 2

- **WHEN** `write_psb` is given a document within the PSD dimension limit
- **THEN** the output header version word is `2` and `read_psd` recovers a document equal to the input

#### Scenario: A PSB source re-saves as a PSB

- **WHEN** a document read from a version-2 PSB (`Document.is_psb` true) with both dimensions at most 30 000 is written with `write_psd`
- **THEN** the output is a version-2 PSB whose bytes round-trip through `read_psd` equal to the source document

#### Scenario: PSB length fields widen and RLE counts widen

- **WHEN** a PSB is written for a document with a layer and a mask
- **THEN** the layer-and-mask section length, the layer-info length, and each layer channel data length are 8-byte fields, each RLE scanline byte-count entry is 4 bytes in both the composite and the layer/mask streams, and the global layer-mask info length stays 4 bytes

#### Scenario: Big-key block length widens in a PSB

- **WHEN** a PSB is written for a layer carrying a preserved additional-layer-information block with a PSB big key (for example `Lr16` or `lnk2`)
- **THEN** that block's length field is 8 bytes, and the block round-trips through `read_psd` equal to the input

#### Scenario: PSB with an authored smart object is readable by psd-tools

- **WHEN** a document with an embedded smart object (authored by the writer) is serialized with `write_psb`
- **THEN** psd-tools opens the file and parses its document-level `lnk2` big-key block without a framing error

#### Scenario: An odd-length block does not corrupt following blocks

- **WHEN** a PSB is written for a layer carrying an odd-length additional-layer-information block followed by a later block
- **THEN** the odd block declares an even length with the pad byte inside that length and psd-tools reads the later block

#### Scenario: A document-level block declares its exact length and pads externally to 4

- **WHEN** a PSB is written for a document carrying a preserved document-level tagged block whose data length is not a multiple of 4
- **THEN** the block declares that exact length and is padded externally to a 4-byte boundary, and psd-tools reads the written file including a later block

#### Scenario: A PSD-sourced document with a preserved big-key block is readable when written as a PSB

- **WHEN** `write_psb` is given a document read from a PSD whose preserved document-level `lnk2` block has a 4-byte length
- **THEN** the `lnk2` block is re-framed with an 8-byte length and psd-tools reads the written file

#### Scenario: Large-dimension round-trip

- **WHEN** a document wider than 30 000 (for example 30 001×1) is written and read back
- **THEN** the resulting document equals the original

#### Scenario: Preserved verbatim channel length widens

- **WHEN** a document carrying a `Layer.raw_channels` stream is written as a PSB
- **THEN** the stream bytes are emitted byte-for-byte with their own compression header and only the declared length field is 8 bytes

#### Scenario: Dimension above the PSB limit errors

- **WHEN** `write_psb` is given a document whose width or height exceeds 300 000
- **THEN** it returns `PsdError::Unsupported` and does not panic

