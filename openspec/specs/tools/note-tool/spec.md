# tools/note-tool Specification

## Purpose
Notes: text pinned to document points, placed with the Note tool and edited in the Notes panel.

## Requirements

### Requirement: Notes

Notes SHALL be stored on `Document::annotations` as a document pixel position
and text, without limit. The Notes panel SHALL show the current note's text,
its position among the document's notes, previous/next buttons, and Delete;
text typed into it SHALL be committed as one "Edit Note" state when the editor
loses focus or the panel moves to another note, and unchanged text SHALL record
nothing.

#### Scenario: Editing a note

- **WHEN** the current note's text is changed in the Notes panel and committed
- **THEN** one "Edit Note" state is recorded and the note holds the new text

### Requirement: Note tool

The Note tool SHALL add a note at a click on empty canvas as one "New Note"
state, make a clicked note current, and delete a note on Alt-click or when
dragged off the canvas as one "Delete Note" state; a dragged note SHALL move as
one "Move Note" state on release. A note becoming current SHALL show it in the
Notes panel and make the panel visible. The options bar's Clear All SHALL delete
every note as one "Delete All Notes" state. The canvas SHALL show every note as
a note glyph with any tool active, outlining the current one.

#### Scenario: Adding and reopening notes

- **WHEN** the `annotation_tools` self-test adds a note, adds a second, and clicks the first
- **THEN** each click on empty canvas records one "New Note" state and the Notes panel shows the clicked note's text

#### Scenario: Deleting notes

- **WHEN** a note is Alt-clicked and then Clear All is used
- **THEN** the note count drops by one and then to zero, and no note is current
