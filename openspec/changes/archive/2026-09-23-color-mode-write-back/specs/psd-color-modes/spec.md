## MODIFIED Requirements

### Requirement: The application reports a normalized color mode

The application SHALL expose the source color mode of an opened document and
SHALL present a conversion notice to the user when a non-RGB file is normalized,
so the user knows how the document is edited and what a save writes. For an
**8-bit** Lab document the notice SHALL indicate that a save preserves Lab (a
16/32-bit Lab document writes RGB like any other converted mode); for a CMYK,
Bitmap, or Indexed document it SHALL indicate that a save writes the working
(RGB) mode. Document creation SHALL continue to offer only RGB and Grayscale.

#### Scenario: Opening a Lab file shows a preserving notice

- **WHEN** the application opens an 8-bit Lab PSD
- **THEN** the view reports that the document was converted from Lab and that a save preserves Lab

#### Scenario: Opening a 16-bit Lab file shows an RGB-save notice

- **WHEN** the application opens a 16-bit Lab PSD
- **THEN** the view reports that the document was converted from Lab and does not claim the save preserves Lab

#### Scenario: Opening a CMYK file shows an RGB-save notice

- **WHEN** the application opens a CMYK PSD
- **THEN** the view reports that the document was converted from CMYK and does not claim the save preserves CMYK

#### Scenario: Opening an RGB file shows no notice

- **WHEN** the application opens an RGB or Grayscale PSD
- **THEN** no conversion notice is shown

## REMOVED Requirements

### Requirement: A normalized document records its source mode and saves as the working mode

**Reason**: The codec now saves a Lab document back as Lab instead of always
writing the working mode, so the "saves as the working mode" half no longer
holds. The recording half is restated (with the new save behavior) in the added
"A normalized document records its source mode and saves in it when the mode
maps back" requirement.

**Migration**: No caller change is required for an RGB, Grayscale, or
constructed document. A Lab document now writes header color mode 9, so a caller
that relied on an RGB output must read the header mode; the application notice
is updated accordingly. The color-mode oracle's "Lab saves as RGB" expectation
is replaced by a Lab round-trip.

## ADDED Requirements

### Requirement: A normalized document records its source mode and saves in it when the mode maps back

When `read_psd` normalizes a non-RGB mode, it SHALL set
`Document.source_mode` to `Some(header_mode)` and SHALL set it to `None` for a
Grayscale or RGB file and for a constructed document. It SHALL convert every
color plane of the document, the composite and each layer, recursing into a
group's descendant layers so a nested pixel layer is converted like a top-level
one. It SHALL drop the Indexed palette from `Document.color_mode_data` on the
normalized document. `write_psd`
SHALL write a document whose `source_mode` is `Lab` (read from an 8-bit Lab file)
back with header color mode Lab, preferring the Lab color planes the read
retained: it SHALL re-emit a plane — the composite's or every layer's color
channels — byte-identically when the plane is unchanged, and SHALL otherwise
convert it from the working RGB with a profile-free inverse of the read-side Lab
conversion. The Lab conversion SHALL be documented as an approximation for an
edited plane (the read is profile-free) and SHALL NOT claim Photoshop
color-management parity. It SHALL re-emit the preserved color-mode-data and
image-resource sections. For every other mode (CMYK, Bitmap, Indexed) and for a
document with no source mode it SHALL continue to write the working mode.

#### Scenario: A Lab document saves as Lab

- **WHEN** an 8-bit Lab document is read and written
- **THEN** the output's header color mode is Lab and an independent decoder's RGB conversion of the output agrees with the input composite within the documented tolerance

#### Scenario: An unedited Lab document saves losslessly

- **WHEN** an 8-bit Lab document is read and written without an edit
- **THEN** the output's Lab planes are byte-identical to the input's and a re-read's working RGB equals the original

#### Scenario: An edited Lab plane is re-encoded approximately

- **WHEN** an 8-bit Lab document's composite is edited and written
- **THEN** the changed plane is re-encoded from the edited working RGB with the profile-free inverse, which the spec documents as approximate for saturated colors

#### Scenario: A Lab layer channel is converted

- **WHEN** an 8-bit Lab document with a pixel layer is written
- **THEN** the layer's color channels are Lab-encoded and read back to the same RGB pixels within tolerance

#### Scenario: A grouped Lab layer is converted

- **WHEN** an 8-bit Lab document has a pixel layer nested in a group
- **THEN** the nested layer's color channels are converted to RGB on read, like a top-level layer, and the save→read round trip keeps the same pixels

#### Scenario: A 16-bit Lab source keeps writing the working mode

- **WHEN** a 16-bit Lab document is read and written
- **THEN** the output's header color mode is RGB, because Lab write-back is scoped to an 8-bit source

#### Scenario: Source mode is recorded

- **WHEN** a CMYK or Lab file is read
- **THEN** the document's `mode` is `Rgb` and its `source_mode` is `Some(ColorMode::Cmyk)` or `Some(ColorMode::Lab)`

#### Scenario: Native modes record no source mode

- **WHEN** an RGB or Grayscale file is read, or a document is constructed
- **THEN** its `source_mode` is `None`

#### Scenario: The Indexed palette is consumed

- **WHEN** an Indexed file is read
- **THEN** the returned document's `color_mode_data` is empty

#### Scenario: Other converted modes save as the working mode

- **WHEN** a document read from a CMYK file is written and read back
- **THEN** the output's header color mode is RGB and the round-tripped document's `mode` is `Rgb`
