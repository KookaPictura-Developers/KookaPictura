## MODIFIED Requirements

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

## ADDED Requirements

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
