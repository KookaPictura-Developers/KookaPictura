## ADDED Requirements

### Requirement: Lasso resolves the combine mode at press

The freehand Lasso SHALL resolve its combine mode at press exactly as the
Rectangular/Elliptical Marquee, the Polygonal Lasso, and the Magic Wand do: from
the modifiers held at press and whether a selection already exists, falling back
to the options-bar mode when no modifier is held. The resolved mode SHALL be
captured for the whole gesture so releasing a modifier mid-drag does not change
the result, and it SHALL drive both `begin_lasso` and the drag cursor.

#### Scenario: Shift extends the selection [lasso_combine_add]

- **WHEN** a selection exists and the Lasso is dragged with Shift held from the
  first press
- **THEN** the committed result is the union of the existing selection and the
  lasso path

#### Scenario: Alt subtracts from the selection [lasso_combine_subtract]

- **WHEN** a selection exists and the Lasso is dragged with Alt held from the
  first press
- **THEN** the committed result excludes the lasso path

#### Scenario: The options-bar mode applies with no modifier [lasso_options_mode]

- **WHEN** no modifier is held and the options-bar combine mode is Subtract
- **THEN** the Lasso subtracts, not replaces

#### Scenario: The mode is locked for the gesture [lasso_combine_locked]

- **WHEN** a Lasso drag starts with Shift held and Shift is released before
  mouse-up
- **THEN** the committed result is still an Add

### Requirement: Combine drag cursor survives a modifier release

While a selection-tool gesture is in progress, the canvas cursor SHALL reflect
the combine mode captured at press (`dragMode_`), not the live keyboard state.
Releasing Shift or Alt partway through a combine drag SHALL keep the
add/subtract cursor until the gesture ends, even when the pointer is inside the
existing selection. Merely hovering an existing selection with no gesture in
progress SHALL still show the move-selection cursor.

#### Scenario: Releasing the modifier keeps the combine cursor [lst_drag_cursor_retained]

- **WHEN** a combine marquee/lasso drag starts with Shift or Alt held and the
  modifier is released before mouse-up
- **THEN** the canvas still shows the add/subtract cursor for the rest of the drag

#### Scenario: Hover still shows the move cursor [lst_drag_cursor_hover]

- **WHEN** no drag is in progress and the selection tool hovers an existing
  selection with no Shift/Alt held
- **THEN** the move-selection cursor is shown
