# layers-filtering-search Specification

## Purpose
TBD - created by archiving change layers-filtering-search. Update Purpose after archive.
## Requirements
### Requirement: Filter row and controls

The Layers panel SHALL provide a filter/search row above the blend/opacity
header, containing a dimension popup, a criteria control, and an on/off switch.
The dimension popup SHALL offer Name, Kind, Effect, Mode, Attribute, and Color,
defaulting to Kind. Selecting a dimension SHALL swap the criteria control to
that dimension's editor: Name is a free-text field, Kind is a set of toggle
buttons (multi-select), and Effect, Mode, Attribute, and Color are value menus.
The Effect dimension SHALL be present but disabled, with a "not implemented
yet" tooltip, until layer effects exist; the other five dimensions SHALL work
against the data the row projection already exposes. The on/off switch SHALL
enable or disable the predicate without discarding the chosen dimension or
criteria.

#### Scenario: The row defaults to the Kind dimension [lfs_row]

- **WHEN** the Layers panel is shown
- **THEN** the filter/search row is present, its dimension popup reads Kind, and
  the on/off switch is off

#### Scenario: Choosing a dimension swaps the criteria control [lfs_row]

- **WHEN** the user picks Name from the dimension popup
- **THEN** the criteria control becomes a text field, and picking Mode makes it
  a blend-mode menu

#### Scenario: The Effect dimension is disabled [lfs_row]

- **WHEN** the user opens the dimension popup before layer effects exist
- **THEN** the Effect entry is disabled and selecting it leaves the filter
  inactive

### Requirement: Filter predicate

The system SHALL match a layer against the active filter as follows. Name SHALL
match as a case-insensitive substring of the layer or group name. Kind SHALL
match when the layer's kind is one of the selected kinds, where the selectable
kinds are exactly those the model exposes (pixel, adjustment, group,
background). Mode SHALL match the layer's blend-mode key exactly. Color SHALL
match the layer's color label index exactly. Attribute SHALL match one of
Visible, Hidden, Locked, Has Mask, or Clipped against the row's state. Multiple
active criteria SHALL be combined with AND, and the selected Kind values with
OR. A dimension with no value selected or typed SHALL be inactive. A filter
that matches no layer SHALL show an empty list without changing the document.

#### Scenario: Name narrows to a substring [lfs_name]

- **WHEN** the Name filter is enabled with `sky`
- **THEN** only layers and groups whose names contain `sky` (case-insensitively)
  are shown, and every other layer is hidden from the panel

#### Scenario: Kind is a multi-select [lfs_kind]

- **WHEN** the Kind filter has pixel and adjustment selected
- **THEN** pixel and adjustment layers are shown and group and background layers
  are hidden

#### Scenario: Mode matches the blend key [lfs_mode]

- **WHEN** the Mode filter is enabled with Multiply
- **THEN** only layers whose blend mode is Multiply are shown

#### Scenario: Color matches the label [lfs_color]

- **WHEN** the Color filter is enabled with Red
- **THEN** only layers whose color label is Red are shown

#### Scenario: Criteria combine with AND [lfs_mode]

- **WHEN** Name `shadow` and Mode Multiply are both active
- **THEN** only layers matching both are shown

#### Scenario: No match leaves the document unchanged [lfs_none]

- **WHEN** the filter matches no layer
- **THEN** the panel is empty, the document's layers and visibility are
  unchanged, and toggling the filter off restores every row

### Requirement: Ancestor promotion

When a layer inside a group matches the filter, the panel SHALL show the group
ancestors of that layer so the hierarchy stays navigable, even when the
ancestors do not themselves match. Non-matching siblings SHALL remain hidden. A
group shown only by ancestor promotion SHALL be treated as a container, not as a
match: it SHALL be auto-expanded so its matching descendant is reachable.

#### Scenario: A matching child keeps its group visible [lfs_ancestor]

- **WHEN** a group contains one matching child and one non-matching child and
  the filter is active
- **THEN** the group row and the matching child are shown, and the non-matching
  child is hidden

#### Scenario: The promoted group auto-expands [lfs_ancestor]

- **WHEN** the filter is activated and a collapsed group contains a match
- **THEN** the group is expanded so the matching descendant is visible

### Requirement: View-only, live, and transient

Filtering SHALL be a view-level predicate: it SHALL NOT mutate the document,
SHALL NOT add a history state, and SHALL NOT be serialized or persisted. The
filtered view SHALL re-evaluate when the document changes — a rename, a
visibility, blend-mode, lock, or color-label change, or a structural change
SHALL add or remove rows from the filtered view as the predicate dictates.
Toggling the filter off SHALL restore the full tree, and switching documents
SHALL reset the filter to its default (Kind, off) without losing document state.

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
- **THEN** the filter returns to Kind/off and the new document's full tree is
  shown

