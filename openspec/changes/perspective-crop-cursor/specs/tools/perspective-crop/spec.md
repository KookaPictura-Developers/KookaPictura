# perspective-crop Specification

## ADDED Requirements

### Requirement: Perspective Crop pointer

With the Perspective Crop tool active, the canvas pointer SHALL be a precise
crosshair (hot spot at its centre), and SHALL change to the move cursor while
hovering within 8 screen pixels of a staged quad's corner handle.

#### Scenario: Crosshair and corner cursor

- **WHEN** a quad is staged and the pointer hovers inside it and then over a corner
- **THEN** the pointer is the crosshair inside and the move cursor over the corner
