# file-drop-routing Specification

## Purpose
Routes operating-system file drags to the shell, opening tabs or placing files into the current document.
## Requirements
### Requirement: Operating-system file drags are accepted on the shell drop targets

The application frame SHALL install a drop handler on the document viewport (each
document's `ImageView`), the document tab strip, the menu bar, the options/tool
context bar, and the main window. On a drag enter or drag move it SHALL accept the
proposed action when the drag's mime data carries at least one local regular file,
and SHALL ignore the drag when it carries none. The handler MUST NOT accept or
consume a drag that carries no URLs, so the Layers panel's internal drag-and-drop
and document tab reordering continue to run their own handling unchanged.

#### Scenario: A local-file drag is accepted on every target

- **WHEN** an OS drag carrying a local image file enters the canvas, the tab strip, the menu bar, or the options bar
- **THEN** that target accepts the proposed action and reports the copy drop cursor

#### Scenario: A URL-less internal drag passes through

- **WHEN** a drag carrying no `text/uri-list`/URLs (the Layers panel's internal MIME or a tab reorder) enters a drop target
- **THEN** the handler ignores it, consumes nothing, and the widget's own drag handling proceeds

#### Scenario: A directory-only drag is refused

- **WHEN** a drag whose only URL is a directory enters a drop target
- **THEN** the target ignores the drag

### Requirement: A canvas drop places each file into the current document

The system SHALL place each local image file dropped on the document viewport —
the canvas or the space around it — as its own new topmost object in the open
document. It SHALL reuse the image-import
routing: a PSD/PSB file through the native `place_smart_object` path and every
other supported image through `place_image`. Each successfully placed file SHALL
record the same history state the existing Place command records. A file that
cannot be decoded SHALL be skipped without recording a state and without
disturbing the other files in the drop.

#### Scenario: Multiple files place as multiple objects

- **WHEN** two supported image files are dropped on the canvas of an open document
- **THEN** two new topmost placed objects appear in that document and exactly two Place history states are recorded

#### Scenario: A PSD dropped on the canvas stays native

- **WHEN** a `*.psd`/`*.psb` file is dropped on the canvas
- **THEN** it is placed through the native smart-object path and not through the image decode edge

#### Scenario: A file that fails to decode is skipped

- **WHEN** a drop contains one supported image and one undecodable file
- **THEN** the supported image is placed, no object or history state is recorded for the undecodable file, and the document otherwise matches the supported placement

### Requirement: Tab strip, menu bar, and options bar drops open each file as a tab

The system SHALL open each local image file dropped on the document tab strip,
the menu bar, or the options/tool context bar as its own new document tab, one
tab per file. It SHALL reuse the image-import open routing: a
PSD/PSB file through the native `read_psd` path and every other supported image
through `open_image`. A file that cannot be opened SHALL be skipped without
affecting the other files in the drop.

#### Scenario: Multiple files open as multiple tabs

- **WHEN** two supported image files are dropped on the tab strip
- **THEN** two new document tabs appear, one per file

#### Scenario: The menu bar and options bar use the open route

- **WHEN** an image file is dropped on the menu bar or the options bar
- **THEN** it opens as a new document tab

#### Scenario: A PSD dropped on a bar stays native

- **WHEN** a `*.psd`/`*.psb` file is dropped on the tab strip, menu bar, or options bar
- **THEN** it is opened through the native `read_psd` path

### Requirement: A canvas drop with no open document opens tabs

The system SHALL open each dropped file as its own new tab when a drop lands on
the document area while no document is open and therefore no canvas exists; the
drop SHALL be treated as the tab/menu/open route.

#### Scenario: No document open falls back to open

- **WHEN** two image files are dropped on the empty document area with no document open
- **THEN** two new document tabs appear, one per file

### Requirement: Non-file and undecodable drops change nothing

The system SHALL ignore a drop that carries no local regular files, only
directories, or only undecodable/non-image files: no document SHALL be created or
modified, no layer SHALL be added, no history state SHALL be recorded, and the
application SHALL NOT crash.

#### Scenario: A drag with no local files is ignored

- **WHEN** a drop carries mime data with no local file URLs
- **THEN** the document count and the active document's layers and history are unchanged

#### Scenario: An all-directory drop is ignored

- **WHEN** every URL in a drop resolves to a directory
- **THEN** no document or layer is created and no history state is recorded

#### Scenario: An undecodable-only drop is ignored

- **WHEN** every file in a drop cannot be decoded by the native reader or Qt
- **THEN** no document or layer is created and no history state is recorded

### Requirement: Existing internal drag-and-drop is preserved

The file-drop handler SHALL NOT alter the Layers panel's internal drag-and-drop
or document tab reordering. A drag carrying no URLs SHALL be left to the widget
that owns it.

#### Scenario: Layers drag still works

- **WHEN** a layer row is dragged within the Layers panel
- **THEN** the existing reorder behavior occurs and the file-drop handler does not intercept the drag

#### Scenario: Tab reordering still works

- **WHEN** a document tab is dragged to a new position in the strip
- **THEN** the documents reorder as before and the file-drop handler does not intercept the drag

