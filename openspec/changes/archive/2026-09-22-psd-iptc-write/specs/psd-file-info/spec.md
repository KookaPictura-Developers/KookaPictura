## REMOVED Requirements

### Requirement: The application shows a read-only File Info dialog

**Reason**: The File Info dialog is now editable for IPTC core fields, so its
mode is no longer read-only.

**Migration**: Use `The application edits IPTC core fields in File Info`, which
keeps the same categories and adds edit/apply behavior.

## ADDED Requirements

### Requirement: The application edits IPTC core fields in File Info

The application SHALL enable `File > File Info…` when a document is open and
SHALL open a dialog listing the decoded metadata under the categories Camera
Data (EXIF), IPTC, and Raw Data (the raw XMP packet). The Camera Data and Raw
Data categories SHALL be read-only. The IPTC category SHALL present the six core
fields — Object Name, By-line, Copyright Notice, Caption/Abstract, Credit, and
Source — as editable text prefilled from the active document, plus a read-only
list of the remaining IPTC records. On OK the application SHALL apply only the
fields the user changed (so an untouched value is never rewritten), record
exactly one undo state labelled for the edit, and mark the document dirty; OK
with no changed fields SHALL apply nothing and record no state; on Cancel it
SHALL apply nothing. The bridge
SHALL expose `iptc_edit_fields()` returning `"record:dataset\tLabel\tValue"` rows
and `apply_iptc_edits(edits)` taking `"record:dataset\tValue"` rows.

#### Scenario: File Info is disabled without a document

- **WHEN** no document is open
- **THEN** the `File > File Info…` command is disabled

#### Scenario: Opening File Info shows the editable categories

- **WHEN** the metadata fixture is open and `File > File Info…` is invoked
- **THEN** the dialog shows the Camera Data, IPTC, and Raw Data categories, the Camera Data category lists the decoded EXIF Make, and the IPTC category shows the document's Object Name in an editable field

#### Scenario: Editing a field and accepting applies one undo state

- **WHEN** the Object Name field is changed and the dialog is accepted
- **THEN** the document's decoded IPTC Object Name is the new value, the document is dirty, and one Undo restores the previous value

#### Scenario: Cancelling applies nothing

- **WHEN** the Object Name field is changed and the dialog is rejected
- **THEN** the document's decoded IPTC Object Name is unchanged and the document is not dirtied by the dialog

#### Scenario: Accepting with no changes records nothing

- **WHEN** the dialog is accepted without changing any field
- **THEN** no edit is applied, no undo state is recorded, and the document is not dirtied by the dialog
