## MODIFIED Requirements

### Requirement: Tools panel

The system SHALL present the tools in a dockable Tools panel as a compact
two-column grid of icon buttons, one cell per tool, showing the active tool
distinctly, and SHALL expose the panel through the `Window` menu. The panel
SHALL include a foreground/background colour control and a screen-mode control,
matching the CS6 toolbox. The tool set and the active-tool contract SHALL be
unchanged.

#### Scenario: Tools panel reflects the active tool

- **WHEN** a tool becomes active by shortcut or by clicking its button
- **THEN** the Tools panel marks that tool as active

#### Scenario: Tools are laid out in a compact grid

- **WHEN** the Tools panel is shown
- **THEN** the tool buttons are arranged in a two-column grid rather than a single column strip

#### Scenario: Foreground and background control

- **WHEN** the foreground/background control shows the current colours and the user clicks a swatch
- **THEN** that swatch becomes the active colour target and colour edits apply to it

#### Scenario: Screen-mode control

- **WHEN** the screen-mode control is clicked
- **THEN** the frame advances to the next screen mode
