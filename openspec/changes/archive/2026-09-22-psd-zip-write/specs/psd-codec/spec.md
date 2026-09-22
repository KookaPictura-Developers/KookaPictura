## MODIFIED Requirements

### Requirement: Composite PSD write and round-trip

The system SHALL serialize an 8-bit RGB or Grayscale `Document` to a valid PSD
(version 1) containing the header, the document's color-mode-data and
image-resource bytes (empty for a freshly constructed document), the layer/mask
section, and an image-data section whose composite is encoded with the document's
recorded composite compression (PackBits RLE by default, or ZIP or
ZIP-with-prediction when the document was read from a file using one), except
that a document whose merged composite is absent SHALL write no image-data
section; reading the output with `read_psd` SHALL produce a document equal to the
input.

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
- **THEN** its recorded compression defaults to RLE and the output is byte-identical to the refreshed fixed PackBits-RLE golden fixture

#### Scenario: A ZIP-composite document round-trips with its compression

- **WHEN** a document whose composite compression is ZIP is written and read back
- **THEN** the image-data section declares compression `2`, the composite bytes are recovered exactly, and the re-read document equals the input

#### Scenario: A document without a merged composite writes none

- **WHEN** a document read from a file that ended after the layer section is written
- **THEN** the output has no image-data section and reading it back yields an equal document with no merged composite

#### Scenario: Reject an unwritable document

- **WHEN** `write_psd` is given a document that is not 8-bit or not RGB/Grayscale
- **THEN** it returns `PsdError::Unsupported`

## REMOVED Requirements

### Requirement: Engine-encoded PSD channels are written with PackBits RLE

**Reason**: The writer no longer hardcodes PackBits RLE. It now emits the
document's recorded compression, which is RLE by default but ZIP (`2`) or
ZIP-with-prediction (`3`) when the source file used one, so an open→save keeps
the source compression and can produce ZIP output.

**Migration**: No caller change is required. A freshly constructed document
records RLE and its output is byte-identical to before; a document read from a
file records the compression it was read with and is re-emitted with that kind.
The replacement requirement below states the full behaviour, including the
verbatim re-emission of preserved `Layer.raw_channels` streams and the
`u16` PackBits-scanline error.

## ADDED Requirements

### Requirement: Engine-encoded PSD channels are written with the document's recorded compression

`write_psd` SHALL encode every channel it owns with the compression recorded on
the document: the merged composite's color planes and any document extra channels
in the image-data section use the recorded composite compression, and every
engine-encoded layer channel — the layer's color channels and its raster mask
(`-2`) — uses the recorded layer-channel compression. The supported kinds are
PackBits RLE (`1`), ZIP (`2`), and ZIP-with-prediction (`3`); a document with no
recorded kind SHALL write RLE. RLE SHALL use the standard layout: the 2-byte
compression word, one 2-byte big-endian byte count per scanline (PSD widths, so
`u16` counts), then the PackBits rows in channel-major, row-major order. ZIP SHALL
zlib-wrap the concatenated planar bytes; ZIP-with-prediction SHALL apply the
reversible per-row delta (`out[i] = data[i] - data[i-1]` within each `width`-byte
row) before deflating. Channels preserved verbatim on `Layer.raw_channels` SHALL
be re-emitted byte-for-byte and SHALL NOT be re-encoded. The encoding SHALL be
deterministic. An RLE row whose PackBits encoding would exceed the `u16`
byte-count limit SHALL return a `PsdError` and SHALL NOT be truncated. A document
whose channels mix compression kinds within a category normalizes to the first
kind recorded for that category.

#### Scenario: Composite declares the document's compression

- **WHEN** an 8-bit RGB or Grayscale document is written with `write_psd`
- **THEN** the image-data section's 2-byte compression word is the document's recorded composite compression and `read_psd` recovers the composite pixels exactly

#### Scenario: Layer color channel and raster mask use the recorded compression

- **WHEN** a document with a pixel layer and a raster mask whose layer compression is ZIP is written
- **THEN** the layer's channel records declare compression `2` and `read_psd` reconstructs the channel and mask bytes exactly

#### Scenario: Document extra channels share the composite compression

- **WHEN** a document carrying a saved-selection extra channel is written
- **THEN** the extra plane shares the image-data section's compression and round-trips unchanged

#### Scenario: Preserved verbatim channels are re-emitted byte-for-byte

- **WHEN** a layer carries a `Layer.raw_channels` stream with its own compression header
- **THEN** read→write→read preserves that stream's exact bytes and the encoder does not re-encode it

#### Scenario: RLE and ZIP write are lossless

- **WHEN** a document read from disk is written and read back
- **THEN** the decoded pixel data of the composite, extra channels, layer channels, and raster mask equals the input

#### Scenario: ZIP-with-prediction writes reversibly

- **WHEN** a document whose compression is ZIP-with-prediction is written and read back
- **THEN** the section declares compression `3`, the predictor is inverted, and the pixels equal the input

#### Scenario: Output is deterministic and standard-reader readable

- **WHEN** the same document is written twice, or a ZIP or ZIP-with-prediction document is written and opened by the `psd-tools` oracle
- **THEN** the two outputs are byte-identical and `psd-tools` decodes the composite and channels to the same pixels

#### Scenario: Oversized scanline errors instead of truncating

- **WHEN** an RLE scanline's PackBits encoding exceeds the `u16` byte-count limit
- **THEN** `write_psd` returns a `PsdError` and writes no truncated count

#### Scenario: Section and folder records do not set the layer compression

- **WHEN** a document read from a file whose first layer record is a section divider is written
- **THEN** the recorded layer compression comes from a surviving layer channel (or defaults to RLE) and the round-trip document is equal
