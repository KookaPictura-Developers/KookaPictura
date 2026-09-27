# slice-tool Specification

## ADDED Requirements

### Requirement: Slice Select pointer

With the Slice Select tool active, the canvas pointer SHALL be the precise
crosshair (the marquee's cursor, hot spot at its centre) except over the
selected slice's body or handles, where the move or matching resize cursor
applies.

#### Scenario: Crosshair away from the selected slice

- **WHEN** the pointer hovers the canvas away from any selected slice
- **THEN** the pointer is the precise crosshair
