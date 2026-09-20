## MODIFIED Requirements

### Requirement: Selection combine modes

The system SHALL support New, Add, Subtract, and Intersect combine modes for the
selection tools. The default SHALL be New, which replaces any existing selection.
While a modal selection tool gestures, holding Shift SHALL mean Add, holding Alt
SHALL mean Subtract, and holding Shift+Alt SHALL mean Intersect, but only when a
selection already exists when the gesture starts; without an existing selection
the modifiers SHALL NOT select Add/Subtract/Intersect (the options-bar mode
applies). The quick mode SHALL be decided at the start of the gesture and locked
for its whole duration until the selection commits, so releasing a modifier
mid-gesture does not change the result. The options-bar combine buttons SHALL
remain the default when no modifier is held. A gesture's combine mode SHALL be
derived from two independent inputs: the **quick mode** locked at press and the
**modifier geometry**, and the two SHALL NOT be conflated — Shift/Alt choose the
combine operation while Alt SHALL only additionally select the from-centre
(alt-pivot) geometry. Alt SHALL NOT pre-toggle the combine mode before a drag
begins, and a modifier held while merely hovering SHALL NOT change the mode. The
canvas cursor SHALL reflect the mode that a press would use: with an existing
selection and Shift or Alt held over the canvas, the canvas SHALL show the
combine cursor rather than the move-selection cursor.

#### Scenario: Add extends an existing selection

- **WHEN** Add mode is selected and a new marquee is dragged overlapping the current selection
- **THEN** the result is the union of the old and new coverage

#### Scenario: Subtract removes coverage

- **WHEN** Subtract mode is selected and a marquee is dragged over the selection
- **THEN** the result excludes the marquee coverage

#### Scenario: Intersect keeps the overlap

- **WHEN** Intersect mode is selected and a marquee overlaps the selection
- **THEN** the result is the overlap of the old and new coverage

#### Scenario: New replaces the selection

- **WHEN** New mode is selected and a marquee is dragged
- **THEN** any previous selection is discarded

#### Scenario: Shift adds mid-gesture

- **WHEN** a selection exists and a selection tool is dragged with Shift held from the first press
- **THEN** the committed result is the union of the existing selection and the dragged shape

#### Scenario: Alt subtracts mid-gesture

- **WHEN** a selection exists and a selection tool is dragged with Alt held from the first press
- **THEN** the committed result excludes the dragged shape from the existing selection

#### Scenario: Shift+Alt intersects

- **WHEN** a selection exists and a selection tool is dragged with Shift+Alt held from the first press
- **THEN** the committed result is the overlap of the existing selection and the dragged shape

#### Scenario: The quick mode is locked for the gesture

- **WHEN** a selection exists and the gesture starts with Shift held, and Shift is released before mouse-up
- **THEN** the committed result is still an Add, not a New replacement

#### Scenario: No existing selection ignores the combine modifier

- **WHEN** there is no selection and a selection tool is dragged with Alt held
- **THEN** the modifier chooses the from-centre geometry when applicable and the committed selection is created as New, not subtracted

#### Scenario: The combine cursor shows while hovering [lst_combine_cursor]

- **WHEN** a selection exists and a selection tool hovers it with Shift or Alt held, without any press
- **THEN** the canvas shows the combine/mode cursor and not the move-selection cursor

#### Scenario: Alt does not pre-toggle before a drag [lst_alt_no_pretoggle]

- **WHEN** Alt is held over the canvas with a selection tool active but no drag has started
- **THEN** the locked quick mode is unchanged and no combine operation is selected until a press captures the modifiers

### Requirement: Marquee modifier constraints and size readout

While a Rectangular or Elliptical Marquee drag is active, holding Shift SHALL
constrain the shape to a square (rectangle) or circle (ellipse), and holding Alt
SHALL treat the press point as the centre (a pivot), mirroring the region around
it on every side. Holding both SHALL produce a square/circle centred on the
press point. These constraints refine the Normal drag and SHALL combine with the
Style option. **The modifiers that govern the gesture SHALL be captured at press
(`dragMods_`) and used unchanged for the live preview geometry, the release
rasterisation, and the drag cursor, so releasing Shift or Alt partway through a
drag keeps the constrained geometry and the mode that were active at press.**
For both tools, a floating readout of the current selection size in document
pixels (`W × H`) SHALL follow the cursor during the drag and SHALL disappear on
release.

#### Scenario: Shift makes a square

- **WHEN** the Rectangular Marquee is dragged with Shift held from the first press
- **THEN** the committed selection is a square whose side equals the longer drag axis

#### Scenario: Shift makes a circle

- **WHEN** the Elliptical Marquee is dragged with Shift held
- **THEN** the committed selection is a circle inscribed in a square

#### Scenario: Alt draws from the centre

- **WHEN** the Rectangular Marquee is dragged with Alt held from a press point
- **THEN** the committed rectangle is centred on the press point and extends on all sides by the drag delta

#### Scenario: Releasing a modifier mid-drag keeps the geometry [lst_marquee_dragmods]

- **WHEN** a marquee drag starts with Shift (or Alt) held and the modifier is released before mouse-up
- **THEN** the committed selection keeps the square/centre constraint captured at press and the quick mode is unchanged

#### Scenario: The size readout tracks the drag

- **WHEN** a Marquee or Elliptical Marquee drag is in progress
- **THEN** a `W × H` pixel readout is shown next to the cursor and cleared when the drag ends

### Requirement: Selection-move hover cursor

The canvas SHALL update the move-selection cursor as the pointer moves over the
document, showing the move cursor while the pointer is inside the active
selection for a selection tool and reverting to the tool cursor when it leaves,
without requiring a mouse press. Holding Ctrl with a selection tool over the
canvas SHALL show the move cursor even before the pointer enters the selection.
**The move-selection cursor SHALL be gated on no Shift and no Alt: with an
existing selection and Shift or Alt held, the combine cursor takes precedence so
a user about to add to or subtract from the selection sees the correct cursor.
While a drag is in progress, the cursor SHALL reflect the captured `dragMods_`
rather than the live keyboard state.**

#### Scenario: Hover shows the move cursor

- **WHEN** a selection exists and the pointer is moved into it with a selection tool active
- **THEN** the move cursor is shown before any click

#### Scenario: Leaving the selection restores the tool cursor

- **WHEN** the pointer is moved from inside the selection to outside it
- **THEN** the tool's own cursor is restored without a click

#### Scenario: Ctrl previews the content move

- **WHEN** a selection exists and Ctrl is pressed with a selection tool active
- **THEN** the move cursor is shown even if the pointer is outside the selection

#### Scenario: A combine modifier beats the move cursor [lst_move_cursor_gate]

- **WHEN** a selection exists, the pointer is inside it with a selection tool active, and Shift or Alt is held
- **THEN** the combine/mode cursor is shown instead of the move-selection cursor
