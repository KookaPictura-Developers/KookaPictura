## MODIFIED Requirements

### Requirement: Options bar

The system SHALL present an options bar whose controls change with the active
tool, and SHALL allow the options bar to be shown or hidden. Each selection tool
SHALL expose its combine mode; Quick Selection SHALL expose a tolerance. Brush
and Pencil SHALL expose size, hardness, opacity, flow, and paint mode; Pencil
SHALL additionally expose Auto Erase. The options bar's numeric controls SHALL
use the shared numeric field control, so each offers a scrubbing label and a
slider popup, and a change to one control SHALL stay in sync with any other
control bound to the same tool value. Every brush-settings control SHALL be
idle-less (no idle outline or background), and the options bar's body SHALL size
to its controls so the bar reserves no vertical space for an idle button and is
not over-tall.

#### Scenario: Options follow the active tool

- **WHEN** the active tool changes
- **THEN** the options bar shows that tool's controls

#### Scenario: Options bar toggle

- **WHEN** the user toggles the options bar
- **THEN** it is shown or hidden without changing the active tool

#### Scenario: A numeric option can be scrubbed and popup-edited [ltf_options_numeric]

- **WHEN** the user interacts with a numeric options-bar control such as the
  brush size or a selection tolerance
- **THEN** it offers a scrubbing label and a slider popup and applies the change
  to the next operation

#### Scenario: The bar is not over-tall [ltf_options_compact]

- **WHEN** the options bar is shown for a tool with a brush-settings control
- **THEN** the brush-settings control is idle-less and the bar's body height
  matches its controls with no reserved idle-button space

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
default-colour (X) reset, a swap control drawn as a top-left and a down-left
arrow that exchanges the foreground and background colours, the `X` key SHALL
swap them when no tool shortcut claims it, and the `D` key SHALL reset them to
the default colours (black foreground, white background) when no tool shortcut
claims it. The panel SHALL show a Paint Mask Mode toggle button in the slot
immediately to the left of the screen-mode control, and the screen-mode control
SHALL open the screen-mode menu. A left-button drag on the Tools column's header
SHALL be tracked for the whole gesture, whether the column starts column-hosted,
floating, or a sibling column, and on release SHALL place the Tools column as a
sibling of the widget column under the pointer or between two widget columns.
While the pointer is in the workspace outer left or right band the column SHALL
show the same blue edge indicator the widget columns use, anchored on the
outermost visible column on that side, and the release SHALL commit the Tools
column on that side as a column. A release with no widget column under the
pointer SHALL float the Tools column at the cursor as a frameless `Qt::Tool`
top-level window parented to (transient for) the main window — no title bar, no
window decorations, and no taskbar entry — rather than an in-window overlay, so
a floating column stays floating and a column can be floated by dragging. The
floating Tools column SHALL be movable outside the main window, its movement
clamped to the available geometry of the screen under the target point so it
cannot be lost off-screen. While floating it SHALL keep its fixed content width
and SHALL remain re-draggable, and it SHALL NOT be resizable: it SHALL offer no
resize grip and SHALL take the minimum its tool-grid content needs. The Tools
column SHALL NOT take an iconic rail mode. The implemented tool set and the
active-tool contract SHALL be unchanged.

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

#### Scenario: The footer controls [ltf_footer_controls]

- **WHEN** the Tools panel's footer is shown
- **THEN** the foreground/background swap control draws a top-left and a
  down-left arrow, the Paint Mask Mode toggle sits immediately left of the
  screen-mode control, and the screen-mode control opens the screen-mode menu

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

### Requirement: Tool flyout indicator and opening

The system SHALL mark every slot whose group has more than one member with a
filled triangle at the lower-right corner, including a group whose members are
all unimplemented, and SHALL NOT mark a single-member slot. Holding the pointer
press on a slot past the hold delay SHALL open the slot's flyout menu positioned
to the right of the button, or to the left when the right side does not fit,
never covering the button that opened it; a right-click SHALL open it
immediately; releasing the press before the delay SHALL activate the slot's
current tool and SHALL NOT open the menu. While a slot's flyout is open, a press
on another slot SHALL close the open flyout and activate that slot (or open its
menu) within the same press, so switching tools takes only one click. Each
flyout item SHALL carry a left margin for its icon and a tighter gap between the
icon and the label. A single-member slot SHALL NOT open a flyout. The system
SHALL expose a deterministic hook that opens a slot's flyout for the self-test.

#### Scenario: A multi-member slot shows the triangle [m40_flyout]

- **WHEN** a slot's group has two or more members
- **THEN** the slot paints a filled triangle at its lower-right corner

#### Scenario: A single-member slot shows no triangle [m40_flyout]

- **WHEN** a slot's group has exactly one member
- **THEN** the slot paints no triangle and opens no flyout

#### Scenario: Holding opens the flyout below the button [m40_flyout]

- **WHEN** the pointer press is held on a multi-member slot past the hold delay
- **THEN** the slot's flyout opens to the right of the button, or to the left
  when the right side does not fit, and does not cover the button

#### Scenario: A right-click opens the flyout immediately [m40_flyout]

- **WHEN** the user right-clicks a multi-member slot
- **THEN** the flyout opens without waiting for the hold delay

#### Scenario: A quick release selects the current tool [m40_flyout]

- **WHEN** the user presses and releases a slot before the hold delay
- **THEN** the slot's current implemented tool becomes active and no flyout opens

#### Scenario: One click switches tools while a flyout is open [ltf_flyout_one_click]

- **WHEN** a slot's flyout is open and the user presses another slot
- **THEN** the open flyout closes and that slot activates (or its own menu opens)
  in the same press, with no second click needed

#### Scenario: Flyout items have icon margin and a tight gap [ltf_flyout_item_gap]

- **WHEN** a flyout menu is shown
- **THEN** each item's icon has a left margin and the gap between the icon and
  the label is tight
