## MODIFIED Requirements

### Requirement: Navigator zoom controls
The system SHALL provide a zoom slider spanning 0.01x to 32x, a Fit control, and
a 100% control that drive the canvas magnification. The zoom slider SHALL use a
tracking slider so pressing on the groove and dragging changes the zoom
continuously, rather than page-stepping or jumping to the clicked position and
stopping there.

#### Scenario: Slider sets zoom
- **WHEN** the user moves the Navigator zoom slider
- **THEN** the canvas magnification follows

#### Scenario: Dragging the slider tracks continuously [lnav_slider_track]
- **WHEN** the user presses the Navigator zoom slider's groove and drags without
  releasing
- **THEN** the canvas magnification follows the pointer continuously for the
  whole drag

#### Scenario: Fit and 100%
- **WHEN** the user activates Fit or 100%
- **THEN** the canvas fits the document or returns to actual pixels respectively
