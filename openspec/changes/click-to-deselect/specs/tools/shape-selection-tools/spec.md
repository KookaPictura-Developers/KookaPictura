# shape-selection-tools Specification

## ADDED Requirements

### Requirement: A click outside the selection deselects

When a selection exists and a selection-tool gesture in New mode encloses no
area, the system SHALL run Deselect (one "Deselect" history state that Reselect
can restore). A gesture encloses no area when a Rectangular or Elliptical
Marquee release has zero width or height, a Lasso release has fewer than three
points, or a Polygonal Lasso outline closes with fewer than three vertices. In
Add, Subtract, or Intersect mode such a gesture SHALL leave the selection and
history unchanged. A press inside the selection keeps starting a selection move.

#### Scenario: Marquee click outside deselects

- **WHEN** a selection exists and the Rectangular Marquee clicks outside it without dragging
- **THEN** the selection is cleared as one "Deselect" state and Reselect restores it

#### Scenario: Lasso and Polygonal Lasso clicks deselect

- **WHEN** the Lasso clicks outside the selection, or the Polygonal Lasso double-clicks one spot outside it
- **THEN** the selection is cleared as one "Deselect" state

#### Scenario: An Add-mode click keeps the selection

- **WHEN** the Rectangular Marquee Shift-clicks outside the selection
- **THEN** the selection and history are unchanged
