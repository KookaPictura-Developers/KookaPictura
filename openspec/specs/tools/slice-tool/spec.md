# tools/slice-tool Specification

## Purpose
Web-export slices: user slices drawn with the Slice tool and the auto slices that tile the rest of the canvas.

## Requirements

### Requirement: User and auto slices

User slices SHALL be stored on `Document::slices`. `pictura_render::resolve_slices`
SHALL return the user slices clipped to the canvas (skipping, but keeping the
index of, any entirely off-canvas) plus auto slices that tile the rest of the
canvas without overlap, merging runs along a row, and SHALL number all of them
from 1 in reading order. An unsliced document SHALL resolve to one auto slice.
`add_slice` SHALL refuse an empty rect.

#### Scenario: Slices tile the canvas

- **WHEN** two user slices are added
- **THEN** the resolved slices cover the canvas area exactly with no overlap

#### Scenario: An empty band merges

- **WHEN** one user slice sits in the middle of the top edge
- **THEN** the band beneath it is a single full-width auto slice

### Requirement: Slice tool

The Slice tool SHALL add a user slice from a drag of at least 1×1 pixel as one
"Slice" history state; a click SHALL add nothing. While the tool is active the
canvas SHALL show every resolved slice (user slices solid blue, auto slices
dotted grey, each with a numbered badge when it fits) and SHALL re-read them
after every edit or undo; leaving the tool SHALL hide them.

#### Scenario: Drawing and undoing a slice

- **WHEN** the `crop_group` self-test drags a slice, clicks once, and undoes
- **THEN** the drag adds one "Slice" state and a numbered user slice, the click adds none, and after Undo the overlay shows the single auto slice

#### Scenario: The overlay follows the tool

- **WHEN** another tool is selected
- **THEN** the slice overlay is hidden

### Requirement: Editing user slices

`pictura_render::set_slice(doc, index, rect)` SHALL replace user slice `index`
and `remove_slice(doc, index)` SHALL delete it; both SHALL refuse without change
an out-of-range index, and `set_slice` SHALL refuse an empty rect.

#### Scenario: Set and remove address the right slice

- **WHEN** slice 1 of two is set to a new rect and slice 0 is then removed
- **THEN** the remaining slice is the new rect, and an out-of-range index or empty rect is refused

### Requirement: Slice Select tool

The Slice Select tool SHALL select the user slice under a click (user slices
before auto slices) and mark it with orange handles; a click on an auto slice
or empty canvas SHALL deselect. Dragging inside the selected slice SHALL move
it and dragging an edge or corner handle (within 6 screen pixels) SHALL resize
it, updating the canvas live and recording one "Edit Slice" state on release
when the rect changed. Delete or Backspace SHALL remove the selected slice as
one "Delete Slice" state; Escape SHALL deselect. The pointer SHALL show the move
or matching resize cursor over the selected slice's body or handles.

#### Scenario: Select, move, resize, and delete

- **WHEN** the `crop_group` self-test selects a 16×16 user slice, drags it by (1, 1), drags its right edge 4 px inward, presses Escape, reselects it, and presses Delete
- **THEN** the move and the resize each record one "Edit Slice" state with the expected rects, Escape clears the selection, and Delete records one "Delete Slice" state leaving only the auto slice
