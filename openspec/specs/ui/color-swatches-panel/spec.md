# color-swatches-panel Specification

## Purpose
Foreground and background colour state, the Color panel controls, and the Color and Swatches dock.
## Requirements
### Requirement: Foreground and background colour state
The system SHALL hold a foreground and a background colour for the application.
The Eyedropper tool SHALL set the foreground colour, and the Color and Swatches
panels SHALL read and write the same state.

#### Scenario: Eyedropper sets the foreground
- **WHEN** the Eyedropper samples a pixel
- **THEN** the foreground colour becomes the sampled colour

#### Scenario: Panel edit updates the state
- **WHEN** the user changes the colour in the Color panel
- **THEN** the foreground colour changes and any bound display updates

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

### Requirement: Default swatch grid
The system SHALL provide a Swatches panel with a default swatch grid; clicking a
swatch SHALL set the foreground colour.

#### Scenario: Click a swatch
- **WHEN** the user clicks a swatch
- **THEN** the foreground colour becomes that swatch colour

### Requirement: Color and Swatches dock and toggle
The system SHALL host the Color and Swatches panels in registered docks with
stable `objectName`s and SHALL expose `Window > Panels > Color` and
`Window > Panels > Swatches` toggles.

#### Scenario: Toggle the Color panel
- **WHEN** the user toggles Color from the Window menu
- **THEN** the panel is shown or hidden

