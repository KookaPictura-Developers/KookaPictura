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
double-headed arrow) that exchanges the foreground and background colours, the
`X` key SHALL swap them when no tool shortcut claims it, and the `D` key SHALL
reset them to the default colours (black foreground, white background) when no
tool shortcut claims it. The panel SHALL be dockable only on the left and right
sides of the workspace — not on the top or bottom — and SHALL also be placeable
beside any widget panel or column, wherever the columns are docked, without
breaking the fixed content size or the no-tabification contract. While the panel
is floating, a left-button drag anywhere on its custom title bar SHALL track the
pointer for the whole drag — including after the dock has grabbed the mouse —
and SHALL resolve a drop on any side of any widget column under the pointer, not
only at the workspace edges. The implemented tool set and the active-tool
contract SHALL be unchanged.

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

#### Scenario: The D key resets the colours [m43_dreset]

- **WHEN** `D` is pressed while no tool shortcut claims it
- **THEN** the foreground colour is black and the background colour is white

#### Scenario: Screen-mode control [m23_toolbox]

- **WHEN** the screen-mode control is clicked
- **THEN** the frame advances to the next screen mode

#### Scenario: The panel docks only left or right [m45_tools_sides]

- **WHEN** the Tools panel is moved toward the top or bottom of the workspace
- **THEN** it is refused there, and it can be docked only on the left or right

#### Scenario: The panel docks beside a widget column [m45_tools_beside_column]

- **WHEN** the Tools panel is moved beside a widget panel or column, wherever
  the columns are docked
- **THEN** it can be placed there without being tabified and without losing its
  fixed content size

#### Scenario: The floating title-bar drag resolves the column under the pointer [m46_tools_gesture]

- **WHEN** the floating Tools panel's title bar is dragged over the interior of a
  widget column
- **THEN** the drag is tracked for the whole gesture and the single blue
  indicator marks that column's side, and releasing there hosts the panel at that
  boundary
