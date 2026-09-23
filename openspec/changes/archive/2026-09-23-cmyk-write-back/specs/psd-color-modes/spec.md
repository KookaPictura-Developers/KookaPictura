## MODIFIED Requirements

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
