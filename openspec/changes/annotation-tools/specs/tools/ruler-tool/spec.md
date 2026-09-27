## ADDED Requirements

### Requirement: Ruler measurement

`pictura_core::Ruler::measure` SHALL report the start X and Y, the width and
height from start to end, the length D1, and the angle A in degrees
anticlockwise from east as seen on screen, so a line running down-right reads
negative. `Ruler::constrained` SHALL snap the end to the nearest 45° ray from
the start.

#### Scenario: A 3-4-5 line

- **WHEN** a ruler runs from (0, 0) to (3, 4)
- **THEN** W is 3, H is 4, D1 is 5, and A is -53.13°

#### Scenario: Shift snaps to 45°

- **WHEN** the end is at (10, 1) from a start at the origin, constrained
- **THEN** the end snaps to (10, 0)

### Requirement: Ruler tool

The Ruler tool SHALL draw a measuring line by dragging (Shift snaps it to 45°),
move the nearer end when a drag starts on an end, and move the whole line when
a drag starts on it; a click without a drag SHALL leave no line. The line SHALL
be view state: never recorded in history nor saved. The options bar SHALL show
X, Y, W, H, A, and D1 for the line and a Clear that removes it. The canvas SHALL
show the line only while the Ruler is active.

#### Scenario: Measuring without history

- **WHEN** the `annotation_tools` self-test drags a ruler from (0, 0) to (3, 4)
- **THEN** the options bar reads A -53.1° and D1 5.0 and no history state is added

#### Scenario: The line follows the tool

- **WHEN** another tool is selected and the Ruler is selected again
- **THEN** the line is hidden and then shown again
