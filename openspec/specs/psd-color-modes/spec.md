# psd-color-modes Specification

## Purpose
TBD - created by archiving change color-mode-read. Update Purpose after archive.
## Requirements
### Requirement: Non-RGB color modes open by normalizing to the working mode

`read_psd` SHALL accept header color modes Bitmap (0), Grayscale (1), Indexed
(2), RGB (3), CMYK (4), and Lab (9). It SHALL convert a non-RGB mode's color
planes into the engine's working mode (RGB for Bitmap/Indexed/CMYK/Lab,
Grayscale unchanged) and SHALL set the returned document's `mode` to `Rgb` or
`Grayscale` and its `depth` to `Eight`. A header color mode of Multichannel (7)
or Duotone (8), any other mode code, and any bit depth other than 8 (other than
depth 1 for Bitmap, and depths 16/32 for the Grayscale/RGB/CMYK/Lab modes read
per `psd-bit-depth`) SHALL return `PsdError::Unsupported`. A header channel
count below the mode's color-channel count SHALL return `PsdError::Invalid`.

#### Scenario: An Indexed document opens as RGB with palette colors

- **WHEN** an 8-bit Indexed PSD whose indices select distinct palette entries is read
- **THEN** `read_psd` returns a document whose `mode` is `Rgb` and whose composite pixels are the palette colors for those indices

#### Scenario: A CMYK document opens as RGB

- **WHEN** an 8-bit CMYK PSD is read
- **THEN** `read_psd` returns a document whose `mode` is `Rgb`, whose `depth` is `Eight`, and whose composite is the profile-free CMYK-to-RGB conversion of the stored planes

#### Scenario: A Lab document opens as RGB

- **WHEN** an 8-bit Lab PSD is read
- **THEN** `read_psd` returns a document whose `mode` is `Rgb` and whose composite is the profile-free CIELAB-to-sRGB conversion of the stored planes

#### Scenario: A Bitmap document opens as black and white RGB

- **WHEN** a depth-1 Bitmap PSD is read
- **THEN** `read_psd` returns an RGB document whose pixels are black where the bit is set and white where it is clear

#### Scenario: Grayscale and RGB are unchanged

- **WHEN** an 8-bit Grayscale or RGB PSD is read
- **THEN** `read_psd` returns a document whose mode is `Grayscale` or `Rgb` respectively, exactly as before this change

#### Scenario: Multichannel and Duotone remain unsupported

- **WHEN** the header color mode is Multichannel (7) or Duotone (8)
- **THEN** `read_psd` returns `PsdError::Unsupported` and does not panic

#### Scenario: Depth 16 and 32 are normalized with the mode

- **WHEN** a 16- or 32-bit CMYK or Lab document is read
- **THEN** the samples are narrowed to 8-bit and the color planes are converted to the working mode per this requirement and `psd-bit-depth`

### Requirement: Bitmap mode is read as MSB-first 1-bit rows

For a Bitmap-mode document at depth 1, `read_psd` SHALL read each row as
`ceil(width / 8)` bytes, most-significant bit first, where pixel `x` is bit
`0x80 >> (x % 8)`, a set bit SHALL decode to black (0) and a clear bit to white
(255), and each row SHALL be padded to a byte boundary. Compression `0` (raw)
and `1` (PackBits RLE) SHALL be supported at depth 1; compression `2` or `3`
SHALL return `PsdError::Unsupported`.

#### Scenario: A set bit is black and a clear bit is white

- **WHEN** a Bitmap row byte is `0xAA` for an 8-pixel-wide row
- **THEN** the decoded row is black, white, black, white, black, white, black, white

#### Scenario: Rows are byte-padded and independently encoded

- **WHEN** a Bitmap document is wider than a multiple of 8 and uses RLE compression
- **THEN** each scanline's RLE decode writes exactly `ceil(width / 8)` bytes into the row

#### Scenario: A Bitmap layer channel is bit-expanded

- **WHEN** a depth-1 Bitmap document carries a layer whose color channel is bit-packed
- **THEN** that layer's color channel is expanded to 8-bit black/white pixels and becomes three RGB channels, not decoded at an 8-bit row stride

#### Scenario: ZIP compression at depth 1 is unsupported

- **WHEN** a depth-1 Bitmap document declares compression 2 or 3
- **THEN** `read_psd` returns `PsdError::Unsupported`

### Requirement: Indexed mode maps pixel indices through the 768-byte palette

For an Indexed-mode document, the color-mode-data section SHALL be read as a
768-byte palette laid out as 256 red bytes, then 256 green bytes, then 256 blue
bytes, and a composite pixel whose stored index is `i` SHALL decode to
`(palette[i], palette[256 + i], palette[512 + i])`. A color-mode-data section
whose length is not 768 SHALL return `PsdError::Invalid`, never a panic.

#### Scenario: An index selects its palette color

- **WHEN** an Indexed pixel stores index `i` and the palette has red at byte `i`, green at byte `256 + i`, and blue at `512 + i`
- **THEN** the decoded pixel is that palette RGB color

#### Scenario: A malformed palette is rejected

- **WHEN** an Indexed document's color-mode-data section is shorter or longer than 768 bytes
- **THEN** `read_psd` returns `PsdError::Invalid`

### Requirement: CMYK and Lab use documented profile-free approximations

The CMYK conversion SHALL compute each RGB channel as `floor(stored_color *
stored_black / 255)` from the stored cyan, magenta, yellow, and black planes,
where a stored value of 0 is full ink and 255 is no ink. The Lab conversion
SHALL treat the three planes as `L* = L * 100 / 255`, `a* = a - 128`, and
`b* = b - 128` and convert CIELAB under D50 to sRGB through a Bradford D50-to-D65
adaptation. Both conversions SHALL be profile-free: the document's embedded ICC
profile SHALL NOT be applied and no Photoshop pixel parity SHALL be claimed.

#### Scenario: CMYK uses the integer formula

- **WHEN** a CMYK pixel stores `(C, M, Y, K) = (128, 64, 32, 200)`
- **THEN** the decoded RGB pixel is `(100, 50, 25)`

#### Scenario: CMYK full black and full white

- **WHEN** a CMYK pixel stores `(0, 0, 0, 0)` or `(255, 255, 255, 255)`
- **THEN** the decoded RGB pixel is `(0, 0, 0)` or `(255, 255, 255)` respectively

#### Scenario: Lab neutral gray and white

- **WHEN** a Lab pixel stores `(200, 128, 128)` or `(255, 128, 128)`
- **THEN** the decoded RGB pixel is a neutral gray near `(194, 194, 194)` or white near `(255, 255, 255)`, within 2 per channel

#### Scenario: An embedded profile is ignored

- **WHEN** a CMYK document carries an embedded ICC profile
- **THEN** the decoded RGB values use the profile-free formula and do not depend on the profile bytes

### Requirement: Layer color planes are converted with the document

When a non-RGB document carries layers, `read_psd` SHALL convert each layer's
color channels with the same conversion as the composite, replacing them with
the working mode's color planes while leaving every non-color channel (`-1`
transparency, `-2` mask, and unmodeled channels) unchanged. A layer whose color
channels do not match the mode's channel layout SHALL be left unchanged rather
than producing a wrong conversion.

#### Scenario: A CMYK layer becomes RGB

- **WHEN** a CMYK document carries a pixel layer with four color channels
- **THEN** the returned layer carries three RGB color channels that are the profile-free conversion of the stored CMYK planes, and its transparency channel is unchanged

#### Scenario: Alpha and mask channels are untouched

- **WHEN** a non-RGB layer carries a `-1` transparency channel and a `-2` mask channel
- **THEN** those channels' bytes are the same as in the file

### Requirement: The application reports a normalized color mode

The application SHALL expose the source color mode of an opened document and
SHALL present a conversion notice to the user when a non-RGB file is normalized,
so the user knows how the document is edited and what a save writes. For an
**8-bit** Lab or CMYK document the notice SHALL indicate that a save preserves
that mode (a 16/32-bit Lab or CMYK document writes RGB like any other converted
mode); for a Bitmap or Indexed document it SHALL indicate that a save writes the
working (RGB) mode. Document creation SHALL continue to offer only RGB and
Grayscale.

#### Scenario: Opening a Lab file shows a preserving notice

- **WHEN** the application opens an 8-bit Lab PSD
- **THEN** the view reports that the document was converted from Lab and that a save preserves Lab

#### Scenario: Opening a CMYK file shows a preserving notice

- **WHEN** the application opens an 8-bit CMYK PSD
- **THEN** the view reports that the document was converted from CMYK and that a save preserves CMYK

#### Scenario: Opening a 16-bit Lab or CMYK file shows an RGB-save notice

- **WHEN** the application opens a 16-bit Lab or CMYK PSD
- **THEN** the view reports the conversion and does not claim the save preserves the source mode

#### Scenario: Opening an RGB file shows no notice

- **WHEN** the application opens an RGB or Grayscale PSD
- **THEN** no conversion notice is shown

### Requirement: Color-mode fixtures agree with an independent decoder

The system SHALL commit the fixtures
`crates/pictura-codec/tests/fixtures/indexed.psd`, `cmyk.psd`, `lab.psd`, and
`bitmap.psd`, regenerated byte-stably by `scripts/generate-fixtures.py`, and a
test SHALL verify that `read_psd`'s decoded pixels agree with the independent
`psd-tools` decoder for the Indexed, CMYK, and Bitmap fixtures (CMYK within the
documented 1-LSB rounding), with the exact lcms2 LAB-to-sRGB transform for the
Lab fixture (within the documented 1-LSB tolerance; psd-tools' optimized
`.convert("RGB")` composite SHALL NOT be the reference), and with a hand-built
Bitmap RLE file. The oracle SHALL self-skip with a clear message when `psd-tools`
is unavailable and SHALL NOT fail the suite in that case.

#### Scenario: Indexed fixture agrees with psd-tools

- **WHEN** `read_psd` decodes `indexed.psd` and `psd-tools` decodes the same file
- **THEN** the RGB pixels are identical

#### Scenario: CMYK fixture agrees with psd-tools

- **WHEN** `read_psd` decodes `cmyk.psd` and `psd-tools` decodes the same file
- **THEN** every RGB channel differs by at most 1 (the engine floors the profile-free formula, Pillow rounds it)

#### Scenario: Lab fixture agrees with the exact lcms2 transform

- **WHEN** `read_psd` decodes `lab.psd` and lcms2's exact (unoptimized) LAB-to-sRGB transform decodes the same file's Lab composite
- **THEN** every RGB channel differs by at most 1

#### Scenario: The Bitmap RLE file decodes against its own bytes

- **WHEN** a hand-built depth-1 Bitmap file with RLE compression is read
- **THEN** the decoded pixels equal the hand-computed black/white pattern

#### Scenario: The oracle self-skips without psd-tools

- **WHEN** the `psd-tools` package is not available
- **THEN** the oracle reports a skip and the suite passes

### Requirement: A normalized document records its source mode and saves in it when the mode maps back

When `read_psd` normalizes a non-RGB mode, it SHALL set
`Document.source_mode` to `Some(header_mode)` and SHALL set it to `None` for a
Grayscale or RGB file and for a constructed document. It SHALL convert every
color plane of the document, the composite and each layer, recursing into a
group's descendant layers so a nested pixel layer is converted like a top-level
one. It SHALL drop the Indexed palette from `Document.color_mode_data` on the
normalized document. `write_psd`
SHALL write a document whose `source_mode` is `Lab` or `Cmyk` (read from an 8-bit
Lab or CMYK file) back with that header color mode, preferring the source color
planes the read retained: it SHALL re-emit a plane — the composite's or every
layer's color channels — byte-identically when the plane is unchanged, and SHALL
otherwise convert it from the working RGB with a profile-free inverse of the
read-side conversion for that mode. The Lab inverse quantizes; the CMYK inverse
(`rgb_to_cmyk`) SHALL be an exact right-inverse of `cmyk_to_rgb` so an edited
CMYK pixel reads back to the edited RGB. Each such conversion SHALL be documented
as an approximation for an edited plane (the read is profile-free) and SHALL NOT
claim Photoshop color-management parity. It SHALL re-emit the preserved
color-mode-data and image-resource sections. For Bitmap, Indexed, and a document
with no source mode it SHALL continue to write the working mode.

#### Scenario: A Lab document saves as Lab

- **WHEN** an 8-bit Lab document is read and written
- **THEN** the output's header color mode is Lab and an independent decoder's RGB conversion of the output agrees with the input composite within the documented tolerance

#### Scenario: A CMYK document saves as CMYK

- **WHEN** an 8-bit CMYK document is read and written
- **THEN** the output's header color mode is CMYK with four color channels, the unedited color planes are byte-identical to the source, and the output agrees with an independent decoder

#### Scenario: An unedited Lab document saves losslessly

- **WHEN** an 8-bit Lab document is read and written without an edit
- **THEN** the output's Lab planes are byte-identical to the input's and a re-read's working RGB equals the original

#### Scenario: An unedited CMYK document saves losslessly

- **WHEN** an 8-bit CMYK document is read and written without an edit
- **THEN** the output's CMYK color planes are byte-identical to the input's and a re-read agrees with the original working RGB

#### Scenario: An edited Lab plane is re-encoded approximately

- **WHEN** an 8-bit Lab document's composite is edited and written
- **THEN** the changed plane is re-encoded from the edited working RGB with the profile-free inverse, which the spec documents as approximate for saturated colors

#### Scenario: An edited CMYK plane uses the exact inverse

- **WHEN** an 8-bit CMYK document's color plane is edited and written
- **THEN** reading the output back yields exactly the edited RGB bytes

#### Scenario: A Lab layer channel is converted

- **WHEN** an 8-bit Lab document with a pixel layer is written
- **THEN** the layer's color channels are Lab-encoded and read back to the same RGB pixels within tolerance

#### Scenario: A CMYK layer channel is converted

- **WHEN** an 8-bit CMYK document with a pixel layer is written
- **THEN** the layer carries four CMYK color channels and reads back to the same RGB pixels within the documented tolerance

#### Scenario: A grouped Lab layer is converted

- **WHEN** an 8-bit Lab document has a pixel layer nested in a group
- **THEN** the nested layer's color channels are converted to RGB on read, like a top-level layer, and the save→read round trip keeps the same pixels

#### Scenario: A 16-bit Lab or CMYK source keeps writing the working mode

- **WHEN** a 16/32-bit Lab or CMYK document is read and written
- **THEN** the output's header color mode is RGB, because the mode write-back is scoped to an 8-bit source

#### Scenario: Source mode is recorded

- **WHEN** a CMYK or Lab file is read
- **THEN** the document's `mode` is `Rgb` and its `source_mode` is `Some(ColorMode::Cmyk)` or `Some(ColorMode::Lab)`

#### Scenario: Native modes record no source mode

- **WHEN** an RGB or Grayscale file is read, or a document is constructed
- **THEN** its `source_mode` is `None`

#### Scenario: The Indexed palette is consumed

- **WHEN** an Indexed file is read
- **THEN** the returned document's `color_mode_data` is empty

#### Scenario: Bitmap and Indexed save as the working mode

- **WHEN** a document read from a Bitmap or Indexed file is written and read back
- **THEN** the output's header color mode is RGB and the round-tripped document's `mode` is `Rgb`

