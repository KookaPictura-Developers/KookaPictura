## MODIFIED Requirements

### Requirement: Active tool applies its cursor
The system SHALL set the canvas cursor to the active tool's SVG cursor whenever
the active tool changes or a canvas is bound to the controller. The SVG cursor is
the fallback pointer appearance; a tool MAY additionally draw an in-canvas overlay
(such as the brush-size circle) on top of it, and the presence of an overlay SHALL
NOT change the tool's resolved cursor or hotspot.

#### Scenario: Switching tools changes the cursor
- **WHEN** the active tool changes
- **THEN** the canvas cursor becomes that tool's SVG cursor

#### Scenario: An overlay does not replace the cursor [lsc_overlay_keeps_cursor]
- **WHEN** a tool draws an in-canvas overlay such as the brush-size circle
- **THEN** the tool's SVG cursor remains the pointer appearance and the overlay
  is drawn in addition
