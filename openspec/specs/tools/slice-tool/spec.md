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
