## ADDED Requirements

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
