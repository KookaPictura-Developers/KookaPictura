## MODIFIED Requirements

### Requirement: Right panel icon rail

The system SHALL present a narrow vertical icon rail on the right edge of the
frame with one button per collapsed panel (History, Actions, Info, Navigator,
Histogram). Each button SHALL show that panel's icon, resolved from the frozen
`window.panels.<name>` asset id, together with a tooltip, and SHALL be a
checkable toggle. Each panel's dock tab SHALL show the same icon, and the
`Window > Panels` entry for the panel SHALL carry it as well.

#### Scenario: Rail is present with its panels

- **WHEN** the frame starts
- **THEN** the right icon rail shows a button for each of History, Actions, Info, Navigator, and Histogram

#### Scenario: Every rail button carries a panel icon

- **WHEN** the rail is built
- **THEN** each of its five buttons has a non-null icon rather than a text glyph

#### Scenario: A dock tab shows its panel icon

- **WHEN** a docked panel is shown
- **THEN** its tab carries the panel's `window.panels.<name>` icon
