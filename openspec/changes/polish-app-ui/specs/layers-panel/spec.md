## MODIFIED Requirements

### Requirement: Layer row badges and delegate

The system SHALL draw each row with a delegate that paints, in CS6 order, the
visibility toggle, the thumbnail (a folder glyph for a group), the name, a
clipping-mask indicator for a clipped layer, the clipping indentation and base
underline, the layer-mask thumbnail when a mask is present, and an
adjustment/style badge when adjustment content is present. A layer whose lock
state has any flag set SHALL also show a lock badge at the right side of its
row; an unlocked layer SHALL show none. The visibility toggle SHALL be an eye
icon (`layers.eyeOn`/`layers.eyeOff`) drawn slightly inset from the panel's left
edge and at the same x for every row, independent of nesting depth; the nesting
indentation SHALL apply to the thumbnail and name, not to the visibility toggle.
A layer whose color label is not `None` SHALL tint the **visibility toggle's own
background** (`eyeRect`) with that label color behind the eye glyph, and the
delegate SHALL NOT paint a color swatch after the name; the tint SHALL keep the
eye glyph and any selection highlight legible, and the label color elsewhere on
the row SHALL fall back to the row background. A group with at least one child
SHALL show a disclosure icon — right when collapsed, down when expanded — at its
indented position, and clicking that icon SHALL expand or collapse the group. A
**regular (non-group) layer's thumbnail SHALL be drawn over a cached two-tone
checkerboard** so transparency reads under it, while a group keeps its folder
glyph and no checkerboard. **Every thumbnail SHALL carry a 1 px black outline**,
and **when exactly one layer is active** (the same singular active-layer
resolution tool edits use) its thumbnail SHALL additionally show white 1 px
corner brackets drawn one pixel outside the outline; with zero or multiple active
layers no brackets are drawn. Row typography SHALL derive from the layer: a
`background` layer's name SHALL be italic/cursive, every other name normal, and
a layer that is a member of the frame's link set or a placed/external smart
object SHALL be underlined through the new `LayerRowLinkedRole` and
`LayerRowPlacedRole` projections. The delegate SHALL report a row height of at
least **28 px** through one named constant used by both `sizeHint` and the
vertical centring math. If an expected icon asset is unavailable, the delegate
SHALL omit that badge while keeping the row legible rather than fail.

#### Scenario: The visibility toggle is an eye icon [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its visibility toggle is drawn from the eye icon asset, and a hidden
  layer uses the off variant

#### Scenario: The eye is left-anchored for every depth [lpr_eye]

- **WHEN** a nested layer is shown under its group
- **THEN** its eye icon is at the same x as a top-level row's, while its
  thumbnail and name are indented

#### Scenario: The color label tints only the eye toggle [lpr_label_tint]

- **WHEN** a layer with a non-`None` color label is shown
- **THEN** the eye toggle's background is filled with that label color behind
  the eye glyph, no swatch is painted after the name, and the rest of the row
  background is unchanged

#### Scenario: An unlabeled layer has no gutter tint [lpr_label_tint_none]

- **WHEN** a layer with color label `None` is shown
- **THEN** its eye gutter uses the normal row background

#### Scenario: A group shows a disclosure icon [lpr_chevron]

- **WHEN** a group with children is shown collapsed and then expanded
- **THEN** it shows the right-pointing icon when collapsed and the down-pointing
  icon when expanded, and clicking the icon toggles it

#### Scenario: A masked layer shows a mask thumbnail [m39_badges]

- **WHEN** a layer carries a layer mask
- **THEN** its row shows a mask thumbnail and the mask-presence flag is set

#### Scenario: A clipped layer shows the clipping indicator [lpc_clip]

- **WHEN** a layer is clipped to the layer below it
- **THEN** its row draws the clipping-mask indicator in addition to the
  indentation and the base layer's underline

#### Scenario: A group row is expandable [m39_badges]

- **WHEN** a group has children
- **THEN** its row is marked expandable and shows an expand/collapse control

#### Scenario: A locked layer shows a right-side lock badge [lpc_lockbadge]

- **WHEN** a layer has any lock flag set
- **THEN** its row draws a lock badge on the right side, and an unlocked layer
  draws no lock badge

#### Scenario: A regular thumbnail shows a checkerboard [lpr_thumb_checker]

- **WHEN** a regular (non-group) layer with translucent pixels is shown
- **THEN** its thumbnail is drawn over a two-tone checkerboard, while a group
  row keeps its folder glyph and draws no checkerboard

#### Scenario: Thumbnails are outlined and the active layer is bracketed [lpr_thumb_bracket]

- **WHEN** a regular layer row is shown with a 1 px black thumbnail outline
- **THEN** the singular active layer's thumbnail additionally shows white
  corner brackets one pixel outside the outline, and a multi- or zero-selection
  row shows no brackets

#### Scenario: Row typography follows the layer kind [lpr_row_fonts]

- **WHEN** the rows for the Background, a linked layer, and a placed smart
  object are inspected
- **THEN** the Background name is italic/cursive, ordinary names are normal, and
  the linked and placed rows are underlined through their roles

#### Scenario: The row height has a floor [lpr_row_height]

- **WHEN** a row's size hint is read
- **THEN** its height is at least 28 px from the single named row-height
  constant

#### Scenario: A missing badge asset does not break the row [m39_badges]

- **WHEN** the adjustment badge's icon asset is missing
- **THEN** the row is still drawn and the badge is simply omitted

### Requirement: Inline rename with Tab navigation

The system SHALL begin inline name editing when the user double-clicks a row's
**name/label content band**, and SHALL NOT begin editing on the visibility,
chevron, thumbnail, lock, fx, or mask regions. The content band SHALL be the
`nameRect` geometry the delegate paints, corrected so it mirrors paint exactly
(including the +4 gap after the thumbnail and the right-edge badge/mask caps)
and is floored to a non-zero minimum width, so a click on an empty label area
still resolves to the name and not to a zero-width rect. A double-click that is
not on the eye/chevron/thumbnail/lock/fx/mask controls and not on a masked or
clipped indicator SHALL be treated as a rename for a normal layer. When the
double-clicked row is the Background layer, the system SHALL convert it to a
normal layer (through the name-and-color dialog) instead of renaming; for every
other layer it SHALL invoke the layer-style affordance, which is a documented
no-op while no Layer Style dialog exists. While editing, the system SHALL commit
the edit and move to the next visible row when the user presses `Tab`, and to
the previous visible row when the user presses `Shift+Tab`. At the last or first
visible row, the commit SHALL occur without wrapping.

#### Scenario: Double-click starts editing [m39_rename]

- **WHEN** the user double-clicks a row's name/label content band
- **THEN** an editor opens on that row and the document is unchanged until the
  edit commits

#### Scenario: Double-click anywhere in the label band renames [lpr_rename_band]

- **WHEN** the user double-clicks a non-Background row anywhere in the content
  band outside the eye/chevron/thumbnail/lock/fx/mask controls, including a
  zero-width-name row when thumbnails are off
- **THEN** the inline editor opens for that row

#### Scenario: Double-click on a control does not rename [lpr_rename_name_only]

- **WHEN** the user double-clicks the eye, chevron, thumbnail, lock, fx, or mask
  control
- **THEN** no inline editor opens and the control's own action runs

#### Scenario: Double-clicking the Background converts it [lpr_background_dblclick]

- **WHEN** the user double-clicks the Background row in the content band
- **THEN** the Background is converted through the name-and-color dialog rather
  than opening an inline editor

#### Scenario: Tab commits and moves down [m39_rename]

- **WHEN** the user edits a name and presses `Tab`
- **THEN** the name changes, the edit is one undo state, and the editor moves to
  the next visible row

#### Scenario: Shift+Tab moves up without wrapping [m39_rename]

- **WHEN** the user is editing the first visible row and presses `Shift+Tab`
- **THEN** the edit commits and editing ends without wrapping to the last row

### Requirement: Layer drag and drop

A layer row SHALL be draggable. Starting a drag SHALL NOT extend the selection
or begin a rubber-band selection. The row model SHALL implement the drag-and-drop
capability virtuals — `mimeTypes()`, `canDropMimeData()`, and
`supportedDropActions()` — so the view's `canDrop()` is true and a real drop
indicator position is computed, and the view SHALL enter the dragging state on
drag enter so the indicator paints. **The model SHALL return
`Qt::ItemIsDropEnabled` for the invalid parent index**, so a top-level row is a
valid drop target surface and Qt's `AboveItem`/`BelowItem` indicator is
available for the top level as well as for nested rows; without it the drop
indicator is empty and reordering a top-level row shows nothing. **The view SHALL
set a closed-hand cursor for the duration of `startDrag`** and restore the
previous cursor when the drag ends. The system SHALL validate a candidate drop
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

#### Scenario: A top-level sibling shows the above/below indicator [lpr_drag_top_level]

- **WHEN** a top-level row is dragged over the gap above or below a top-level
  sibling
- **THEN** the model is drop-enabled for the invalid parent, the drop indicator
  is above/below, and the reorder is accepted

#### Scenario: The drag cursor is a closed hand [lpr_drag_cursor]

- **WHEN** a row drag starts
- **THEN** the viewport cursor is a closed hand until the drag ends, then the
  previous cursor is restored

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

## ADDED Requirements

### Requirement: Nesting lock button is hidden in the panel

The Layers panel SHALL hide its nesting-lock toggle button (`lockNesting_`), so
the panel exposes only the transparency, pixel, position, and all lock controls.
Hiding the button SHALL NOT change the engine: the `NESTING` flag SHALL remain
part of the layer lock state, SHALL still refuse Group Layers and Ungroup Layers
on a nesting-locked layer, SHALL still round-trip through the PSD `lspf` block,
and SHALL still be settable and readable through the bridge. A layer already
carrying the flag SHALL still report it in its row and keep its structural
parent; the button is a presentation removal, not a feature removal.

#### Scenario: The panel has no nesting lock button [lpr_nesting_hidden]

- **WHEN** the Layers panel header lock controls are shown
- **THEN** the nesting-lock button is hidden and the other lock controls remain

#### Scenario: The engine nesting rules still hold [lpr_nesting_engine]

- **WHEN** a layer reports the `NESTING` bit programmatically and Group Layers
  is attempted
- **THEN** the operation is still refused and the PSD round-trip still preserves
  the bit
