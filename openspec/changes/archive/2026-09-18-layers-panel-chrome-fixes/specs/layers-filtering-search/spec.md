## MODIFIED Requirements

### Requirement: Filter row and controls

The Layers panel SHALL provide a filter/search row at the top of the header,
containing a dimension popup, a criteria control, and an on/off lightswitch
toggle. The dimension popup SHALL offer Name, Kind, Effect, Mode, Attribute, and
Color, defaulting to Kind. Selecting a dimension SHALL swap the criteria control
to that dimension's editor: Name is a free-text field, Kind is a set of toggle
buttons (multi-select), and Effect, Mode, Attribute, and Color are value menus.
The Effect dimension SHALL be present but disabled, with a "not implemented
yet" tooltip, until layer effects exist; the other five dimensions SHALL work
against the data the row projection already exposes. The toggle SHALL be shown
as a lightswitch icon, SHALL default to on, and SHALL enable or disable the
predicate without discarding the chosen dimension or criteria. With no criterion
chosen the enabled toggle SHALL leave every row visible.

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

### Requirement: View-only, live, and transient

Filtering SHALL be a view-level predicate: it SHALL NOT mutate the document,
SHALL NOT add a history state, and SHALL NOT be serialized or persisted. The
filtered view SHALL re-evaluate when the document changes — a rename, a
visibility, blend-mode, lock, or color-label change, or a structural change
SHALL add or remove rows from the filtered view as the predicate dictates.
Toggling the filter off SHALL restore the full tree, and switching documents
SHALL reset the filter to its default (Kind, on) without losing document state;
with no criterion active the reset shows the full tree.

#### Scenario: Filtering adds no history [lfs_toggle]

- **WHEN** the filter is enabled, changed, and disabled
- **THEN** the history count is unchanged and the document is not marked
  modified by the filtering itself

#### Scenario: A rename updates the filtered view [lfs_live]

- **WHEN** a layer is renamed to match the active Name filter
- **THEN** the layer appears in the filtered view without re-entering the filter

#### Scenario: Toggling off restores every row [lfs_toggle]

- **WHEN** the on/off switch is turned off after filtering
- **THEN** the full layer tree is shown again with the prior expansion preserved

#### Scenario: Switching documents resets the filter [lfs_reset]

- **WHEN** the active document changes while a filter is active
- **THEN** the filter returns to Kind/on with no criterion and the new
  document's full tree is shown
