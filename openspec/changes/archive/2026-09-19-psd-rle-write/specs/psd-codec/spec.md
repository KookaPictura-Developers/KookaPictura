## MODIFIED Requirements

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

## ADDED Requirements

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
