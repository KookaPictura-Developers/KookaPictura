## MODIFIED Requirements

### Requirement: A normalized document records its source mode and saves in it when the mode maps back

When `read_psd` normalizes a non-RGB mode, it SHALL set
`Document.source_mode` to `Some(header_mode)` and SHALL set it to `None` for a
Grayscale or RGB file and for a constructed document. It SHALL convert every
color plane of the document, the composite and each layer, recursing into a
group's descendant layers so a nested pixel layer is converted like a top-level
one. It SHALL drop the Indexed palette from `Document.color_mode_data` on the
normalized document and SHALL instead retain that palette (and each Indexed
index plane, the composite's and every layer's) in the source store, so an
unchanged Indexed document can be written back.

`write_psd` SHALL write a document whose `source_mode` is `Lab` or `Cmyk` back
with that header color mode and, when the read retained native samples, at that
source depth, preferring the source color planes the read retained: it SHALL
re-emit a plane — the composite's or every layer's color channels —
byte-identically at the source depth when the plane is unchanged, and SHALL
otherwise convert it from the working RGB with a profile-free inverse of the
read-side conversion for that mode (8-bit, widened to the source depth when it is
16/32). The Lab inverse quantizes; the CMYK inverse (`rgb_to_cmyk`) SHALL be an
exact right-inverse of `cmyk_to_rgb` so an edited CMYK pixel reads back to the
edited RGB. Each such conversion SHALL be documented as an approximation for an
edited plane (the read is profile-free) and SHALL NOT claim Photoshop
color-management parity.

`write_psd` SHALL write an Indexed document back with header color mode Indexed
(one index channel and the retained palette) when its `source_mode` is `Indexed`
and every retained index plane — the composite's when a merged composite is
present, and every pixel layer's — still forward-converts through the palette to
the current working RGB; otherwise it SHALL write the working mode. It SHALL NOT
invent an RGB-to-palette quantization for an edited document.

`write_psd` SHALL write a flat, unchanged Bitmap document back with header color
mode Bitmap (mode 0, depth 1, one 1-bit channel) at its source compression,
re-emitting the retained packed plane byte-identically, when its `source_mode` is
`Bitmap`, it carries no layers or extra channels, and the retained packed plane
still expands to the working RGB. A Bitmap document that has been edited, or that
carries layers or extra channels, SHALL write the working mode; no RGB-to-1-bit
threshold SHALL be invented, and layered or extra-channel Bitmap write-back is
out of scope.

It SHALL re-emit the preserved color-mode-data and image-resource sections. For
a document with no source mode it SHALL continue to write the working mode.

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

#### Scenario: A 16/32-bit Lab or CMYK source saves back in its source mode and depth

- **WHEN** a 16/32-bit Lab or CMYK document is read and written unchanged
- **THEN** the output's header color mode is Lab or CMYK and its header depth is the source depth, and the retained native color planes are byte-identical to the input's

#### Scenario: Source mode is recorded

- **WHEN** a CMYK or Lab file is read
- **THEN** the document's `mode` is `Rgb` and its `source_mode` is `Some(ColorMode::Cmyk)` or `Some(ColorMode::Lab)`

#### Scenario: Native modes record no source mode

- **WHEN** an RGB or Grayscale file is read, or a document is constructed
- **THEN** its `source_mode` is `None`

#### Scenario: The Indexed palette is consumed

- **WHEN** an Indexed file is read
- **THEN** the returned document's `color_mode_data` is empty

#### Scenario: A flat unchanged Bitmap document saves as Bitmap

- **WHEN** a flat depth-1 Bitmap file with no layers or extra channels is read and written without an edit
- **THEN** the output's header color mode is Bitmap with depth 1 and one color channel, its packed plane is byte-identical to the input's, and a re-read's working RGB equals the original

#### Scenario: An edited or layered Bitmap document saves as the working mode

- **WHEN** a Bitmap document's composite is edited, or the document carries a pixel layer or an extra channel, and it is written
- **THEN** the output's header color mode is RGB and the round-tripped document's `mode` is `Rgb`, because no RGB-to-1-bit threshold is invented and layered/extra-channel Bitmap output is out of scope

#### Scenario: An unchanged Indexed document saves back as Indexed

- **WHEN** an 8-bit Indexed document is read and written without an edit
- **THEN** the output's header color mode is Indexed with one color channel, its palette is byte-identical to the input's, its index planes are byte-identical to the input's, and a re-read's working RGB equals the original

#### Scenario: An edited Indexed document saves as the working mode

- **WHEN** an 8-bit Indexed document's composite or a pixel layer is edited and written
- **THEN** the output's header color mode is RGB and the round-tripped document's `mode` is `Rgb`, because no RGB-to-palette quantization is invented

#### Scenario: An Indexed layer and its group are retained

- **WHEN** an 8-bit Indexed document has a pixel layer nested in a group and is written unchanged
- **THEN** the nested layer's index channel and the composite are re-emitted byte-identically and a re-read matches the original pixels

#### Scenario: An Indexed document without a merged composite saves back as Indexed

- **WHEN** an 8-bit Indexed document with no merged composite and an unchanged pixel layer is written
- **THEN** the output's header color mode is Indexed with one index channel, its palette is byte-identical to the input's, its layer index channel is byte-identical, and no image-data section is written
