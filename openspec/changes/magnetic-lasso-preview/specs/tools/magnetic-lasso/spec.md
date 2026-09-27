# magnetic-lasso Specification

## ADDED Requirements

### Requirement: Magnetic Lasso live outline preview

While a Magnetic Lasso outline is open, the canvas SHALL preview it as a closed,
dashed outline made of the fastened path, the live wire, and the cursor point,
closed by a straight connector back to the origin (the first fastening point).
The origin SHALL be marked with a hollow square 5 screen pixels wide at any
zoom. Closing, cancelling, or abandoning the outline SHALL clear both the
outline and the marker.

#### Scenario: The open outline closes back to a marked origin

- **WHEN** the first point is placed and the pointer moves away
- **THEN** the preview is a closed outline and the origin marker is shown

#### Scenario: Cancelling clears the marker

- **WHEN** the outline is abandoned by deleting its first fastening point
- **THEN** neither the preview nor the origin marker remains
