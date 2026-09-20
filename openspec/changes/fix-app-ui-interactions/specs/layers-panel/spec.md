## MODIFIED Requirements

### Requirement: Layer drag and drop

A layer row SHALL be draggable. Starting a drag SHALL NOT extend the selection
or begin a rubber-band selection. The row model SHALL implement the drag-and-drop
capability virtuals — `mimeTypes()`, `canDropMimeData()`, and
`supportedDropActions()` — so the view's `canDrop()` is true and a real drop
indicator position is computed, and the view SHALL enter the dragging state on
drag enter so the indicator paints. The system SHALL validate a candidate drop
during the drag and SHALL show a drop indicator only for a valid target, and it
SHALL reject an invalid drop without calling the bridge or changing the document.
Releasing a dragged row above or below another row SHALL reorder it at that
position, releasing it onto a group row SHALL reparent it as a child of that
group, and releasing it on the empty viewport below the last row SHALL move it to
the document root (out of any group), each as one undoable step. The system SHALL
refuse, leaving the document unchanged, a move of the Background layer, of a
fully-locked or nesting-locked layer, of a row onto itself, or of a row into its
own descendant. Dropping a dragged row on a bottom-strip button SHALL apply that
button's action to the dragged row: Delete deletes it, New Layer duplicates it
unless the dragged row is the Background — in which case New Layer converts the
Background to a normal layer in place — and New Group groups it; buttons whose
operation is not yet implemented (mask, link, fx) SHALL be inert. A sibling
reorder SHALL resolve to an above/below drop and SHALL NOT be mistaken for a drop
into the row.

#### Scenario: Dragging reorders a row [lpr_drag]

- **WHEN** a row is dragged and released above a sibling
- **THEN** the row moves to that position in one undo step and the drag does not
  select any other row

#### Scenario: A sibling reorder resolves at the drop [lpr_drag_reorder_mode]

- **WHEN** a row is dragged over the gap above or below a sibling
- **THEN** the drop indicator is set to the above/below position and the reorder
  is accepted rather than being treated as a drop into the sibling

#### Scenario: The view enters the dragging state [lpr_drag_state]

- **WHEN** a drag enters the layer tree's viewport
- **THEN** the view sets its dragging state so the drop indicator is painted, and
  an invalid target shows no indicator

#### Scenario: Dragging onto a group reparents [lpr_drag]

- **WHEN** a row is dragged and released onto a group row
- **THEN** the row becomes a child of that group in one undo step

#### Scenario: Dragging out of a group to the viewport [lpr_drop_out]

- **WHEN** a row inside a group is dragged and released on the empty viewport
  below the last row
- **THEN** the row is reparented to the document root in one undo step

#### Scenario: An invalid target is rejected before commit [lpr_drop_rules]

- **WHEN** a row is dragged over its own descendant, onto itself, or over a
  refusal target such as the Background
- **THEN** no valid drop indicator is shown and releasing the drag changes
  nothing and adds no history state

#### Scenario: A locked or Background row is refused [lpr_drag]

- **WHEN** the Background, a fully-locked, or a nesting-locked row is dragged
  onto another row
- **THEN** the move is refused and the document is unchanged

#### Scenario: A press-move starts a drag rather than extending a selection [lpr_drag_enabled]

- **WHEN** the user presses a draggable row and moves the pointer
- **THEN** the drag pipeline starts, the selection is not extended, and no
  rubber-band selection appears

#### Scenario: Dropping on Delete deletes the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the Delete strip button and released
- **THEN** that row is deleted in one undo step

#### Scenario: Dropping on New Layer duplicates the dragged row [lpr_drop_button]

- **WHEN** a non-Background row is dragged onto the New Layer strip button and
  released
- **THEN** the row is duplicated in one undo step

#### Scenario: Dropping the Background on New Layer converts it [lpr_drop_background_convert]

- **WHEN** the Background is dragged onto the New Layer strip button and released
- **THEN** the Background is converted to a normal layer in place in one undo
  step, rather than cloned as a locked `Background copy`

#### Scenario: Dropping on New Group groups the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the New Group strip button and released
- **THEN** the row is wrapped in a new group in one undo step

### Requirement: Inline rename with Tab navigation

The system SHALL begin inline name editing only when the user double-clicks a
row's name, not the rest of the row. A double-click elsewhere on the row —
including the visibility, lock, chevron, and thumbnail regions — SHALL NOT begin
editing. When the double-clicked row is the Background layer, the system SHALL
convert it to a normal layer; for every other layer it SHALL instead invoke the
layer-style affordance, which is a documented no-op while no Layer Style dialog
exists. While editing, the system SHALL commit the edit and move to the next
visible row when the user presses `Tab`, and to the previous visible row when the
user presses `Shift+Tab`. At the last or first visible row, the commit SHALL occur
without wrapping.

#### Scenario: Double-click starts editing [m39_rename]

- **WHEN** the user double-clicks a row's name
- **THEN** an editor opens on that row and the document is unchanged until the edit commits

#### Scenario: Double-click elsewhere does not rename [lpr_rename_name_only]

- **WHEN** the user double-clicks a non-Background row outside the name region
- **THEN** no inline editor opens, the name is unchanged, and the layer-style
  affordance is invoked instead

#### Scenario: Double-clicking the Background converts it [lpr_background_dblclick]

- **WHEN** the user double-clicks the Background row outside its name region
- **THEN** the Background is converted to a normal layer in one undo step instead
  of invoking the layer-style no-op

#### Scenario: Tab commits and moves down [m39_rename]

- **WHEN** the user edits a name and presses `Tab`
- **THEN** the name changes, the edit is one undo state, and the editor moves to the next visible row

#### Scenario: Shift+Tab moves up without wrapping [m39_rename]

- **WHEN** the user is editing the first visible row and presses `Shift+Tab`
- **THEN** the edit commits and editing ends without wrapping to the last row

## ADDED Requirements

### Requirement: Thumbnail Ctrl-click selects layer pixels

The layer row delegate SHALL expose the thumbnail's hit rectangle, and a
`Ctrl`-click inside that rectangle SHALL select that layer's pixels: the bridge
SHALL build a document-sized selection from the layer's alpha channel (`-1`,
treating a missing alpha channel as fully opaque `255`), offset by the layer's
`rect`, and combine it with the current selection using the New mode. The click
SHALL be consumed by the thumbnail and SHALL NOT start a drag, begin rename, or
toggle visibility.

#### Scenario: Ctrl-clicking a thumbnail selects its pixels [lpr_thumbnail_select]

- **WHEN** the user `Ctrl`-clicks a layer's thumbnail
- **THEN** a document-sized selection matching that layer's alpha shape becomes
  the current selection and the layer's row stays selected

#### Scenario: A thumbnail Ctrl-click is not a drag or rename [lpr_thumbnail_consumes]

- **WHEN** the user `Ctrl`-clicks a layer's thumbnail
- **THEN** no drag starts, no editor opens, and the visibility is unchanged
