# Spec Delta

## MODIFIED Requirements

### Requirement: Default dock grouping and canvas colour

The system SHALL present the default workspace with the right-hand panels
hosted by the `PanelColumn` in the authentic CS6 Essentials two-column layout:
a wider main right-hand column with the groups **Color, Swatches**;
**Adjustments, Styles**; and **Layers, Channels, Paths**, and a narrower
secondary right-hand column collapsed to icons holding **History** and
**Properties**. The central area SHALL be a horizontal splitter hosting an
ordered set of `PanelColumn`s around the document tab area: zero or more columns
to the left of the document tabs and zero or more to the right, with the document
tabs keeping the stretch. A `PanelColumn` SHALL be creatable dynamically by a
drop and removed when emptied. The document canvas SHALL use the CS6 dark canvas
colour, and the document tab strip SHALL be styled to match the chrome. Panel
`objectName`s SHALL remain stable so the persisted session layout keeps working.
The Tools panel SHALL remain a left/right dock separate from the columns.

#### Scenario: Panels form the CS6 Essentials groups

- **WHEN** the frame starts with a fresh session
- **THEN** Color and Swatches share a group, Adjustments and Styles share a
  group, Layers, Channels, and Paths share a group, and a secondary iconic
  column holds History and Properties

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
