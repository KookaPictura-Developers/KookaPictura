## ADDED Requirements

### Requirement: An embedded non-sRGB ICC profile is applied on read

`read_psd` SHALL convert an RGB document to the sRGB working space when its
image resources carry an ICC profile (resource `1039`) that parses as an ICC
profile and is not sRGB: the composite's color planes and every layer's color
channels SHALL be converted from the embedded profile to sRGB with the
relative-colorimetric intent, and the original profile bytes SHALL be recorded
in `Document.source_icc`. "Is sRGB" SHALL be decided by the profile's
`Description` tag containing `srgb`, case-insensitively. A document that is not
RGB (for example Grayscale), a profile that is absent, does not parse, is sRGB,
or cannot be transformed by the engine, a malformed resource section, or a layer
missing a color channel SHALL leave the pixels and resources unchanged and SHALL
NOT set `source_icc`. The document's `mode` and `depth` SHALL be unchanged by
this pass.

#### Scenario: An Adobe-RGB document is converted

- **WHEN** an RGB file carries an Adobe RGB (non-sRGB) ICC profile and a known pixel layer
- **THEN** `read_psd` returns that layer's pixels converted to sRGB (matching an independent lcms2 conversion), `source_icc` is the embedded profile bytes, and `mode` is unchanged

#### Scenario: A Grayscale document with a profile is untouched

- **WHEN** a Grayscale file carries a non-sRGB profile
- **THEN** the pixels and the resource bytes are unchanged and `source_icc` is `None`

#### Scenario: Other resources survive the conversion

- **WHEN** the file also carries an XMP resource
- **THEN** the XMP bytes are unchanged in the output resources while resource `1039` is removed

#### Scenario: A layer's color channels are converted

- **WHEN** the file has a pixel layer with color channels in the non-sRGB space
- **THEN** those channels are converted too, so the composited result matches the profile-correct color

#### Scenario: An sRGB document is untouched

- **WHEN** the embedded profile is sRGB (or names sRGB) or there is no profile
- **THEN** every pixel and the image-resource bytes are unchanged from a profile-free read

#### Scenario: An undecodable profile is ignored

- **WHEN** resource `1039` is present but does not parse as an ICC profile
- **THEN** the pixels and resources are unchanged and `source_icc` is `None`

### Requirement: An ICC-normalized document saves untagged

`write_psd` SHALL write an image-resource section without resource `1039` when
the document was ICC-normalized on read (`source_icc` is `Some`), so the sRGB
pixels are not re-tagged with the source profile. Every other resource SHALL be
re-emitted byte-for-byte.

#### Scenario: The saved file drops the source profile

- **WHEN** an ICC-normalized document is written and re-read
- **THEN** its image resources no longer contain resource `1039` and its pixels are the converted sRGB values

#### Scenario: Other resources survive

- **WHEN** the source file also carries an EXIF or XMP resource
- **THEN** those resources are present with identical bytes in the output

### Requirement: The app reports an ICC conversion

The app SHALL show a status-bar notice when an opened document was normalized
from an embedded ICC profile, naming the source profile's description.

#### Scenario: Opening a converted document shows the notice

- **WHEN** a document with a non-sRGB embedded profile is opened
- **THEN** the view reports an "embedded ICC profile" conversion notice

### Requirement: The ICC fixture is proven by an independent converter

The committed fixture SHALL embed the synthesized profile
`crates/pictura-codec/tests/fixtures/psd_icc_rgb.icc` (produced by
`cargo run -p pictura-color --example dump_adobe_rgb`) as the image-resource
profile of `crates/pictura-codec/tests/fixtures/icc_profile.psd`, and a test
SHALL compare the converted composite against an independent lcms2 conversion of
a known source color.

#### Scenario: The converted composite matches lcms2

- **WHEN** the fixture is read and an independent converter converts its source color from the embedded profile to sRGB
- **THEN** the composite's pixel is within 1 LSB of that conversion

