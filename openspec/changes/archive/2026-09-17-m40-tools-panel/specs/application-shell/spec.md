## ADDED Requirements

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be allowed only in the left and right dock areas and SHALL
support being moved, floated, and closed, but SHALL NOT be dockable at the top or
the bottom and SHALL NOT be grouped with other panels in a tab group. A drop of
the Tools panel onto a tab bar SHALL NOT tabify it; when a drop still results in
tabification, the frame SHALL re-dock the panel to its previous side as a
fallback. The panel's custom title bar SHALL remain draggable so the panel can be
moved and floated.

#### Scenario: The panel docks only left or right [m40_dock]

- **WHEN** the Tools panel's allowed areas are queried
- **THEN** only the left and right dock areas are permitted

#### Scenario: The panel can float [m40_dock]

- **WHEN** the Tools panel is dragged out of its dock area
- **THEN** it floats as an independent window and can be docked back to the left or right

#### Scenario: Tabification is refused [m40_dock]

- **WHEN** the Tools panel is dropped onto another panel's tab bar
- **THEN** it does not become a tab in that group

#### Scenario: A tabified drop falls back to a side dock [m40_dock]

- **WHEN** a drop nonetheless leaves the Tools panel tabified with another panel
- **THEN** the frame re-docks it to its previous left or right area
