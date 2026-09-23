## ADDED Requirements

### Requirement: A saved ICC profile matches the output color mode

`write_psd` SHALL emit image resource `1039` only when the profile's ICC
data-space signature matches the output header color mode: `RGB ` for RGB,
`GRAY` for Grayscale, `CMYK` for CMYK, and `Lab ` for Lab. A framable preserved
profile whose data-space signature differs SHALL be dropped from the emitted
section, so a document normalized from a CMYK or Lab source is not saved with the
mismatched profile still tagged. A framable profile whose signature matches, and
a profile shorter than the 20-byte ICC header that carries no data-space
signature, SHALL be re-emitted byte-for-byte, as SHALL every other image resource
and any bytes the section parser could not frame; a document that carries no
resource `1039` SHALL be unchanged.

#### Scenario: A 16-bit CMYK source saves untagged

- **WHEN** a 16/32-bit CMYK file is read (normalized to RGB, its CMYK resource `1039` preserved) and written
- **THEN** the output's header color mode is RGB and its image resources carry no resource `1039`

#### Scenario: An 8-bit CMYK source keeps its CMYK profile

- **WHEN** an 8-bit CMYK document carrying a CMYK resource `1039` is written
- **THEN** the output's header color mode is CMYK and its resource `1039` data is byte-identical to the input's

#### Scenario: A 16-bit Lab source saves untagged

- **WHEN** a 16/32-bit Lab file is read (normalized to RGB, its Lab resource `1039` preserved) and written
- **THEN** the output's header color mode is RGB and its image resources carry no resource `1039`

#### Scenario: A Grayscale document keeps its GRAY profile

- **WHEN** a Grayscale document carrying a `GRAY` resource `1039` is written
- **THEN** the output's header color mode is Grayscale and its resource `1039` data is byte-identical to the input's

#### Scenario: A matching RGB profile survives

- **WHEN** an RGB document carrying an RGB resource `1039` is written
- **THEN** the output's resource `1039` data is byte-identical to the input's

#### Scenario: Other resources survive the filter

- **WHEN** a document carrying resource `1039` and an XMP resource is written and its `1039` is dropped
- **THEN** the XMP resource is present with identical bytes in the output

#### Scenario: A profile too short to classify is preserved

- **WHEN** a document carries a resource `1039` whose data is shorter than the ICC data-space field
- **THEN** its image-resource section is re-emitted byte-for-byte
