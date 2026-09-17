## MODIFIED Requirements

### Requirement: Tools panel column layout

The system SHALL default the Tools panel to a single column of slots and SHALL
provide a custom title bar carrying a double-arrow control and no separate
`Tools` title label. The control SHALL reflow the slots to two columns and back
to one, showing the icon of the layout it switches to, using the bundled
`panel.columnsTwo` and `panel.columnsOne` icons respectively. In two columns the
slots SHALL be laid out left-to-right and top-to-bottom. The one- and two-column
widths SHALL fit the slot buttons plus the foreground/background control rather
than a fixed constant, so the foreground/background control fits within the
current column width and never widens it. The chosen column count SHALL persist
in the session store at schema version 4 and SHALL load as one column when the
store is missing, older, or the value is out of range. The foreground/background
control and the screen-mode control SHALL remain below the slots in both
layouts. A missing column icon SHALL fall back to a text arrow rather than fail.

#### Scenario: The double-arrow toggles to two columns and back [m41_tools]

- **WHEN** the user clicks the title-bar double arrow while in one column
- **THEN** the 23 slots reflow into two columns and the button shows the
  one-column icon, and a second click restores one column

#### Scenario: The title bar has no Tools label [m41_tools]

- **WHEN** the Tools panel's title bar is inspected
- **THEN** it carries the double-arrow control and no `Tools` text label

#### Scenario: The width fits the content [m41_tools]

- **WHEN** the one- and two-column widths are queried
- **THEN** each is derived from the slot and foreground/background control sizes
  and the foreground/background control fits within the current column width

#### Scenario: Column count persists [m40_session]

- **WHEN** the user selects two columns and the session is saved and reloaded
- **THEN** the panel opens in two columns
