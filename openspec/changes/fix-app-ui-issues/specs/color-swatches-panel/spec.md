## MODIFIED Requirements

### Requirement: Color panel controls
The system SHALL provide RGB and HSB sliders, a hexadecimal field, and a colour
spectrum, and SHALL keep them synchronised with the foreground colour. Each
colour slider SHALL use a tracking slider so pressing on the groove and dragging
changes the channel continuously, rather than page-stepping or jumping to the
clicked position and stopping there.

#### Scenario: Slider and hex stay in sync
- **WHEN** the user changes any colour control
- **THEN** the other controls and the foreground swatch reflect the same colour

#### Scenario: Dragging a colour slider tracks continuously [lcs_slider_track]
- **WHEN** the user presses a Color panel slider's groove and drags without
  releasing
- **THEN** that channel follows the pointer continuously for the whole drag and
  the other controls update live
