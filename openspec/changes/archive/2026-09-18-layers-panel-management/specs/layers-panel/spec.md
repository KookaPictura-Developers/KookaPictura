## ADDED Requirements

### Requirement: Layer management commands

The system SHALL expose the layer-management commands from the Layers panel
menus and the `Layer` menu, wired to the active document through the bridge.
The set SHALL include Merge Down / Merge Layers (`Ctrl+E`), Merge Visible
(`Shift+Ctrl+E`), Merge Clipping Mask, Flatten Image, Layer from Background…,
Background From Layer, Layer via Copy (`Ctrl+J`), Layer via Cut
(`Shift+Ctrl+J`), Delete Hidden Layers, Hide Layers, Select Similar, Select
Linked Layers, Link Layers, Unlink Layers, and the Rasterize entries. Each
available command SHALL apply as exactly one undoable step and the panel rows
SHALL update afterwards. A command that does not apply to the current selection
SHALL be disabled rather than shown enabled and refused, and the `Rasterize`
Type, Shape, Vector Mask, Smart Object, Video, and 3D entries SHALL stay visible
and disabled. `Alt`-clicking the New Layer/New Group panel button SHALL open
the New Layer/New Group dialog instead of creating immediately.

#### Scenario: Merge is reachable from the Layer menu and the panel menu

- **WHEN** the `Layer` menu or the panel menu is opened with a document active
- **THEN** Merge Down / Merge Layers, Merge Visible, Merge Clipping Mask, and
  Flatten Image are present and wired to the bridge

#### Scenario: A selection-dependent command is enabled correctly

- **WHEN** one layer is selected and the Layer menu is opened
- **THEN** the merge command is enabled for Merge Down, and with multiple
  selected layers it is enabled for Merge Layers

#### Scenario: Alt-click opens the dialog

- **WHEN** the user Alt-clicks the New Layer panel button
- **THEN** the New Layer/New Group dialog opens instead of a layer being created
  immediately

## MODIFIED Requirements

### Requirement: Layer drag and drop

A layer row SHALL be draggable. Starting a drag SHALL NOT extend the selection
or begin a rubber-band selection. The system SHALL validate a candidate drop
during the drag and SHALL show a drop indicator only for a valid target, and it
SHALL reject an invalid drop without calling the bridge or changing the
document. Releasing a dragged row above or below another row SHALL reorder it at
that position, releasing it onto a group row SHALL reparent it as a child of
that group, and releasing it on the empty viewport below the last row SHALL move
it to the document root (out of any group), each as one undoable step. The
system SHALL refuse, leaving the document unchanged, a move of the Background
layer, of a fully-locked or nesting-locked layer, of a row onto itself, or of a
row into its own descendant. Dropping a dragged row on a bottom-strip button
SHALL apply that button's action to the dragged row: Delete deletes it, New
Layer duplicates it, and New Group groups it; buttons whose operation is not yet
implemented (mask, link, fx) SHALL be inert.

#### Scenario: Dragging reorders a row [lpr_drag]

- **WHEN** a row is dragged and released above a sibling
- **THEN** the row moves to that position in one undo step and the drag does not
  select any other row

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

#### Scenario: Dropping on Delete deletes the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the Delete strip button and released
- **THEN** that row is deleted in one undo step

#### Scenario: Dropping on New Layer duplicates the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the New Layer strip button and released
- **THEN** the row is duplicated in one undo step

#### Scenario: Dropping on New Group groups the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the New Group strip button and released
- **THEN** the row is wrapped in a new group in one undo step
