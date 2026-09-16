# psd-layer-io Specification

## Purpose
TBD - created by archiving change m1-document-model. Update Purpose after archive.
## Requirements
### Requirement: Read the PSD layer-and-mask section

`read_psd` SHALL parse the Layer and Mask Information section when present: the
outer section length, the layer info length, the signed layer count (its
magnitude is the record count), the layer records, then each record's channel
image data in layer order, then the global layer mask info. A zero-length layer
section SHALL produce an empty layer list.

#### Scenario: Layerless documents still parse

- **WHEN** a PSD has a zero-length layer-and-mask section
- **THEN** `read_psd` succeeds with `doc.layers` empty and a valid composite

#### Scenario: Oracle fixtures parse with the expected dimensions and mode

- **WHEN** each of `two_layers.psd`, `group.psd`, `masked.psd`, `gray.psd`,
  `adjustment.psd` is read
- **THEN** it yields an 8×8 document with the color mode the generator used

#### Scenario: Global layer mask info is consumed

- **WHEN** a layer section carries a global layer mask info block
- **THEN** the codec advances past it by its declared length and still parses the
  image data section

### Requirement: Channel image data supports raw and PackBits RLE

Layer channel image data SHALL support compression code `0` (raw) and `1`
(PackBits/RLE). RLE scanline byte-count entries SHALL be 2 bytes in PSD and 4
bytes in PSB. RLE decoding SHALL be lossless and SHALL reproduce the raw bytes
exactly. Compression codes `2` and `3` (ZIP) SHALL return an unsupported error.

#### Scenario: RLE decodes to the raw bytes

- **WHEN** a channel is encoded with PackBits literal and repeat runs
- **THEN** decoding yields the original byte sequence

#### Scenario: PSB uses four-byte scanline counts

- **WHEN** a PSB (`version == 2`) channel is RLE-decoded
- **THEN** the scanline count table is read as 4-byte entries and the row decodes
  correctly

#### Scenario: ZIP compression is rejected, not misread

- **WHEN** a layer channel declares compression `2` or `3`
- **THEN** `read_psd` returns `PsdError::Unsupported` and does not panic

### Requirement: luni Unicode layer names

The codec SHALL read a layer's Unicode name from the `'luni'` additional-layer
tagged block and SHALL prefer it over the legacy Pascal name. Writing SHALL emit
a `'luni'` block for each layer.

#### Scenario: Unicode names from the oracle are used

- **WHEN** `psd-tools`-authored `two_layers.psd` is read
- **THEN** the layers are named `Red` and `Blue` from their `'luni'` blocks

#### Scenario: Names round-trip through write/read

- **WHEN** a layer named with non-ASCII text is written and read back
- **THEN** the name is preserved

### Requirement: lsct group markers build the group tree

The codec SHALL read group structure from `'lsct'` section markers: a section
divider (type 3) bounds a group and folder records (types 1 and 2) close it, with
open and closed folders treated the same for M1. When the `'lsct'` block carries
a `'8BIM'` blend key, that key SHALL take precedence over the folder record's own
key, so a Pass Through group loads as `BlendMode::PassThrough`.

#### Scenario: Oracle group reads as a pass-through group

- **WHEN** `group.psd` is read
- **THEN** the top-level entry is a group named `Group A` with
  `blend == BlendMode::PassThrough` and two children

#### Scenario: A Pass Through group round-trips

- **WHEN** a document with a `PassThrough` group is written and read back
- **THEN** the group is still a group with `BlendMode::PassThrough` and its
  children are unchanged

### Requirement: Write the PSD layer-and-mask section

`write_psd` SHALL emit a layer-and-mask section for a document with pixel layers,
groups, and raster masks, such that read-write-read returns a `Document` equal to
the input. Groups SHALL be emitted as a section divider followed by folder
records, in depth-first bottom-first order, with a `'luni'` name and an `'lsct'`
marker per record. Raster mask data SHALL be written as a `-2` channel. A layer
whose lock flags are set SHALL emit a `'lspf'` block, a layer whose color label
is not `None` SHALL emit a `'lclr'` block, and a layer whose fill opacity is not
`255` SHALL emit an `'iOpa'` block; each block SHALL be omitted at its default.
A document that uses only default fill, no lock, and no color label MUST
serialize byte-for-byte identically to a build without these blocks, so
`write_psd`'s output for existing documents is unchanged.

#### Scenario: Layers, a group, and a mask round-trip

- **WHEN** a document with a pixel layer, a group of two children, and a masked
  layer is written and read back
- **THEN** the reconstructed `Document` equals the original, including names,
  blends, opacity, bounds, group children, and mask rect/disabled/data

#### Scenario: Grayscale layer round-trips

- **WHEN** a grayscale document with one pixel layer is written and read back
- **THEN** the reconstructed document equals the original

#### Scenario: Adjustment blocks are preserved opaquely

- **WHEN** a layer carries an unrecognized additional-layer adjustment block
- **THEN** the block's 4-byte key and payload are written back unchanged on
  round-trip, without the codec interpreting them

#### Scenario: Default attributes preserve the byte layout

- **WHEN** a document whose every layer has fill 255, no lock flags, and no
  color label is written
- **THEN** it emits no `'lspf'`, `'lclr'`, or `'iOpa'` block and the bytes are
  identical to the output before this change

### Requirement: Round-trip verified against the psd-tools oracle

The fixtures SHALL be authored by the independent `psd-tools` library, not by
the codec. Tests SHALL assert the codec reconstructs the expected tree from those
fixtures, and a PSD written by the codec SHALL open and parse in `psd-tools`.

#### Scenario: Two-layer tree agrees with the oracle

- **WHEN** `two_layers.psd` is read
- **THEN** its two layers, names, and bounds match the values `psd-tools` wrote

#### Scenario: Masked fixture exposes its mask

- **WHEN** `masked.psd` is read
- **THEN** one layer named `Masked` is returned with a raster layer mask

#### Scenario: psd-tools opens codec output

- **WHEN** `pictura-codec` writes a PSD carrying an extra channel and
  `psd-tools` opens it
- **THEN** `psd-tools` reports the bumped channel count and its composite alpha
  equals the extra plane

### Requirement: Malformed input errors, never panics

Malformed or truncated layer data SHALL return a `PsdError` and SHALL NOT panic,
index out of bounds, or loop forever. This includes a bad file signature, a
section whose declared length exceeds the file, an impossible layer count, a
bogus channel length, and a bad tagged-block or blend signature.

#### Scenario: Truncated file is an error

- **WHEN** a valid PSD is truncated inside or before the layer section
- **THEN** `read_psd` returns an error rather than panicking

#### Scenario: Bogus layer count is an error

- **WHEN** a layer record count claims more records than the section holds
- **THEN** `read_psd` returns an error

#### Scenario: Bogus channel length is an error

- **WHEN** a channel-info length field is set to an impossible value
- **THEN** `read_psd` returns an error

#### Scenario: Bad signatures are errors

- **WHEN** the file signature is not `8BPS`, the layer blend signature is not
  `8BIM`, or a tagged block signature is not `8BIM`
- **THEN** `read_psd` returns the corresponding error

### Requirement: Read layer lock, color, and fill attributes

`read_psd` SHALL parse the `'lspf'` (protected setting), `'lclr'` (sheet color),
and `'iOpa'` (fill opacity) additional-layer-info blocks. `'lspf'` SHALL be read
as the low bits of its 4-byte value: transparency (`0x01`), image pixels
(`0x02`), and position (`0x04`); a value with all three bits sets Lock All.
`'lclr'` SHALL be read from its leading 2-byte color value, mapping the CS6
palette `None, Red, Orange, Yellow, Green, Blue, Violet, Gray` and treating any
unrecognized value as `None`. `'iOpa'` SHALL be read from the first byte of its
payload, independent of the block's declared length. An absent block SHALL mean
the default: no lock, no color, fill `255`. The parser SHALL NOT fail on a
length-variant or unrecognized payload; it SHALL take the default instead.

#### Scenario: A lock, color, and fill file reads back

- **WHEN** a PSD whose layer carries `'lspf'` position, `'lclr'` Red, and
  `'iOpa'` 128 is read
- **THEN** the layer reports the position lock, the Red color label, and fill 128

#### Scenario: Missing tags default

- **WHEN** a layer record has no `'lspf'`, `'lclr'`, or `'iOpa'` block
- **THEN** the layer reports no lock, no color label, and fill 255

#### Scenario: Attributes round-trip

- **WHEN** a document with non-default fill, lock, and color is written and read
  back
- **THEN** the reconstructed document equals the original

#### Scenario: psd-tools agrees with codec output

- **WHEN** the codec writes a document with fill 128, a lock, and a color label
  and `psd-tools` opens it
- **THEN** `psd-tools` reports the same `fill_opacity`, lock flags, and
  `sheet_color`

#### Scenario: Unknown color falls back

- **WHEN** a `'lclr'` payload holds a value outside the CS6 palette
- **THEN** the layer reports no color label and reading does not error

