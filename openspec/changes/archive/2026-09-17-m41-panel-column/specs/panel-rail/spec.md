## MODIFIED Requirements

### Requirement: Panel set

The system SHALL provide panels for Gradients, Patterns, Properties,
Adjustments, Libraries, Channels, Paths, and Actions, each with a stable
`objectName` and a CS6-appropriate empty state, in addition to the existing
Layers, History, Navigator, Color, Swatches, Info, and Histogram panels. The
panels SHALL be plain `QWidget` content widgets hosted by the `PanelColumn`
rather than `QDockWidget`s. The default CS6 Essentials set SHALL add a `Styles`
panel in the former Gradients/Patterns slot and SHALL fold the Properties
content into `Adjustments`; the Gradients, Patterns, Properties, and Libraries
panels SHALL remain registered and reachable from `Window > Panels` but SHALL NOT
be in a default visible group. The placeholder panels SHALL show an explicit
empty state rather than fabricated content.

#### Scenario: Every panel exists as a content widget

- **WHEN** the frame is shown
- **THEN** each of the listed panels exists as a `QWidget` with its documented `objectName` and is hosted by the `PanelColumn`

#### Scenario: Properties shows its empty state

- **WHEN** the Properties panel is shown with nothing selected
- **THEN** it displays a "No Properties" empty state

#### Scenario: Styles takes the former placeholder slot

- **WHEN** the frame starts with a fresh session
- **THEN** the Color/Swatches group's third tab is `Styles`

### Requirement: Rail and Window menu share panel toggles

The `Window > Panels > <name>` command SHALL toggle its panel's visibility and
SHALL be the single command path for panel visibility. The panel's registry
visibility SHALL reflect the command's state, and SHALL track visibility changes
made from the column. The former rail buttons SHALL no longer exist.

#### Scenario: Window menu entry toggles the panel

- **WHEN** `Window > Panels > <name>` is invoked
- **THEN** the panel's visibility toggles and the column reflects the new state

#### Scenario: Grouped panels have menu toggles

- **WHEN** `Window > Panels > Gradients` (or Patterns, Properties, Adjustments, Libraries, Channels, Paths, Actions) is invoked
- **THEN** that panel is shown or hidden

## REMOVED Requirements

### Requirement: Right panel icon rail

**Reason**: The far-right `PanelRail` toolbar is not part of Photoshop CS6 and
is replaced by the `PanelColumn`'s built-in width toggle and compact/iconic
strip (`panel-column`). Its five buttons duplicated the `Window > Panels`
toggles, which already own the command path.

**Migration**: Use the `Window > Panels > <name>` toggles to show or hide a
panel. Use the `panelColumnToggle` control to switch the column between `normal`
and `iconic` mode, where a panel icon opens the same panel in a `Qt::Popup`
flyout. The M24 self-test assertions that read `PanelRail` (`m24_rail`) are
retired and replaced by the M41 `m41_rail` check.
