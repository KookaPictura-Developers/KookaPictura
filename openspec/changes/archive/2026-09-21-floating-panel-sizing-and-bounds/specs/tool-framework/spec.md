## MODIFIED Requirements

### Requirement: Tools panel

The system SHALL present the tools in a tabless, atomic Tools column as a list
of flyout slots in CS6 order, showing the active tool distinctly, and SHALL
expose the panel through the `Window` menu. The Tools column SHALL be hosted like
a `PanelColumn`, with a column header as its top area and the tool grid as a
single plain content child, and SHALL have no tab bar and no `PanelGroup`. The
column SHALL be placeable as a sibling of any widget column on either side of
the document tab area. The Tools column and widget panels SHALL NOT be combined
in either direction: the Tools column SHALL NOT tabify with a widget panel or
group, a widget panel SHALL NOT be inserted into the Tools column, and the Tools
column SHALL NOT be dropped into a widget group, panel, column, or float. The
panel SHALL default to a single column of slots and SHALL be reflowable to two
columns and back to one. A slot whose group has more than one member SHALL show
a filled triangle at the slot's lower-right corner, whether or not those members
are implemented, while keeping the slot icon centred, and SHALL reveal the
group's members when the pointer press is held past the hold delay or the slot is
right-clicked. An unimplemented tool SHALL be shown disabled with the tooltip
`<label> — not implemented yet`. The panel SHALL keep the foreground/background
colour control and the screen-mode control pinned below the slots. The
foreground/background control SHALL provide, besides the two swatches and the
default-colour (X) reset, a swap control (a double-headed arrow) that exchanges
the foreground and background colours, the `X` key SHALL swap them when no tool
shortcut claims it, and the `D` key SHALL reset them to the default colours
(black foreground, white background) when no tool shortcut claims it. A
left-button drag on the Tools column's header SHALL be tracked for the whole
gesture, whether the column starts column-hosted, floating, or a sibling column,
and on release SHALL place the Tools column as a sibling of the widget column
under the pointer or between two widget columns. While the pointer is in the
workspace outer left or right band the column SHALL show the same blue edge
indicator the widget columns use, anchored on the outermost visible column on
that side, and the release SHALL commit the Tools column on that side as a
column. A release with no widget column under the pointer SHALL float the Tools
column at the cursor as a frameless `Qt::Tool` top-level window parented to
(transient for) the main window — no title bar, no window decorations, and no
taskbar entry — rather than an in-window overlay, so a floating column stays
floating and a column can be floated by dragging. The floating Tools column SHALL
be movable outside the main window, its movement clamped to the available
geometry of the screen under the target point so it cannot be lost off-screen.
While floating it SHALL keep its fixed content width and SHALL remain
re-draggable, and it SHALL NOT be resizable: it SHALL offer no resize grip and
SHALL take the minimum its tool-grid content needs. The Tools column SHALL NOT
take an iconic rail mode. The implemented tool set and the active-tool contract
SHALL be unchanged.

#### Scenario: Tools panel reflects the active tool [m23_toolbox]

- **WHEN** a tool becomes active by shortcut or by clicking its button
- **THEN** the Tools panel marks that tool's slot as active

#### Scenario: Tools default to a single column of slots [m40_columns]

- **WHEN** the Tools panel is shown with no saved column choice
- **THEN** the tool buttons are arranged in a single column of flyout slots

#### Scenario: The Tools column is tabless and atomic [tpc_atomic]

- **WHEN** the Tools column is shown
- **THEN** it renders a column header and the tool grid with no tab bar and no
  `PanelGroup`, and it is a sibling column of the widget columns

#### Scenario: The header drag places the column among columns [tpc_sibling]

- **WHEN** the Tools column's header is dragged to the left or right side of a
  widget column, or between two widget columns
- **THEN** a single blue indicator marks that boundary and on release the Tools
  column is placed as a sibling column at that boundary with its fixed content
  width

#### Scenario: The outer band commits the tools column [tpc_edge_commit]

- **WHEN** the Tools column's header is dragged into the workspace outer left or
  right band
- **THEN** the shared blue edge indicator is shown on the outermost visible
  column on that side and the release commits the Tools column on that side as a
  column

#### Scenario: The floating tools column is a frameless tool window [tpc_inwindow_float]

- **WHEN** the Tools column's header is dragged and released with no widget
  column under the pointer
- **THEN** the Tools column floats at the cursor as a frameless `Qt::Tool`
  top-level window parented to the main window with no title bar, decorations,
  or taskbar entry, movable outside the main window and clamped to the screen

#### Scenario: A widget drag cannot combine with the tools column [tpc_atomic_target]

- **WHEN** a widget panel or group is dragged toward the Tools column, or the
  Tools column is dragged toward a widget panel or group
- **THEN** no tabification, insertion, or combined drop occurs and no insertion
  indicator is shown for the forbidden combination

### Requirement: Tools panel column layout

The system SHALL present the Tools column with a column header that carries one
width-toggle control and no separate `Tools` title label. The control SHALL share
the widget column's toggle style and SHALL reflow the slots between one and two
columns, showing the icon of the layout it switches to, using the bundled
`panel.columnsTwo` and `panel.columnsOne` icons. The Tools column SHALL NOT have
an iconic rail mode, so the control SHALL change the tool-grid width and SHALL
NOT collapse the column to an icon rail. In two columns the slots SHALL be laid
out left-to-right and top-to-bottom. The one- and two-column widths SHALL fit the
slot buttons plus the foreground/background control rather than a fixed constant,
so the foreground/background control fits within the current column width and
never widens it. Each width SHALL be the tight content width plus only the body
layout's margins, so the column carries no extra horizontal space in either
mode. Each width SHALL be fixed: the column's minimum and maximum width SHALL
both equal its content width for the active column count, recomputed when the
column count changes and while the column is floating, and dragging the column
separator SHALL NOT resize it. Both the content width and the content height
SHALL be derived from one content formula and SHALL be recomputed on every
column-count change and on every float/column-host change, releasing the stale
fixed axis before re-fixing it, so neither the one- nor the two-column layout is
cut off or left over-tall. While the column is hosted in the workspace the tool
grid SHALL fill its column height; while the column is floating the overlay SHALL
be a frameless `Qt::Tool` top-level window parented to the main window and its
height SHALL be fixed to the minimum its content needs, with no resize grip and
no drag-resize. The chosen column count SHALL persist in the session store and
SHALL load as one column when the store is missing, older, or the value is out of
range. The foreground/background control and the screen-mode control SHALL remain
below the slots in both layouts. A missing column icon SHALL fall back to a text
arrow rather than fail.

#### Scenario: The double-arrow toggles to two columns and back [m41_tools]

- **WHEN** the user clicks the header double arrow while in one column
- **THEN** the 23 slots reflow into two columns and the button shows the
  one-column icon, and a second click restores one column

#### Scenario: The header has no Tools label [m41_tools]

- **WHEN** the Tools column's header is inspected
- **THEN** it carries the double-arrow control and no `Tools` text label

#### Scenario: Each mode sizes exactly to its content [m45_tools_sizing]

- **WHEN** the Tools column is shown in one column and then switched to two
  columns, floating and column-hosted
- **THEN** in each mode the width equals that mode's content width and the
  height equals that mode's content height while floating, with no stale lock, no
  cut-off, and no leftover extra height

#### Scenario: The width fits the content [m41_tools]

- **WHEN** the one- and two-column widths are queried
- **THEN** each is derived from the slot and foreground/background control sizes
  and the foreground/background control fits within the current column width

#### Scenario: The width has no extra horizontal space [m42_tools]

- **WHEN** the column's actual width is compared with the content width in either
  one- or two-column mode
- **THEN** the difference is only the body layout's margins

#### Scenario: The width is fixed [m43_tools]

- **WHEN** the Tools column's minimum and maximum widths are queried in one- or
  two-column mode
- **THEN** both equal the content width for that mode

#### Scenario: The floating height is fixed [m44_toolsfloat]

- **WHEN** the Tools column is floating and a resize is attempted
- **THEN** its height stays the minimum its content needs and does not change

#### Scenario: The tools column has no iconic mode [tpc_no_iconic]

- **WHEN** the Tools column's width control is activated
- **THEN** it switches the tool grid between one and two columns and never
  collapses the column to an icon rail

#### Scenario: Column count persists [m40_session]

- **WHEN** the user selects two columns and the session is saved and reloaded
- **THEN** the panel opens in two columns
