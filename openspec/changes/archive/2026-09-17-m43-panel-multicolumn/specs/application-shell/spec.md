## MODIFIED Requirements

### Requirement: Default dock grouping and canvas colour

The system SHALL present the default workspace with the right-hand panels
hosted by the `PanelColumn` in the CS6 Essentials groups: **Color, Swatches, and
Styles**; **Adjustments**; **Layers, Channels, and Paths**; **Navigator,
Histogram, and Info**; and the iconic **History** and **Actions**. The central
area SHALL be a horizontal splitter hosting an ordered set of `PanelColumn`s
around the document tab area: zero or more columns to the left of the document
tabs and zero or more to the right, with the document tabs keeping the stretch.
A `PanelColumn` SHALL be creatable dynamically by a drop and removed when
emptied. The document canvas SHALL use the CS6 dark canvas colour, and the
document tab strip SHALL be styled to match the chrome. Panel `objectName`s SHALL
remain stable so the persisted session layout keeps working. The Tools panel
SHALL remain a left/right dock separate from the columns.

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

#### Scenario: The document tabs keep the stretch [m43_newcolumn]

- **WHEN** a new panel column is created on either side
- **THEN** the document tab area keeps the stretch and the new column takes only
  its own width

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be allowed only in the left and right dock areas and SHALL
support being moved, floated, and closed, but SHALL NOT be dockable at the top or
the bottom and SHALL NOT be grouped with other panels in a tab group. A drop of
the Tools panel onto a tab bar SHALL NOT tabify it; when a drop still results in
tabification, the frame SHALL re-dock the panel to its previous side as a
fallback. When floated, the panel SHALL size to the minimum height its content
needs rather than expanding to fill the window. The panel's custom title bar
SHALL remain draggable so the panel can be moved and floated. The panel's width
SHALL be fixed to its content width in both the one- and two-column layouts and
while floating, and dragging the dock separator SHALL NOT resize it. The panel
SHALL be placeable on either side of the workspace or beside a panel column
without breaking the fixed-width rule or the left/right-only dock contract.

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

#### Scenario: The width cannot be dragged [m43_tools]

- **WHEN** the dock separator beside the Tools panel is dragged
- **THEN** the Tools panel's width does not change and stays at its content width

#### Scenario: The fixed width holds beside a column [m43_tools]

- **WHEN** the Tools panel is docked beside a panel column in one- or two-column
  mode
- **THEN** its width is the fixed content width and the column can still be laid
  out beside it
