## MODIFIED Requirements

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
