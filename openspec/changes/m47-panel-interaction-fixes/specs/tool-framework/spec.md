## MODIFIED Requirements

### Requirement: Tools panel

The system SHALL present the tools in a dockable Tools panel as a list of flyout
slots in CS6 order, showing the active tool distinctly, and SHALL expose the
panel through the `Window` menu. The panel SHALL default to a single column and
SHALL be reflowable to two columns and back to one. A slot whose group has more
than one member SHALL show a filled triangle at the slot's lower-right corner,
whether or not those members are implemented, while keeping the slot icon
centred, and SHALL reveal the group's members when the pointer press is held past
the hold delay or the slot is right-clicked. An unimplemented tool SHALL be shown
disabled with the tooltip `<label> — not implemented yet`. The panel SHALL keep
the foreground/background colour control and the screen-mode control pinned below
the slots. The foreground/background control SHALL provide, besides the two
swatches and the default-colour (X) reset, a swap control (a double-headed arrow)
that exchanges the foreground and background colours, the `X` key SHALL swap them
when no tool shortcut claims it, and the `D` key SHALL reset them to the default
colours (black foreground, white background) when no tool shortcut claims it. The
panel SHALL be dockable only on the left and right sides of the workspace — not
on the top or bottom — and SHALL also be placeable beside any widget panel or
column, wherever the columns are docked, without breaking the fixed content size
or the no-tabification contract. A left-button drag on the panel's custom title
bar SHALL be tracked for the whole gesture, whether the panel starts docked,
floating, or hosted as a splitter pane, and on release SHALL place the panel as a
splitter pane on either side of the widget column under the pointer or between
two widget columns; when no column is under the pointer the panel SHALL keep its
current state. While the panel is hosted as a central-splitter pane it SHALL keep
its fixed content width and SHALL NOT be pinned to its content height, and it
SHALL remain re-draggable. The implemented tool set and the active-tool contract
SHALL be unchanged.

#### Scenario: Tools panel reflects the active tool [m23_toolbox]

- **WHEN** a tool becomes active by shortcut or by clicking its button
- **THEN** the Tools panel marks that tool's slot as active

#### Scenario: Tools default to a single column of slots [m40_columns]

- **WHEN** the Tools panel is shown with no saved column choice
- **THEN** the tool buttons are arranged in a single column of flyout slots

#### Scenario: The panel docks only left or right [m45_tools_sides]

- **WHEN** the Tools panel is moved toward the top or bottom of the workspace
- **THEN** it is refused there, and it can be docked only on the left or right

#### Scenario: The title-bar drag places the panel among columns [m47_tools_among_columns]

- **WHEN** the Tools panel's title bar is dragged from docked or floating to the
  left or right side of a widget column, or between two widget columns
- **THEN** a single blue indicator marks that boundary and on release the panel is
  hosted as a splitter pane at that boundary with its fixed content width

#### Scenario: The panel is transparent to widget drags [m47_widget_over_tools]

- **WHEN** a widget group is dragged toward the docked Tools panel
- **THEN** the widget overlay can cross the panel and a drop near it resolves to
  the neighbouring widget column
