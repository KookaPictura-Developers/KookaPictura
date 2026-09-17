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
disabled with the tooltip `<label> — not implemented yet`. The panel SHALL render
the slot and screen-mode icons at a larger pixmap size than the M40 20×20 pass.
The panel SHALL keep the foreground/background colour control and the screen-mode
control pinned below the slots. The foreground/background control SHALL provide,
besides the two swatches and the default-colour (X) reset, a swap control (a
double-headed arrow) that exchanges the foreground and background colours, and
the `X` key SHALL swap them when no tool shortcut claims it. The implemented tool
set and the active-tool contract SHALL be unchanged.

#### Scenario: Tools panel reflects the active tool [m23_toolbox]

- **WHEN** a tool becomes active by shortcut or by clicking its button
- **THEN** the Tools panel marks that tool's slot as active

#### Scenario: Tools default to a single column of slots [m40_columns]

- **WHEN** the Tools panel is shown with no saved column choice
- **THEN** the tool buttons are arranged in a single column of flyout slots

#### Scenario: The tool icons are larger [m42_tools]

- **WHEN** the Tools panel is shown
- **THEN** its slot icons and screen-mode icon render at a larger pixmap size
  than the M40 20×20 pass

#### Scenario: A group with hidden tools [m40_flyout]

- **WHEN** the user holds the mouse on a slot whose group has more than one member
- **THEN** the slot shows a lower-right triangle and its flyout lists every member of the group

#### Scenario: An unimplemented tool is disabled [m38_tools]

- **WHEN** the flyout or slot for a tool with no engine is shown
- **THEN** that tool is disabled and its tooltip is `<label> — not implemented yet`

#### Scenario: Foreground and background control [m23_toolbox]

- **WHEN** the foreground/background control shows the current colours and the user clicks a swatch
- **THEN** that swatch becomes the active colour target and colour edits apply to it

#### Scenario: The swap control exchanges the colours [m42_fgbg]

- **WHEN** the foreground/background control's swap control is activated
- **THEN** the foreground and background colours are exchanged

#### Scenario: The X key swaps the colours [m42_fgbg]

- **WHEN** `X` is pressed while no tool shortcut claims it
- **THEN** the foreground and background colours are exchanged

#### Scenario: Screen-mode control [m23_toolbox]

- **WHEN** the screen-mode control is clicked
- **THEN** the frame advances to the next screen mode

### Requirement: Tools panel column layout

The system SHALL default the Tools panel to a single column of slots and SHALL
provide a custom title bar carrying a double-arrow control and no separate
`Tools` title label. The control SHALL reflow the slots to two columns and back
to one, showing the icon of the layout it switches to, using the bundled
`panel.columnsTwo` and `panel.columnsOne` icons respectively. In two columns the
slots SHALL be laid out left-to-right and top-to-bottom. The one- and two-column
widths SHALL fit the slot buttons plus the foreground/background control rather
than a fixed constant, so the foreground/background control fits within the
current column width and never widens it. Each width SHALL be the tight content
width plus only the body layout's margins, so the dock carries no extra
horizontal space in either mode. The chosen column count SHALL persist in the
session store at schema version 4 and SHALL load as one column when the store is
missing, older, or the value is out of range. The foreground/background control
and the screen-mode control SHALL remain below the slots in both layouts. A
missing column icon SHALL fall back to a text arrow rather than fail.

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

#### Scenario: The width has no extra horizontal space [m42_tools]

- **WHEN** the dock's actual width is compared with the content width in either
  one- or two-column mode
- **THEN** the difference is only the body layout's margins

#### Scenario: Column count persists [m40_session]

- **WHEN** the user selects two columns and the session is saved and reloaded
- **THEN** the panel opens in two columns
