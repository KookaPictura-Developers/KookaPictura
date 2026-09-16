## MODIFIED Requirements

### Requirement: Tools panel

The system SHALL present the tools in a dockable Tools panel as a single-column
list of flyout slots in CS6 order, showing the active tool distinctly, and SHALL
expose the panel through the `Window` menu. A slot with hidden tools SHALL show a
corner triangle and reveal its members on hold; an unimplemented tool SHALL be
shown disabled with the tooltip `<label> — not implemented yet`. The panel SHALL
include a foreground/background colour control and a screen-mode control,
matching the CS6 toolbox. The implemented tool set and the active-tool contract
SHALL be unchanged.

#### Scenario: Tools panel reflects the active tool

- **WHEN** a tool becomes active by shortcut or by clicking its button
- **THEN** the Tools panel marks that tool's slot as active

#### Scenario: Tools are laid out as a single column of slots

- **WHEN** the Tools panel is shown
- **THEN** the tool buttons are arranged in a single column of flyout slots rather than a two-column grid

#### Scenario: A group with hidden tools

- **WHEN** the user holds the mouse on a slot whose group has more than one member
- **THEN** the slot shows a corner triangle and its flyout lists every member of the group

#### Scenario: An unimplemented tool is disabled

- **WHEN** the flyout or slot for a tool with no engine is shown
- **THEN** that tool is disabled and its tooltip is `<label> — not implemented yet`

#### Scenario: Foreground and background control

- **WHEN** the foreground/background control shows the current colours and the user clicks a swatch
- **THEN** that swatch becomes the active colour target and colour edits apply to it

#### Scenario: Screen-mode control

- **WHEN** the screen-mode control is clicked
- **THEN** the frame advances to the next screen mode
