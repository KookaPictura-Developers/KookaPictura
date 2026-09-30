# Spec Delta

## ADDED Requirements

### Requirement: Repeated working-space conversion reuses the parsed profile

Converting a document's pixels to its working/display color space SHALL reuse the
parsed working profile (`document_icc`) and its built transform across repeated
conversions of the same document state, rather than reparsing the profile and
rebuilding the transform each time. The converted pixels MUST be byte-identical
to a conversion that reparses the profile, and a change to the document's working
profile (assign or convert) SHALL invalidate the reuse.

#### Scenario: Repeated conversions are byte-identical to a fresh parse

- **WHEN** a document's pixels are converted to the working space twice without a
  change to its working profile
- **THEN** both conversions return the same bytes, equal to a conversion that
  reparses the profile

#### Scenario: A working-profile change is picked up

- **WHEN** a document's working profile is changed and its pixels are converted
  again
- **THEN** the conversion uses the new profile and equals a fresh conversion under
  that profile

#### Scenario: A document in the sRGB working space is not parsed

- **WHEN** a document whose `document_icc` is `None` has its pixels converted to
  the working space
- **THEN** the pixels are returned unchanged and no profile is parsed
