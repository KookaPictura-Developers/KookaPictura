## MODIFIED Requirements

### Requirement: The XMP packet is exposed as raw text

`read_metadata` SHALL expose the document's XMP image resource (id 1060) as a
UTF-8 (lossy) string, without resolving XML entities, so XXE and entity-expansion
payloads have no effect. `pictura-codec` SHALL additionally expose the packet's
fixed typed properties (title, creator, description, subject, rights, credit,
source, headline, marked) parsed from the same resource, per
`psd-xmp-metadata`. The raw string is unchanged by the addition.

#### Scenario: The fixture's XMP packet is returned as text

- **WHEN** `read_metadata` reads the fixture
- **THEN** its XMP string equals the stored resource 1060 bytes decoded as text

#### Scenario: The parsed properties accompany the raw string

- **WHEN** the fixture's XMP is read
- **THEN** the parsed title, creator, description, and rights are available alongside the raw string

### Requirement: The application edits IPTC core fields in File Info

The application SHALL enable `File > File Info…` when a document is open and
SHALL open a dialog listing the decoded metadata under the categories Camera
Data (EXIF), Description (the parsed XMP properties), IPTC, and Raw Data (the raw
XMP packet). The Camera Data, Description, and Raw Data categories SHALL be
read-only. The Description category SHALL list the parsed XMP properties. The
IPTC category SHALL present the six IPTC-Core fields — Object Name, By-line,
Copyright Notice, Caption/Abstract, Credit, and Source — as editable text
prefilled from the active document, plus a read-only list of the remaining IPTC
records. On OK the application SHALL apply only the fields
the user changed (so an untouched value is never rewritten), write each changed
field to both the XMP packet and the IIM record so the two never conflict, record
exactly one undo state labelled for the edit, and mark the document dirty; OK
with no changed fields SHALL apply nothing and record no state; on Cancel it
SHALL apply nothing. The bridge SHALL expose `iptc_edit_fields()` returning
`"record:dataset\tLabel\tValue"` rows, `xmp_rows()` returning the parsed XMP
properties as label/value rows, and an apply method taking `"record:dataset\tValue"`
rows.

#### Scenario: File Info is disabled without a document

- **WHEN** no document is open
- **THEN** the `File > File Info…` command is disabled

#### Scenario: Opening File Info shows the editable categories

- **WHEN** the metadata fixture is open and `File > File Info…` is invoked
- **THEN** the dialog shows the Camera Data, Description, IPTC, and Raw Data categories, the Camera Data category lists the decoded EXIF Make, the Description category lists the parsed XMP title read-only, and the IPTC category shows the document's Object Name in an editable field

#### Scenario: Editing a field and accepting applies one undo state

- **WHEN** the Object Name field is changed and the dialog is accepted
- **THEN** the document's decoded IPTC Object Name and parsed XMP title are both the new value, the document is dirty, and one Undo restores the previous value in both

#### Scenario: Cancelling applies nothing

- **WHEN** the Object Name field is changed and the dialog is rejected
- **THEN** the document's decoded IPTC Object Name and parsed XMP title are unchanged and the document is not dirtied by the dialog

#### Scenario: Accepting with no changes records nothing

- **WHEN** the dialog is accepted without changing any field
- **THEN** no edit is applied, no undo state is recorded, and the document is not dirtied by the dialog
