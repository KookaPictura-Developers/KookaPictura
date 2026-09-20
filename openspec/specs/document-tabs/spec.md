# document-tabs Specification

## Purpose
TBD - created by archiving change m17-document-lifecycle. Update Purpose after archive.
## Requirements
### Requirement: Tabbed document area
The system SHALL host each open document in its own tab with its own canvas, and
tabs SHALL be closable and switchable. Opening or creating a document SHALL add a
tab and make it active.

#### Scenario: Opening a document adds a tab
- **WHEN** a document is opened while another is already open
- **THEN** a second tab appears and each tab shows its own composited document

#### Scenario: Switching tabs changes the canvas
- **WHEN** the user selects a different tab
- **THEN** the canvas and status bar show that document

### Requirement: Active-document targeting
Menu commands, the editable dock controls, and the status bar SHALL operate on
the active document. A document with no active tab (no documents open) SHALL
leave document-requiring commands disabled.

#### Scenario: Commands target the active document
- **WHEN** the user runs a command with several documents open
- **THEN** only the active document is changed

#### Scenario: No documents open
- **WHEN** the last document is closed
- **THEN** document-requiring commands are disabled and the application does not crash

### Requirement: Tab title and modified marker
Each tab SHALL show the document's file name, or an untitled name when it has no
path, and SHALL append a modified marker while the document is dirty. The window
title SHALL track the active document. A document whose mode and bit depth are
known SHALL show the title as `name (Mode/Bits)`, for example `photo (RGB/8)`,
where `Mode` is the document's color mode (such as `RGB`, `Grayscale`, or `CMYK`)
and `Bits` is its bit depth. An opened raster file SHALL use its file's base name
as the tab's display name rather than a generated `Untitled-N` name. A document
with no name SHALL continue to show a generated untitled name.

#### Scenario: Untitled name
- **WHEN** a document has no path
- **THEN** its tab shows a generated untitled name

#### Scenario: Modified marker
- **WHEN** the active document is modified
- **THEN** its tab title carries the modified marker

#### Scenario: Marker clears on save
- **WHEN** the document is saved
- **THEN** the modified marker is removed

#### Scenario: The title carries the mode and bit depth [ldt_mode_bits]
- **WHEN** a document with a known mode and depth is shown
- **THEN** its tab title is `name (Mode/Bits)` plus the modified marker while
  dirty

#### Scenario: An opened raster keeps its display name [ldt_display_name]
- **WHEN** a raster file is opened
- **THEN** the tab shows the file's base name with its mode and bit depth rather
  than `Untitled-N`

#### Scenario: A renamed untitled document updates its title [ldt_rename]
- **WHEN** an untitled document's name changes
- **THEN** its tab title reflects the new name if the title is shown

### Requirement: Closing a tab
The system SHALL close the selected document when its tab close control is used,
after applying the unsaved-changes prompt. Closing one document SHALL NOT affect
the others.

#### Scenario: Close the active tab
- **WHEN** the user closes a tab whose document is unmodified
- **THEN** that tab is removed and the remaining documents stay open

#### Scenario: Close with unsaved changes
- **WHEN** the user closes a modified document and the prompt is cancelled
- **THEN** the tab remains open

