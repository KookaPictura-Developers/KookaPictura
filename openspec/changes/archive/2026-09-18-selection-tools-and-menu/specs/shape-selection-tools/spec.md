## MODIFIED Requirements

### Requirement: Rectangular and elliptical marquee
The system SHALL create a selection from a dragged rectangle (Rectangular
Marquee) or ellipse (Elliptical Marquee) whose bounds are the drag rectangle. At
least one pixel covered at the pixel centre SHALL be selected. Both tools SHALL
be enabled in the toolbox, SHALL support New, Add, Subtract, and Intersect
combine modes, and SHALL apply a tool-time Feather radius (0-250 px, decimal) to
the rasterised shape before combining. The Style option SHALL constrain the drag
geometry before rasterisation: Normal follows the drag, Fixed Ratio keeps the
entered width-to-height ratio, and Fixed Size places the entered pixel size
centred on the mousedown. Anti-alias SHALL be offered only for the Elliptical
Marquee; because the engine rasteriser is binary, the control SHALL be visible
but disabled with a reason rather than silently ignored. Fixed-ratio and
fixed-size defaults, and non-pixel size units, are inferred and MAY be deferred.

#### Scenario: Drag a rectangular selection
- **WHEN** the Rectangular Marquee tool is dragged from one point to another
- **THEN** the selection covers the dragged rectangle

#### Scenario: Drag an elliptical selection
- **WHEN** the Elliptical Marquee tool is dragged from one point to another
- **THEN** the selection covers the ellipse inscribed in the dragged rectangle

#### Scenario: Tool-time feather softens the boundary
- **WHEN** the Elliptical Marquee is dragged with Feather greater than zero
- **THEN** the committed selection has boundary pixels strictly between 0 and 255

#### Scenario: Fixed ratio constrains the geometry
- **WHEN** Style is Fixed Ratio with a 2:1 ratio and the tool is dragged
- **THEN** the committed selection's width-to-height ratio is 2 within 1 px

#### Scenario: Fixed size ignores the drag extent
- **WHEN** Style is Fixed Size with a 100x50 size and the tool is dragged any distance
- **THEN** the committed selection is 100x50 centred on the mousedown

#### Scenario: Anti-alias is honest about the ceiling
- **WHEN** the Elliptical Marquee options are inspected
- **THEN** the Anti-alias control is visible and disabled with a reason that the rasteriser is binary

## ADDED Requirements

### Requirement: Polygonal Lasso
The system SHALL provide an enabled Polygonal Lasso tool that builds a selection
from clicked vertices: each click appends a vertex, the in-progress path SHALL
preview over the canvas, and clicking the first vertex, double-clicking, or
pressing Enter SHALL close and rasterise the polygon with even-odd winding via
the existing polygon path. The tool SHALL support New, Add, Subtract, and
Intersect combine modes and a tool-time Feather radius, and SHALL NOT create a
selection from fewer than three vertices. Pressing Escape SHALL clear the
in-progress path and leave the document selection unchanged.

#### Scenario: Click vertices close into a selection
- **WHEN** the Polygonal Lasso places three or more vertices and closes on the first vertex
- **THEN** the selection covers the polygon interior

#### Scenario: Double-click closes the path
- **WHEN** the Polygonal Lasso places vertices and the user double-clicks
- **THEN** the polygon closes and commits

#### Scenario: Too few vertices create nothing
- **WHEN** the in-progress path has fewer than three vertices and is closed
- **THEN** no selection is created and no history state is recorded

#### Scenario: Escape cancels without changing the selection
- **WHEN** Escape is pressed during a polygonal trace
- **THEN** the in-progress path is discarded and the document selection is unchanged

### Requirement: Selection tool options bars
The system SHALL show context-sensitive options for each enabled selection tool:
combine mode (New/Add/Subtract/Intersect) and Feather for the marquee and lasso
tools, Tolerance and Contiguous for the Magic Wand, and mode plus Tolerance for
Quick Selection. Options that the engine does not model SHALL be visible and
disabled with a stated reason: Anti-alias for the ellipse, lasso, and wand;
Sample All Layers for the wand and Quick Selection; and Auto-Enhance for Quick
Selection. Quick Selection SHALL offer New, Add, and Subtract only, with no
Intersect mode.

#### Scenario: The wand options bar exposes the engine parameters
- **WHEN** the Magic Wand tool is active
- **THEN** the options bar shows combine mode, Tolerance, Contiguous, Anti-alias, and Sample All Layers

#### Scenario: Contiguous switches the wand mode
- **WHEN** Contiguous is unchecked and the wand clicks a colour region
- **THEN** every matching pixel in the image is selected, not only the connected region

#### Scenario: Unmodelled options are disabled with a reason
- **WHEN** the wand or Quick Selection options bar is shown
- **THEN** Anti-alias, Sample All Layers, and Auto-Enhance are disabled and carry a reason

#### Scenario: Quick Selection has no Intersect
- **WHEN** the Quick Selection options bar is shown
- **THEN** only New, Add, and Subtract are present

### Requirement: Deferred selection tools stay visible and disabled
The Magnetic Lasso SHALL remain visible in the Lasso tool group but disabled,
with a documented reason that the engine has no edge-map or fastening-point
tracker. Enabling the Elliptical Marquee, Polygonal Lasso, and Magic Wand MUST
NOT change the disabled state of the Magnetic Lasso or any other unimplemented
tool.

#### Scenario: Magnetic Lasso cannot be activated
- **WHEN** the user attempts to activate the Magnetic Lasso
- **THEN** the tool is not activated and the active tool is unchanged
