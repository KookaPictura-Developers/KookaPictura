# slice-tool Specification

## ADDED Requirements

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
