# document-lifecycle Specification

## Purpose
TBD - created by archiving change m17-document-lifecycle. Update Purpose after archive.
## Requirements
### Requirement: New document creation
The system SHALL create a new document from a New Document dialog that specifies
name, width, height, color mode, bit depth, and background. Creation SHALL be
supported for 8-bit Grayscale and 8-bit RGB with a white or transparent
background; requests outside that set SHALL be rejected rather than produce a
document the codec cannot write. A new document SHALL be untitled, unmodified,
with empty history and no selection. A document SHALL be created only by an
explicit New, Open, or Place action; launch SHALL NOT fabricate a document, so a
normal launch starts with no document open and the document-requiring commands
disabled.

#### Scenario: Create an RGB document
- **WHEN** the user confirms New with an 8-bit RGB document, white background, and dimensions
- **THEN** a new untitled tab opens showing a document of those dimensions filled white

#### Scenario: Create a transparent document
- **WHEN** the user confirms New with a transparent background
- **THEN** the new document's initial composite is fully transparent

#### Scenario: Unsupported mode or depth is rejected
- **WHEN** a document is requested with a mode or depth the codec cannot write
- **THEN** creation fails and no document is added

#### Scenario: Launch fabricates no document [ldl_no_launch_doc]
- **WHEN** the application is launched with no document argument
- **THEN** no document exists until the user runs New, Open, or Place

#### Scenario: An explicit New creates a document after launch [ldl_new_after_launch]
- **WHEN** the user runs New after a launch with no document
- **THEN** a document is created and the document-requiring commands become
  enabled

### Requirement: Open documents
The system SHALL open a PSD/PSB file chosen from a file dialog into a new
document, and SHALL offer the most recently opened files through an Open Recent
list. A successful open SHALL leave the document unmodified with its path
recorded.

#### Scenario: Open a file adds a document
- **WHEN** the user opens a valid PSD
- **THEN** a new tab shows the composited document and the command line argument path is recorded

#### Scenario: Open failure does not add a document
- **WHEN** the chosen file cannot be read
- **THEN** no tab is added and the application remains usable

#### Scenario: Recent files
- **WHEN** a document is opened successfully
- **THEN** it appears at the front of the bounded Open Recent list

### Requirement: Save and Save As

The system SHALL write the active document's composite and layer tree through the
PSD codec, writing atomically so a failed write never truncates an existing file.
The `composite` it serializes SHALL be the active document's current rendered
composite, so a save after any edit round-trips to the edited pixels rather than
to a pre-edit merged image. The `write_psd` byte layout and its use of
`doc.composite.channels` to derive the PSD colour-plane count MUST NOT change;
an RGB document's merged composite is RGBA (four planes) after a canvas rebuild,
while a grayscale document keeps its composite colour-plane count. Save on an
untitled document SHALL behave as Save As. A successful save SHALL record the
file path and clear the modified state.

#### Scenario: Save As writes a file

- **WHEN** the user runs Save As and chooses a path
- **THEN** a valid PSD is written at that path and the document is no longer modified

#### Scenario: Save uses the recorded path

- **WHEN** a document has a path and the user runs Save
- **THEN** the file at that path is overwritten atomically

#### Scenario: Save on an untitled document prompts for a path

- **WHEN** Save is invoked on an untitled document
- **THEN** a Save As path is requested before any write

#### Scenario: Save after an edit writes the edited composite

- **WHEN** an edit changes the canvas and the document is saved and read back
- **THEN** the read-back document's composite equals the edited rendered composite, not the pre-edit merged image, and its colour-plane count matches the saved document's colour-plane count

### Requirement: Modified state tracking
The system SHALL expose `is_dirty()` for the active document. Every successful
mutating command SHALL mark the document modified; opening and saving SHALL clear
the modified state. A rejected command SHALL NOT change the modified state.

#### Scenario: A mutating command marks modified
- **WHEN** a filter, adjustment, document op, or selection command succeeds
- **THEN** the document reports modified

#### Scenario: Save clears modified
- **WHEN** a modified document is saved successfully
- **THEN** the document reports unmodified

#### Scenario: Open clears modified
- **WHEN** a document is opened
- **THEN** it reports unmodified

### Requirement: Unsaved-changes prompt
The system SHALL prompt Save / Discard / Cancel before closing, reverting, or
replacing a modified document. Cancel SHALL abort the operation and leave the
document open and modified.

#### Scenario: Discard closes without saving
- **WHEN** the user chooses Discard in the prompt
- **THEN** the document closes and no file is written

#### Scenario: Save then close
- **WHEN** the user chooses Save in the prompt
- **THEN** the document is saved and then closed

#### Scenario: Cancel aborts
- **WHEN** the user chooses Cancel
- **THEN** the document remains open and modified

### Requirement: Revert
The system SHALL revert a saved document by reloading it from its path and
discarding in-memory changes. Revert SHALL be unavailable for an untitled
document.

#### Scenario: Revert reloads from disk
- **WHEN** the user reverts a modified document with a path
- **THEN** the document matches the file on disk and reports unmodified

#### Scenario: Revert unavailable when untitled
- **WHEN** the active document has no path
- **THEN** Revert is disabled

### Requirement: Recent files persistence
The system SHALL persist the recent-files list in the session store, bounded to a
maximum length, and SHALL drop entries whose files can no longer be found.

#### Scenario: Recent list survives restart
- **WHEN** a document is opened and the application restarts
- **THEN** the file appears in the Open Recent list

#### Scenario: Missing recent entry is dropped
- **WHEN** a recent file no longer exists
- **THEN** it is not offered in Open Recent

