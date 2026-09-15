## MODIFIED Requirements

### Requirement: Default dock grouping and canvas colour

The system SHALL present the default workspace with the panels grouped into
three CS6 tabbed docks: **Color, Swatches, Gradients, and Patterns**;
**Properties, Adjustments, and Libraries**; and **Layers, Channels, and Paths**.
The document canvas SHALL use the CS6 dark canvas colour, and the document tab
strip SHALL be styled to match the chrome. Panel `objectName`s SHALL remain
stable so `Tab`/`Shift+Tab` and the persisted session layout keep working.

#### Scenario: Panels share tabbed docks

- **WHEN** the frame starts with a fresh session
- **THEN** Color, Swatches, Gradients, and Patterns share one tabbed dock; Properties, Adjustments, and Libraries share one; and Layers, Channels, and Paths share one

#### Scenario: Existing panel behaviour is unchanged

- **WHEN** `Tab` is pressed
- **THEN** all panel docks are hidden and restored as before, including the grouped ones

#### Scenario: Canvas matches the chrome

- **WHEN** the frame is shown
- **THEN** the document view background is the CS6 dark canvas colour
