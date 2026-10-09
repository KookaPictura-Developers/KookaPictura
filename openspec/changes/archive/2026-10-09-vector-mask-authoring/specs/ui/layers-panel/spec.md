# Spec Delta

## MODIFIED Requirements

### Requirement: Layer mask row indicators

When a layer row carries a layer mask, the delegate SHALL draw a link glyph
between the layer thumbnail and the mask thumbnail when that mask is linked to
its layer, and SHALL draw a red cross over the mask thumbnail when the mask is
disabled. When a row carries a vector mask, the delegate SHALL likewise draw a
vector-mask thumbnail with its own link glyph when linked and red cross when
disabled. Clicking a mask's link glyph SHALL toggle that mask's linked state,
and `Shift`-clicking a mask thumbnail SHALL toggle that mask's enabled state,
through the matching bridge; those clicks SHALL be consumed so they do not start
a rename, select another row, or begin a drag. The model SHALL expose the
per-row mask and vector-mask linked and disabled states through roles populated
from the bridge.

#### Scenario: A linked mask shows the link glyph [lmk_row_link]

- **WHEN** a row whose mask is linked is shown
- **THEN** a link glyph is drawn between the layer and mask thumbnails

#### Scenario: A disabled mask shows a red cross [lmk_row_disabled]

- **WHEN** a row whose mask is disabled is shown
- **THEN** a red cross is drawn over the mask thumbnail

#### Scenario: Clicking the link glyph unlinks the mask [lmk_row_link_click]

- **WHEN** the user clicks the link glyph on a linked mask row
- **THEN** the mask is unlinked in one undoable step

#### Scenario: Shift-clicking the mask thumbnail disables it [lmk_row_shift]

- **WHEN** the user `Shift`-clicks the mask thumbnail of an enabled mask
- **THEN** the mask is disabled in one undoable step

#### Scenario: A vector mask shows its own indicator [vmk_row_vector]

- **WHEN** a row whose vector mask is linked and enabled is shown
- **THEN** a vector-mask thumbnail is drawn alongside the layer thumbnail with
  its link glyph

#### Scenario: A disabled vector mask shows a red cross [vmk_row_vector_disabled]

- **WHEN** a row whose vector mask is disabled is shown
- **THEN** a red cross is drawn over the vector-mask thumbnail

#### Scenario: Clicking the vector link glyph unlinks it [vmk_row_vector_link_click]

- **WHEN** the user clicks the vector-mask link glyph on a linked row
- **THEN** the vector mask is unlinked in one undoable step

#### Scenario: Shift-clicking the vector thumbnail disables it [vmk_row_vector_shift]

- **WHEN** the user `Shift`-clicks the vector-mask thumbnail of an enabled vector mask
- **THEN** the vector mask is disabled in one undoable step

## ADDED Requirements

### Requirement: Vector mask commands

The system SHALL provide `Layer ▸ Vector Mask` leaves for Reveal All, Hide All,
Current Path, Delete, Enable, Disable, Link, and Unlink, and a
`Layer ▸ Rasterize ▸ Vector Mask` leaf, each registered as implemented with a
frozen command id and wired to the vector-mask bridge over the active layer,
each applied action one undoable step. Add-style leaves SHALL be enabled when a
document exists and the active layer has no vector mask; Current Path SHALL also
require a non-empty work path; Delete, Enable, Disable, Link, Unlink, and
Rasterize SHALL be enabled only when the active layer has a vector mask.

#### Scenario: The add leaves light up without a vector mask [vmk_menu_add]

- **WHEN** a document's active layer has no vector mask
- **THEN** Reveal All and Hide All are enabled and Delete, Enable, Disable, Link,
  Unlink, and Rasterize are disabled

#### Scenario: The mask leaves light up with a vector mask [vmk_menu_present]

- **WHEN** the active layer carries a vector mask
- **THEN** Delete, Enable, Disable, Link, Unlink, and Rasterize are enabled and
  Reveal All and Hide All are disabled

#### Scenario: Current Path needs a work path [vmk_menu_current]

- **WHEN** the document has no work path
- **THEN** Current Path is disabled

#### Scenario: Reveal All adds and Delete removes through the menu [vmk_menu_dispatch]

- **WHEN** the user dispatches Reveal All and then Delete for the active layer
- **THEN** a vector mask is added and then removed, each in one undoable step

#### Scenario: Rasterize consumes the vector mask [vmk_menu_rasterize]

- **WHEN** the user dispatches Rasterize Vector Mask for a layer with a vector mask
- **THEN** the layer carries a layer mask instead and one undoable step is recorded
