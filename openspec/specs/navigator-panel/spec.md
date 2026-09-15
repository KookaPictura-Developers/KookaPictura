# navigator-panel Specification

## Purpose
TBD - created by archiving change m20-panels. Update Purpose after archive.
## Requirements
### Requirement: Navigator thumbnail and proxy
The system SHALL show a thumbnail of the active document and a rectangle marking
the portion currently visible in the canvas. The proxy SHALL reflect the canvas
zoom and pan and SHALL update when either changes.

#### Scenario: Proxy reflects the view
- **WHEN** the canvas is zoomed or panned
- **THEN** the Navigator's proxy rectangle changes to match the visible region

#### Scenario: No document
- **WHEN** no document is open
- **THEN** the Navigator shows an empty thumbnail and does not crash

### Requirement: Navigator zoom controls
The system SHALL provide a zoom slider spanning 0.01x to 32x, a Fit control, and
a 100% control that drive the canvas magnification.

#### Scenario: Slider sets zoom
- **WHEN** the user moves the Navigator zoom slider
- **THEN** the canvas magnification follows

#### Scenario: Fit and 100%
- **WHEN** the user activates Fit or 100%
- **THEN** the canvas fits the document or returns to actual pixels respectively

### Requirement: Navigator dock and toggle
The system SHALL host the Navigator in a registered dock with a stable
`objectName` and SHALL expose a `Window > Panels > Navigator` toggle.

#### Scenario: Toggle the Navigator
- **WHEN** the user toggles Navigator from the Window menu
- **THEN** the panel is shown or hidden

