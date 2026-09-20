## MODIFIED Requirements

### Requirement: Inline rename with Tab navigation

The system SHALL begin inline name editing only when the user double-clicks a
row's name, not the rest of the row. A double-click elsewhere on the row —
including the visibility, lock, chevron, and thumbnail regions — SHALL NOT begin
editing and SHALL instead invoke the layer-style affordance, which is a
documented no-op while no Layer Style dialog exists. While editing, the system
SHALL commit the edit and move to the next visible row when the user presses
`Tab`, and to the previous visible row when the user presses `Shift+Tab`. At the
last or first visible row, the commit SHALL occur without wrapping.

#### Scenario: Double-click starts editing [m39_rename]

- **WHEN** the user double-clicks a row's name
- **THEN** an editor opens on that row and the document is unchanged until the edit commits

#### Scenario: Double-click elsewhere does not rename [lpr_rename_name_only]

- **WHEN** the user double-clicks a row outside the name region
- **THEN** no inline editor opens, the name is unchanged, and the layer-style
  affordance is invoked instead

#### Scenario: Tab commits and moves down [m39_rename]

- **WHEN** the user edits a name and presses `Tab`
- **THEN** the name changes, the edit is one undo state, and the editor moves to the next visible row

#### Scenario: Shift+Tab moves up without wrapping [m39_rename]

- **WHEN** the user is editing the first visible row and presses `Shift+Tab`
- **THEN** the edit commits and editing ends without wrapping to the last row

### Requirement: Layer row badges and delegate

The system SHALL draw each row with a delegate that paints, in CS6 order, the
visibility toggle, the thumbnail (a folder glyph for a group), the name, the
color-label swatch, a clipping-mask indicator for a clipped layer, the clipping
indentation and base underline, the layer-mask thumbnail when a mask is present,
and an adjustment/style badge when adjustment content is present. A layer whose
lock state has any flag set SHALL also show a lock badge at the right side of its
row; an unlocked layer SHALL show none. The visibility toggle SHALL be an eye
icon (`layers.eyeOn`/`layers.eyeOff`) drawn slightly inset from the panel's left
edge and at the same x for every row, independent of nesting depth; the nesting
indentation SHALL apply to the thumbnail and name, not to the visibility toggle.
A layer whose color label is not `None` SHALL tint the eye gutter with that label
color behind the eye glyph, so the label reads at the left of the row as it does
in Photoshop, and the tint SHALL keep the eye glyph and any selection highlight
legible. A group with at least one child SHALL show a disclosure icon — right
when collapsed, down when expanded — at its indented position, and clicking that
icon SHALL expand or collapse the group. If an expected icon asset is unavailable,
the delegate SHALL omit that badge while keeping the row legible rather than fail.

#### Scenario: The visibility toggle is an eye icon [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its visibility toggle is drawn from the eye icon asset, and a hidden
  layer uses the off variant

#### Scenario: The eye is left-anchored for every depth [lpr_eye]

- **WHEN** a nested layer is shown under its group
- **THEN** its eye icon is at the same x as a top-level row's, while its
  thumbnail and name are indented

#### Scenario: The color label tints the eye gutter [lpr_label_tint]

- **WHEN** a layer with a non-`None` color label is shown
- **THEN** the eye gutter is filled with that label color behind the eye glyph

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

#### Scenario: A missing badge asset does not break the row [m39_badges]

- **WHEN** the adjustment badge's icon asset is missing
- **THEN** the row is still drawn and the badge is simply omitted

### Requirement: Layer grouping commands

The system SHALL offer Group Layers and Ungroup Layers from the panel, the row
context menu, the `Layer` menu, and the `Ctrl+G`/`Shift+Ctrl+G` shortcuts, acting
on the active document's currently selected layers. Group Layers SHALL wrap the
selection in one new group placed at the topmost selected layer's position,
preserving the selected layers' order, and SHALL name the group `"Group N"` where
N is one more than the highest existing `Group <number>` name. Group Layers SHALL
be refused, leaving the document unchanged, when the selection is empty, contains
the Background layer, contains a fully locked layer, or spans more than one
container. Ungroup Layers SHALL splice each selected group's children into the
parent at the group's position, preserving their order, and SHALL skip a selected
layer that is not a group. Each applied operation SHALL be one undoable step. The
keyboard shortcut path SHALL use the same selection-aware operation as the menu
path; it SHALL NOT operate on only the current row.

#### Scenario: Group a layer

- **WHEN** the user runs Group Layers on a selected layer
- **THEN** the layer is wrapped in a new group that occupies the layer's position and the wrapped layer becomes its only child

#### Scenario: Group a multi-selection [m39_multi]

- **WHEN** the user runs Group Layers on several selected layers that share a
  container
- **THEN** one group is created at the topmost selected position containing all
  of them in their existing order, in one undo step

#### Scenario: Ctrl+G groups the selection [lpr_ctrl_g]

- **WHEN** several layers are selected and `Ctrl+G` is pressed
- **THEN** the whole selection is wrapped in one group in one undo step, exactly
  as the menu Group Layers command does

#### Scenario: Group refuses the Background

- **WHEN** the selection contains the Background layer
- **THEN** Group Layers is refused and the document is unchanged

#### Scenario: Ungroup a group with children [m39_multi]

- **WHEN** the user runs Ungroup Layers on one or more selected groups with children
- **THEN** each group is replaced in place by its children in their existing order, in one undo step

#### Scenario: Ungroup skips a non-group

- **WHEN** the user runs Ungroup Layers on a selection that includes a pixel or adjustment layer
- **THEN** the non-group is left unchanged and remains selected

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
implemented (mask, link, fx) SHALL be inert. The row model SHALL advertise the
drag and drop capabilities so the view starts a drag for a draggable row rather
than falling back to rubber-band selection.

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

#### Scenario: A press-move starts a drag rather than extending a selection [lpr_drag_enabled]

- **WHEN** the user presses a draggable row and moves the pointer
- **THEN** the drag pipeline starts, the selection is not extended, and no
  rubber-band selection appears

#### Scenario: Dropping on Delete deletes the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the Delete strip button and released
- **THEN** that row is deleted in one undo step

#### Scenario: Dropping on New Layer duplicates the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the New Layer strip button and released
- **THEN** the row is duplicated in one undo step

#### Scenario: Dropping on New Group groups the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the New Group strip button and released
- **THEN** the row is wrapped in a new group in one undo step
