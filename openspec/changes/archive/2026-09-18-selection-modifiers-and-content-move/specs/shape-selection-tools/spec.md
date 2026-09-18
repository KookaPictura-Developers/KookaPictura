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
remain the default when no modifier is held.

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

### Requirement: Polygonal Lasso
The system SHALL provide an enabled Polygonal Lasso tool that builds a selection
from clicked vertices: each click appends a vertex, the in-progress path SHALL
preview over the canvas, and clicking the first vertex, double-clicking, or
pressing Enter SHALL close and rasterise the polygon with even-odd winding via
the existing polygon path. The in-progress preview SHALL be a solid polyline
that runs from the first vertex through every clicked vertex to the live cursor
position (a rubber band), and SHALL NOT draw the closing edge until the polygon
commits. The tool SHALL support New, Add, Subtract, and Intersect combine modes
and a tool-time Feather radius, and SHALL NOT create a selection from fewer than
three vertices. Pressing Escape SHALL clear the in-progress path and leave the
document selection unchanged.

#### Scenario: Click vertices close into a selection
- **WHEN** the Polygonal Lasso places three or more vertices and closes on the first vertex
- **THEN** the selection covers the polygon interior

#### Scenario: Double-click closes the path
- **WHEN** the Polygonal Lasso places vertices and the user double-clicks
- **THEN** the polygon closes and commits

#### Scenario: The rubber band tracks the cursor
- **WHEN** the Polygonal Lasso has clicked one or more vertices and the pointer moves without clicking
- **THEN** the preview is a solid open path from the first vertex through the clicked vertices to the pointer position

#### Scenario: Too few vertices create nothing
- **WHEN** the in-progress path has fewer than three vertices and is closed
- **THEN** no selection is created and no history state is recorded

#### Scenario: Escape cancels without changing the selection
- **WHEN** Escape is pressed during a polygonal trace
- **THEN** the in-progress path is discarded and the document selection is unchanged

## ADDED Requirements

### Requirement: Marquee modifier constraints and size readout
While a Rectangular or Elliptical Marquee drag is active, holding Shift SHALL
constrain the shape to a square (rectangle) or circle (ellipse), and holding Alt
SHALL treat the press point as the centre (a pivot), mirroring the region around
it on every side. Holding both SHALL produce a square/circle centred on the
press point. These constraints refine the Normal drag and SHALL combine with the
Style option. For both tools, a floating readout of the current selection size
in document pixels (`W × H`) SHALL follow the cursor during the drag and SHALL
disappear on release.

#### Scenario: Shift makes a square
- **WHEN** the Rectangular Marquee is dragged with Shift held from the first press
- **THEN** the committed selection is a square whose side equals the longer drag axis

#### Scenario: Shift makes a circle
- **WHEN** the Elliptical Marquee is dragged with Shift held
- **THEN** the committed selection is a circle inscribed in a square

#### Scenario: Alt draws from the centre
- **WHEN** the Rectangular Marquee is dragged with Alt held from a press point
- **THEN** the committed rectangle is centred on the press point and extends on all sides by the drag delta

#### Scenario: The size readout tracks the drag
- **WHEN** a Marquee or Elliptical Marquee drag is in progress
- **THEN** a `W × H` pixel readout is shown next to the cursor and cleared when the drag ends

### Requirement: Selection-move hover cursor
The canvas SHALL update the move-selection cursor as the pointer moves over the
document, showing the move cursor while the pointer is inside the active
selection for a selection tool and reverting to the tool cursor when it leaves,
without requiring a mouse press. Holding Ctrl with a selection tool over the
canvas SHALL show the move cursor even before the pointer enters the selection.

#### Scenario: Hover shows the move cursor
- **WHEN** a selection exists and the pointer is moved into it with a selection tool active
- **THEN** the move cursor is shown before any click

#### Scenario: Leaving the selection restores the tool cursor
- **WHEN** the pointer is moved from inside the selection to outside it
- **THEN** the tool's own cursor is restored without a click

#### Scenario: Ctrl previews the content move
- **WHEN** a selection exists and Ctrl is pressed with a selection tool active
- **THEN** the move cursor is shown even if the pointer is outside the selection
