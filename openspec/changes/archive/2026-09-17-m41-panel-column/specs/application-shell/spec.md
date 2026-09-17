## MODIFIED Requirements

### Requirement: Default dock grouping and canvas colour

The system SHALL present the default workspace with the right-hand panels
hosted by the `PanelColumn` in the CS6 Essentials groups: **Color, Swatches, and
Styles**; **Adjustments**; **Layers, Channels, and Paths**; **Navigator,
Histogram, and Info**; and the iconic **History** and **Actions**. The document
canvas SHALL use the CS6 dark canvas colour, and the document tab strip SHALL be
styled to match the chrome. Panel `objectName`s SHALL remain stable so the
persisted session layout keeps working. The Tools panel SHALL remain a
left/right dock separate from the column.

#### Scenario: Panels form the CS6 Essentials groups

- **WHEN** the frame starts with a fresh session
- **THEN** Color, Swatches, and Styles share one group; Adjustments is its own
  group; Layers, Channels, and Paths share one; Navigator, Histogram, and Info
  share one; and History and Actions are iconic

#### Scenario: Existing panel behaviour is unchanged

- **WHEN** a panel is shown or hidden from the `Window > Panels` menu
- **THEN** its visibility toggles as before, including the grouped panels

#### Scenario: Canvas matches the chrome

- **WHEN** the frame is shown
- **THEN** the document view background is the CS6 dark canvas colour
