## ADDED Requirements

### Requirement: Panel set

The system SHALL provide dockable panels for Gradients, Patterns, Properties,
Adjustments, Libraries, Channels, Paths, and Actions, each with a stable
`objectName` and a CS6-appropriate empty state, in addition to the existing
Layers, History, Navigator, Color, Swatches, Info, and Histogram panels. The
placeholder panels SHALL show an explicit empty state rather than fabricated
content.

#### Scenario: Every panel exists and is dockable

- **WHEN** the frame is shown
- **THEN** each of the listed panels exists as a `QDockWidget` with its documented `objectName`

#### Scenario: Properties shows its empty state

- **WHEN** the Properties panel is shown with nothing selected
- **THEN** it displays a "No Properties" empty state

### Requirement: Right panel icon rail

The system SHALL present a narrow vertical icon rail on the right edge of the
frame with one button per collapsed panel (History, Actions, Info, Navigator,
Histogram). Each button SHALL show the panel's icon and a tooltip and SHALL be a
checkable toggle.

#### Scenario: Rail is present with its panels

- **WHEN** the frame starts
- **THEN** the right icon rail shows a button for each of History, Actions, Info, Navigator, and Histogram

### Requirement: Rail and Window menu share panel toggles

Clicking a rail button SHALL toggle its panel's visibility, and the
`Window > Panels > <name>` command for the same panel SHALL perform the same
toggle. The rail button's checked state SHALL reflect the panel's visibility,
and SHALL track visibility changes made from the menu.

#### Scenario: Rail button opens the panel

- **WHEN** a rail button is clicked and its panel was hidden
- **THEN** the panel becomes visible and the button is checked

#### Scenario: Window menu entry matches the rail

- **WHEN** `Window > Panels > <name>` is invoked
- **THEN** the panel toggles and the corresponding rail button's checked state updates

#### Scenario: Grouped panels have menu toggles

- **WHEN** `Window > Panels > Gradients` (or Patterns, Properties, Adjustments, Libraries, Channels, Paths, Actions) is invoked
- **THEN** that panel is shown or hidden
