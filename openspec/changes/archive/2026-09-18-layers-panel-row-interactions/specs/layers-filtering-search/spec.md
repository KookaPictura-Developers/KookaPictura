## MODIFIED Requirements

### Requirement: Filter row and controls

The Layers panel SHALL provide a filter/search row at the top of the header,
containing a dimension popup, a criteria control, and an on/off lightswitch
toggle. The dimension popup SHALL offer Name, Kind, Effect, Mode, Attribute, and
Color, defaulting to Kind. Selecting a dimension SHALL swap the criteria control
to that dimension's editor: Name is a free-text field, Kind is a set of toggle
buttons (multi-select), and Effect, Mode, Attribute, and Color are value menus.
Each Kind toggle button SHALL carry an icon for its kind — pixel, adjustment,
group, or background — with the kind name as its tooltip. The Effect dimension
SHALL be present but disabled, with a "not implemented yet" tooltip, until layer
effects exist; the other five dimensions SHALL work against the data the row
projection already exposes. The toggle SHALL be shown as a lightswitch icon,
SHALL default to on, and SHALL enable or disable the predicate without
discarding the chosen dimension or criteria. With no criterion chosen the
enabled toggle SHALL leave every row visible.

#### Scenario: The Kind buttons carry icons [lfs_row]

- **WHEN** the Kind dimension is shown
- **THEN** each of its toggle buttons displays a kind icon and its tooltip is
  the kind name

#### Scenario: The row defaults to the Kind dimension with the toggle on [lfs_row]

- **WHEN** the Layers panel is shown
- **THEN** the filter/search row is present, its dimension popup reads Kind, and
  the lightswitch toggle is on

#### Scenario: The toggle is a lightswitch icon [lfs_row]

- **WHEN** the filter/search row is shown
- **THEN** the on/off control is an icon button (a lightswitch) rather than a
  text button, and its icon reflects the on/off state

#### Scenario: Choosing a dimension swaps the criteria control [lfs_row]

- **WHEN** the user picks Name from the dimension popup
- **THEN** the criteria control becomes a text field, and picking Mode makes it
  a blend-mode menu

#### Scenario: The Effect dimension is disabled [lfs_row]

- **WHEN** the user opens the dimension popup before layer effects exist
- **THEN** the Effect entry is disabled and selecting it leaves the filter
  inactive

#### Scenario: An on-by-default toggle with no criterion shows every row [lfs_row]

- **WHEN** the panel is shown with the lightswitch on and no criterion chosen
- **THEN** every layer row is visible
