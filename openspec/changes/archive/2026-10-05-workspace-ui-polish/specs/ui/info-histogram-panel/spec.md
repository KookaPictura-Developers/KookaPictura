## ADDED Requirements

### Requirement: Info readout menu first-open geometry

A readout block's menu SHALL be sized from its size hint before it is shown, so
the first activation opens the menu fully at its final size rather than partially,
and later activations are unchanged. The existing beside-the-button placement
SHALL be preserved.

#### Scenario: The first click opens the menu fully [info_menu_first_open]

- **WHEN** a readout block's menu is opened for the first time
- **THEN** it is sized from its size hint and appears fully at its final size
  beside the button rather than partially open
