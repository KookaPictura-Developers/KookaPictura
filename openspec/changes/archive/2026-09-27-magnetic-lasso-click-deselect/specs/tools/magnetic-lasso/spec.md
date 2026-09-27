# magnetic-lasso Specification

## ADDED Requirements

### Requirement: Magnetic Lasso click deselects

When a selection exists and a Magnetic Lasso outline in New mode closes with
fewer than three points (for example a double-click on one spot), the system
SHALL run Deselect as one "Deselect" history state. In Add, Subtract, or
Intersect mode such an outline SHALL leave the selection and history unchanged.

#### Scenario: A double-click outside the selection deselects

- **WHEN** a selection exists and the Magnetic Lasso double-clicks one spot outside it
- **THEN** the selection is cleared as one "Deselect" state
