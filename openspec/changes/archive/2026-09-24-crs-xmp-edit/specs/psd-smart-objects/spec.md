# Specs delta: crs-xmp-edit

## MODIFIED Requirements

### Requirement: Camera Raw crs settings are preserved

The system SHALL expose the `crs:` XMP settings associated with an embedded
source on read as a typed view of the fixed property set (`Exposure2012`,
`Contrast2012`, `Highlights2012`, `Shadows2012`, `Whites2012`, `Blacks2012`,
`Clarity2012`, `Vibrance`, `Saturation`, `Temperature`, `Tint`), each an
optional finite `f64`. Properties outside that set SHALL remain only inside the
raw packet. An unedited document SHALL re-emit the embedded payload and the
preserved `lnk*` section byte-exact on write.

The system SHALL provide `set_crs_property(document, uuid, name, value)` that
replaces one fixed-set property in the packet, updates the smart object's
payload and typed view, and rewrites the matching embedded payload inside the
document's preserved linked-record section so a subsequent write emits the new
value. A name outside the fixed set, a missing uuid, a non-embedded object, or
a packet that cannot be safely patched SHALL return an error and MUST NOT
mutate the document.

#### Scenario: Settings are exposed on read

- **WHEN** an embedded source carries a packet with `crs:Exposure2012="+0.50"`
- **THEN** the typed view's exposure is `Some(0.5)` and the raw packet is retained

#### Scenario: An unedited document round-trips byte-exact

- **WHEN** a document with `crs:` settings is written without calling the edit API
- **THEN** the written linked-record section is byte-identical to the input

#### Scenario: Editing exposure round-trips through write

- **WHEN** `set_crs_property` sets `Exposure2012` to `1.25` on an embedded source
  and the document is written and read back
- **THEN** the typed view's exposure is `Some(1.25)` and bytes outside the
  patched packet span are unchanged

#### Scenario: An unknown property name is rejected without mutation

- **WHEN** `set_crs_property` is called with a name outside the fixed set
- **THEN** it returns an error and the document's linked-record section is unchanged

#### Scenario: A missing uuid is rejected without mutation

- **WHEN** `set_crs_property` is called with a uuid that matches no layer
- **THEN** it returns an error and the document is unchanged
