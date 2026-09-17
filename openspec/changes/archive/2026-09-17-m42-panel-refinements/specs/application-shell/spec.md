## MODIFIED Requirements

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be allowed only in the left and right dock areas and SHALL
support being moved, floated, and closed, but SHALL NOT be dockable at the top or
the bottom and SHALL NOT be grouped with other panels in a tab group. A drop of
the Tools panel onto a tab bar SHALL NOT tabify it; when a drop still results in
tabification, the frame SHALL re-dock the panel to its previous side as a
fallback. When floated, the panel SHALL size to the minimum height its content
needs rather than expanding to fill the window. The panel's custom title bar
SHALL remain draggable so the panel can be moved and floated.

#### Scenario: The panel docks only left or right [m40_dock]

- **WHEN** the Tools panel's allowed areas are queried
- **THEN** only the left and right dock areas are permitted

#### Scenario: The panel can float [m40_dock]

- **WHEN** the Tools panel is dragged out of its dock area
- **THEN** it floats as an independent window and can be docked back to the left or right

#### Scenario: The floated dock hugs its content height [m42_tools]

- **WHEN** the Tools panel is floated
- **THEN** its height is the minimum its content needs and it does not expand to
  fill the window

#### Scenario: Tabification is refused [m40_dock]

- **WHEN** the Tools panel is dropped onto another panel's tab bar
- **THEN** it does not become a tab in that group

#### Scenario: A tabified drop falls back to a side dock [m40_dock]

- **WHEN** a drop nonetheless leaves the Tools panel tabified with another panel
- **THEN** the frame re-docks it to its previous left or right area

## ADDED Requirements

### Requirement: Menu bar is not overlaid

The application shell SHALL keep the menu-bar row clear of every other widget: no
toolbar, dock title bar, floating overlay, or stray child widget SHALL be drawn
over the menu bar. A persisted layout that no longer matches the current chrome
SHALL be discarded rather than restoring a widget over the menu bar.

#### Scenario: Nothing overlays the menu bar [m42_menubar]

- **WHEN** the frame is shown in its default and restored arrangements
- **THEN** no child widget's global geometry intersects the menu-bar row and only
  the menu bar is drawn there

#### Scenario: A stale layout is discarded [m42_menubar]

- **WHEN** a persisted layout that references chrome no longer present is
  restored
- **THEN** it is discarded in favour of the default arrangement and no widget is
  drawn over the menu bar
