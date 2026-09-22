# color-profile-assignment Specification

## Purpose
TBD - created by archiving change assign-convert-profile. Update Purpose after archive.
## Requirements
### Requirement: A document carries a working color profile

`pictura-core` `Document` SHALL expose `document_icc: Option<Vec<u8>>` holding
the ICC bytes of the profile the document's stored pixels are in, where `None`
means the sRGB working space. A document opened with the Convert incoming policy
(`read_psd`'s behaviour), constructed, or assigned/converted to sRGB SHALL have
`document_icc` equal to `None`; a document opened with the Preserve policy for a
non-sRGB embedded profile SHALL have `document_icc` equal to the embedded bytes,
because its stored pixels remain in that profile. `document_icc` SHALL be
distinct from `source_icc`, which records the profile a read normalised away.

#### Scenario: A freshly opened document is in the sRGB working space

- **WHEN** a PSD with no embedded profile (or with an sRGB profile) is read
- **THEN** `document_icc` is `None`

#### Scenario: A read-normalised document records the source separately

- **WHEN** a PSD with a non-sRGB embedded profile is read with the Convert policy
- **THEN** `source_icc` holds the embedded bytes and `document_icc` is still
  `None`

#### Scenario: A preserved document carries the embedded profile

- **WHEN** a PSD with a non-sRGB embedded profile is read with the Preserve policy
- **THEN** `document_icc` holds the embedded bytes, the pixels are unchanged, and
  `source_icc` is `None`

### Requirement: Assign Profile retags a document without changing pixels

`pictura-codec` SHALL expose an operation that, given a document and a target
profile, sets the document's working profile to that target and rewrites its
image-resource section so it carries resource 1039 framed with the target's ICC
bytes, or omits 1039 when the target is the sRGB working space. It SHALL NOT
modify the document composite or any layer's color-channel bytes.

#### Scenario: Assign leaves every pixel byte unchanged

- **WHEN** Assign Profile is applied to a document with a target profile
- **THEN** `doc.composite` and every layer's channel data are byte-identical to
  before the operation

#### Scenario: Assign tags the target profile for saving

- **WHEN** Assign Profile to Adobe RGB is applied and the document is written
- **THEN** the image-resource section contains a 1039 block whose data is Adobe
  RGB's ICC bytes

#### Scenario: Assigning the sRGB working space removes the tag

- **WHEN** Assign Profile is applied with the sRGB working space as the target
- **THEN** the document has no 1039 resource and `document_icc` is `None`

### Requirement: Convert to Profile transforms pixels and retags

`pictura-codec` SHALL expose an operation that, given a document and a
destination profile, converts the document composite and every layer's color
channels from the document's current working profile (or sRGB when
`document_icc` is `None`) to the destination using relative colorimetric intent
with no black-point compensation, then sets the working profile and resource
1039 as Assign Profile does. The transform SHALL be applied with the existing
planar color conversion, and a document whose color channels are incomplete SHALL
be left with those channels unconverted rather than failing.

#### Scenario: Convert transforms the pixels

- **WHEN** Convert to Profile is applied to a document whose composite is a
  known color patch
- **THEN** the composite bytes equal the reference conversion of that patch from
  the current profile to the destination within one LSB per channel

#### Scenario: Convert preserves appearance and tags the destination

- **WHEN** a document is converted from sRGB to Adobe RGB and then displayed
- **THEN** the displayed bytes are close to the pre-conversion display and the
  document carries a 1039 resource for Adobe RGB

### Requirement: An assigned or converted document is displayed in the working space

The application SHALL convert the composite it shows on the canvas from the
document's working profile to the sRGB working space before display, while
keeping `doc.composite` and layer channels in the document's working profile. A
document with no working profile SHALL be displayed unchanged.

#### Scenario: Assign changes the displayed appearance

- **WHEN** a document is assigned a non-sRGB profile that reinterprets its
  numbers
- **THEN** the displayed canvas pixels change while the stored pixels do not

#### Scenario: Convert preserves the displayed appearance

- **WHEN** a document is converted to a different profile
- **THEN** the displayed canvas pixels are within the color-conversion tolerance
  of the pre-conversion display

### Requirement: Assign Profile and Convert to Profile are single undo steps

Each operation SHALL record exactly one history state on the active document and
mark it dirty. Undoing a Convert SHALL restore the pre-conversion composite and
layer pixels exactly; undoing an Assign SHALL restore the previous profile and
image-resource section.

#### Scenario: Convert is a single reversible step

- **WHEN** Convert to Profile is applied and then undone
- **THEN** the composite, every layer's pixel bytes, and the working profile
  equal their pre-conversion values, and the operation added exactly one
  history state

#### Scenario: Assign records no pixel change but one state

- **WHEN** Assign Profile is applied
- **THEN** exactly one history state is added and undoing it restores the prior
  profile

