## ADDED Requirements

### Requirement: CS6-style chrome styling

The system SHALL style the application chrome (menu bar, options bar, Tools
panel, panel docks and their tabs and title bars, status bar, tool buttons, and
scrollbars) with a CS6-style dark stylesheet driven by the same brightness ramp
as the palette, and SHALL rebuild the stylesheet when the brightness level
changes. The bar chrome SHALL be a low-contrast dark surface distinct from the
document canvas, and panel tabs SHALL be flat with a distinctly highlighted
active tab.

#### Scenario: Chrome is styled at startup

- **WHEN** the frame is shown
- **THEN** the dock tabs, dock title bars, tool buttons, menu bar, and status bar use the dark stylesheet rather than unstyled Fusion defaults

#### Scenario: Brightness restyles the chrome

- **WHEN** the brightness level changes with `Shift+F1` or `Shift+F2`
- **THEN** the stylesheet is regenerated for the new level and the chrome colours change

### Requirement: Default dock grouping and canvas colour

The system SHALL present the default workspace with the panels grouped into
tabbed docks: Color with Swatches, Layers with History, and Navigator with Info
and Histogram. The document canvas SHALL use the CS6 dark canvas colour, and the
document tab strip SHALL be styled to match the chrome. Panel `objectName`s
SHALL remain stable so `Tab`/`Shift+Tab` and the persisted session layout keep
working.

#### Scenario: Panels share tabbed docks

- **WHEN** the frame starts with a fresh session
- **THEN** Color and Swatches share one tabbed dock, Layers and History share one, and Navigator, Info, and Histogram share one

#### Scenario: Existing panel behaviour is unchanged

- **WHEN** `Tab` is pressed
- **THEN** all panel docks are hidden and restored as before, including the grouped ones

#### Scenario: Canvas matches the chrome

- **WHEN** the frame is shown
- **THEN** the document view background is the CS6 dark canvas colour
