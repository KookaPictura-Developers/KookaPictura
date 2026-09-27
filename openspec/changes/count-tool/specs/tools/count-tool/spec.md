## ADDED Requirements

### Requirement: Count marks

`Document::annotations` SHALL hold Count marks as a list independent of color
samplers and notes, numbered in placement order. Adding a mark SHALL append it;
removing a mark SHALL shift later numbers down. Count marks SHALL be undoable
through the history snapshot and SHALL never change pixels.

#### Scenario: Independent numbered list

- **WHEN** two Count marks and one note are added
- **THEN** the Count list holds two marks, the note list one, and removing the first Count mark leaves the second at index 0

### Requirement: Count tool

The Count (Extended) tool SHALL add a numbered mark at a click on empty canvas
as one "New Count" state, move a dragged mark as one "Move Count" state on
release, and delete a mark on Alt-click or when dragged off the canvas as one
"Delete Count" state. The options bar's Clear SHALL delete every mark as one
"Clear Counts" state. The canvas SHALL show every mark as a numbered disc with
any tool active.

#### Scenario: Counting objects

- **WHEN** the `healing_tools` self-test clicks the Count tool twice and then Clears
- **THEN** two marks are shown on the overlay and Clear removes them, each step recording one state
