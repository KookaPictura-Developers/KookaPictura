## ADDED Requirements

### Requirement: Rectangular and elliptical marquee
The system SHALL create a selection from a dragged rectangle (Rectangular
Marquee) or ellipse (Elliptical Marquee) whose bounds are the drag rectangle. At
least one pixel covered at the pixel centre SHALL be selected.

#### Scenario: Drag a rectangular selection
- **WHEN** the Rectangular Marquee tool is dragged from one point to another
- **THEN** the selection covers the dragged rectangle

#### Scenario: Drag an elliptical selection
- **WHEN** the Elliptical Marquee tool is dragged from one point to another
- **THEN** the selection covers the ellipse inscribed in the dragged rectangle

### Requirement: Lasso selection
The system SHALL create a selection from a freehand closed polygon traced by the
Lasso tool, filling the polygon's interior with even-odd winding.

#### Scenario: Lasso a region
- **WHEN** the Lasso tool is dragged around a region and released
- **THEN** the selection covers the polygon interior

#### Scenario: Lasso needs three points
- **WHEN** a lasso drag produces fewer than three points
- **THEN** no selection is created

### Requirement: Quick Selection
The system SHALL grow a selection around the cursor while the Quick Selection
tool is dragged, as a per-point flood selection with the current tolerance.

#### Scenario: Drag grows the selection
- **WHEN** the Quick Selection tool is dragged across a flat-coloured region
- **THEN** the selection grows to cover pixels within the tolerance of the points sampled

### Requirement: Selection combine modes
The system SHALL support New, Add, Subtract, and Intersect combine modes for the
selection tools. The default SHALL be New, which replaces any existing selection.

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

### Requirement: Selection bounds overlay
The system SHALL show a rubber-band overlay for the selection being dragged and
SHALL expose the committed selection's integer bounds.

#### Scenario: Rubber band during the drag
- **WHEN** a selection tool is being dragged
- **THEN** an outline of the in-progress region is drawn over the canvas

#### Scenario: Committed selection bounds
- **WHEN** a selection has been committed
- **THEN** its bounding rectangle can be queried
