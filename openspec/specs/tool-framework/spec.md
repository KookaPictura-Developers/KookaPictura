# tool-framework Specification

## Purpose
TBD - created by archiving change m18-toolbox-tools. Update Purpose after archive.
## Requirements
### Requirement: Tool registry and active tool

The system SHALL define a fixed set of tools identified by a stable `ToolId`
and SHALL track exactly one active tool. The implemented set SHALL remain the
ten tools (Move, Marquee, Lasso, Quick Selection, Crop, Eyedropper, Brush,
Pencil, Hand, Zoom). The system SHALL refuse to activate a tool that has no
engine. Selecting a tool SHALL be possible from a single-letter keyboard
shortcut, and every tool in one flyout group SHALL share that group's slot
letter. The system SHALL honour the `Use Shift Key For Tool Switch` preference:
when it is on (the default), pressing the letter alone SHALL activate the slot's
current member and `Shift`+letter SHALL cycle to the next implemented member of
the group; when it is off, the letter alone SHALL cycle. Cycling SHALL wrap
deterministically and SHALL skip members that are not implemented. A group with
no implemented member SHALL react to no letter.

#### Scenario: Activating a tool by shortcut [m40_shift]

- **WHEN** the user presses a tool slot's letter
- **THEN** the slot's current member becomes active and no other tool remains active

#### Scenario: Cycling the paint tool slot [m40_shift]

- **WHEN** `Shift+B` is pressed while Brush is active and `Use Shift Key For Tool Switch` is on
- **THEN** Pencil becomes active

#### Scenario: The preference controls whether Shift is required [m40_shift]

- **WHEN** the same letter is pressed with the preference off
- **THEN** the letter alone advances to the next implemented member of the group

#### Scenario: A group with no implemented member does nothing [m40_shift]

- **WHEN** the letter of a group whose members are all unimplemented is pressed
- **THEN** the active tool does not change

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

### Requirement: Options bar

The system SHALL present an options bar whose controls change with the active
tool, and SHALL allow the options bar to be shown or hidden. Each selection tool
SHALL expose its combine mode; Quick Selection SHALL expose a tolerance. Brush
and Pencil SHALL expose size, hardness, opacity, flow, and paint mode; Pencil
SHALL additionally expose Auto Erase. The options bar's numeric controls SHALL
use the shared numeric field control, so each offers a scrubbing label and a
slider popup, and a change to one control SHALL stay in sync with any other
control bound to the same tool value.

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

### Requirement: Canvas pointer routing to the active tool
The system SHALL route canvas pointer press, move, and release events to the
active tool with image-space coordinates. When the Hand tool is active the canvas
SHALL pan on left-drag as before; other tools SHALL receive the events.

#### Scenario: Hand tool pans
- **WHEN** the Hand tool is active and the user left-drags the canvas
- **THEN** the canvas pans

#### Scenario: A drawing tool receives the drag
- **WHEN** a selection or crop tool is active and the user presses on the canvas
- **THEN** that tool begins its drag and is given the pressed image coordinates

### Requirement: Tool status and cursor hints

The system SHALL show a context hint for the active tool in the bottom status bar
as a hint bar of bordered keycaps with descriptions, and SHALL set a
tool-appropriate cursor while the tool is active. For Brush and Pencil the cursor
SHALL be blank so the drawn brush-size ring is the pointer affordance. When the
active tool would edit pixels and the target layer is pixel-locked, the cursor
SHALL indicate that the action is not allowed, and a refusal SHALL be reported to
the user rather than the action silently doing nothing. **The canvas SHALL also
show a Block/Forbidden cursor when a pixel-editing tool's active layer is
invisible (an edit would be refused), and the transient Alt eyedropper SHALL
show the eyedropper cursor while Alt is held with a paint tool active. The
cursor SHALL be resolved in one place with the precedence transient eyedropper >
transform session > blank paint > invisible target > pixel-locked target >
tool default, so the branches cannot disagree.**

#### Scenario: Status hint reflects the tool

- **WHEN** a tool becomes active
- **THEN** the status-bar hint bar shows that tool's context keycaps

#### Scenario: Brush uses a blank cursor [ltf_blank_cursor]

- **WHEN** Brush or Pencil is active over the canvas
- **THEN** the canvas cursor is blank and the brush-size ring is drawn

#### Scenario: A locked target marks the cursor [ltf_locked_cursor]

- **WHEN** a pixel-editing tool is active over a pixel-locked layer
- **THEN** the canvas cursor indicates the action is not allowed and an attempted
  edit reports a refusal

#### Scenario: An invisible target marks the cursor [ltf_invisible_cursor]

- **WHEN** a pixel-editing tool is active over a layer whose visibility is off
- **THEN** the canvas shows a Block/Forbidden cursor and an attempted paint is
  refused

#### Scenario: The transient eyedropper cursor shows [ltf_alt_cursor]

- **WHEN** Alt is held with a paint tool active over the canvas
- **THEN** the canvas shows the eyedropper cursor, which takes precedence over
  the blank paint cursor

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
horizontal space in either mode. Each width SHALL be fixed: the dock's minimum
and maximum width SHALL both equal its content width for the active column count,
recomputed when the column count changes and while the dock is floating, and
dragging the dock separator SHALL NOT resize it. Both the content width and the
content height SHALL be derived from one content formula and SHALL be recomputed
on every column-count change and on every dock/float change, releasing the stale
fixed axis before re-fixing it, so neither the one- nor the two-column layout is
cut off or left over-tall. While floating, the dock's height SHALL also be fixed
to the minimum its content needs and SHALL NOT be drag-resizable. The chosen
column count SHALL persist in the session store at schema version 4 and SHALL
load as one column when the store is missing, older, or the value is out of
range. The foreground/background control and the screen-mode control SHALL remain
below the slots in both layouts. A missing column icon SHALL fall back to a text
arrow rather than fail.

#### Scenario: The double-arrow toggles to two columns and back [m41_tools]

- **WHEN** the user clicks the title-bar double arrow while in one column
- **THEN** the 23 slots reflow into two columns and the button shows the
  one-column icon, and a second click restores one column

#### Scenario: The title bar has no Tools label [m41_tools]

- **WHEN** the Tools panel's title bar is inspected
- **THEN** it carries the double-arrow control and no `Tools` text label

#### Scenario: Each mode sizes exactly to its content [m45_tools_sizing]

- **WHEN** the Tools panel is shown in one column and then switched to two
  columns, floating and docked
- **THEN** in each mode the width equals that mode's content width and the
  height equals that mode's content height, with no stale lock, no cut-off, and
  no leftover extra height

#### Scenario: The width fits the content [m41_tools]

- **WHEN** the one- and two-column widths are queried
- **THEN** each is derived from the slot and foreground/background control sizes
  and the foreground/background control fits within the current column width

#### Scenario: The width has no extra horizontal space [m42_tools]

- **WHEN** the dock's actual width is compared with the content width in either
  one- or two-column mode
- **THEN** the difference is only the body layout's margins

#### Scenario: The width is fixed [m43_tools]

- **WHEN** the Tools panel's minimum and maximum widths are queried in one- or
  two-column mode
- **THEN** both equal the content width for that mode

#### Scenario: The floated height is fixed [m44_toolsfloat]

- **WHEN** the Tools panel is floating and a resize is attempted
- **THEN** its height stays the minimum its content needs and does not change

#### Scenario: Column count persists [m40_session]

- **WHEN** the user selects two columns and the session is saved and reloaded
- **THEN** the panel opens in two columns

### Requirement: Tool flyout indicator and opening

The system SHALL mark every slot whose group has more than one member with a
filled triangle at the lower-right corner, including a group whose members are
all unimplemented, and SHALL NOT mark a single-member slot. Holding the pointer
press on a slot past the hold delay SHALL open the slot's flyout menu positioned
below the button; a right-click SHALL open it immediately; releasing the press
before the delay SHALL activate the slot's current tool and SHALL NOT open the
menu. A single-member slot SHALL NOT open a flyout. The system SHALL expose a
deterministic hook that opens a slot's flyout for the self-test.

#### Scenario: A multi-member slot shows the triangle [m40_flyout]

- **WHEN** a slot's group has two or more members
- **THEN** the slot paints a filled triangle at its lower-right corner

#### Scenario: A single-member slot shows no triangle [m40_flyout]

- **WHEN** a slot's group has exactly one member
- **THEN** the slot paints no triangle and opens no flyout

#### Scenario: Holding opens the flyout below the button [m40_flyout]

- **WHEN** the pointer press is held on a multi-member slot past the hold delay
- **THEN** the slot's flyout opens with its top edge at the button's bottom edge

#### Scenario: A right-click opens the flyout immediately [m40_flyout]

- **WHEN** the user right-clicks a multi-member slot
- **THEN** the flyout opens without waiting for the hold delay

#### Scenario: A quick release selects the current tool [m40_flyout]

- **WHEN** the user presses and releases a slot before the hold delay
- **THEN** the slot's current implemented tool becomes active and no flyout opens

### Requirement: Tool flyout shortcut keys

The system SHALL show every flyout item's slot key right-aligned in the item,
including a disabled item, and SHALL keep every unimplemented item disabled with
the tooltip `<label> — not implemented yet`. The system SHALL display the key
through `QAction::setShortcut` with `QAction::setShortcutVisibleInContextMenu`
enabled, scoped to the flyout with `Qt::WidgetWithChildrenShortcut`, and SHALL
NOT register a window-global shortcut for a flyout item, so a key shared by a
group's members — including a disabled member — cannot steal the slot letter from
the toolbox's keyboard handler.

#### Scenario: Every item shows its shared key [m40_keys]

- **WHEN** a group's flyout is opened
- **THEN** every item shows the group's letter right-aligned

#### Scenario: A disabled item stays disabled with its tooltip [m40_keys]

- **WHEN** the flyout contains a tool with no engine
- **THEN** that item is disabled and its tooltip is `<label> — not implemented yet`

#### Scenario: A disabled item's shared key does not steal the letter [m40_keys]

- **WHEN** a group's flyout has been opened and closed and the group's letter is pressed
- **THEN** the toolbox's keyboard handler runs and the disabled item's shortcut does not consume the key

### Requirement: Shift-key tool switching preference

The system SHALL store a `Use Shift Key For Tool Switch` preference that defaults
to on and SHALL gate the `Shift` requirement for cycling a tool group on it. The
preference SHALL persist in the session store at schema version 4, and a store
that is missing the field or older than version 4 SHALL load the default. Adding
the preference SHALL NOT change the load and save round-trip of the other session
fields.

#### Scenario: The preference defaults to on [m40_session]

- **WHEN** the session preference is read with no saved value
- **THEN** `Use Shift Key For Tool Switch` is on

#### Scenario: The preference round-trips [m40_session]

- **WHEN** the preference is changed, the session is saved, and the store is reloaded
- **THEN** the changed value is restored alongside the other session fields

#### Scenario: An older store loads the default [m40_session]

- **WHEN** a schema-3 store with neither new field is loaded
- **THEN** the column count is 1 and the preference is on

### Requirement: Active layer resolution for tool edits

The application SHALL maintain exactly one active layer for the active document
and SHALL resolve it through one shared active-layer resolver used by every
tool-edit entry point (paint, filter, Free Transform, and content move). The
Layers panel SHALL push its selection into the view when the selection changes,
and a non-PSD raster import SHALL make its imported layer active. A tool edit
SHALL be refused, with the document unchanged and a user-visible refusal, when no
layer is active or when more than one layer is selected. No tool-edit entry point
SHALL fall back to the topmost raster layer.

#### Scenario: Exactly one active layer receives the edit [ltf_active_layer]

- **WHEN** a tool edit runs with exactly one layer active
- **THEN** that layer is the edit target and the other layers are unchanged

#### Scenario: The panel pushes its selection [ltf_active_layer_push]

- **WHEN** the user changes the selected layer in the Layers panel
- **THEN** the view's active layer follows the panel selection and the next tool
  edit targets it

#### Scenario: Zero or multiple selection refuses [ltf_active_layer_refuse]

- **WHEN** a tool edit runs with no layer active or with more than one layer
  selected
- **THEN** the edit is refused with a user-visible refusal, the document is
  unchanged, and no history state is added

### Requirement: Active-layer scope of the resolver

The resolver SHALL be the single source of the edit target and SHALL be consulted
by paint, filter, Free Transform, and content move. The resolver SHALL NOT be
used to block structural layer operations, and a position lock SHALL remain the
only reason a move is refused. When the active layer changes, the tool status and
cursor SHALL reflect the new target without a separate per-tool lookup.

#### Scenario: Every tool edit uses the same resolver [ltf_active_layer_shared]

- **WHEN** paint, a filter, Free Transform, and a content move each run with the
  same single active layer
- **THEN** all four target that layer through the same resolver

### Requirement: Transient alt-tool cursor is resolved centrally

The cursor precedence SHALL live in one `refreshCursor` path so the transient
eyedropper, the invisible-layer refusal, and the existing blank/locked cursors
are not re-evaluated in separate handlers. Changing the active layer, its
visibility, or the held modifiers SHALL refresh the cursor through that one path
without a per-tool lookup.

#### Scenario: One path resolves all cursor states [ltf_cursor_precedence]

- **WHEN** the active layer becomes invisible, then visible again, with a paint
  tool active
- **THEN** the canvas cursor changes through the single precedence path each time
  and no other handler re-sets it

### Requirement: Canvas cursor refreshes on an active-layer change

When the active layer's visibility or identity changes, the canvas cursor SHALL
be refreshed without requiring a pointer move, so an invisible active layer shows
the Block/Forbidden cursor (and a visible one restores the tool cursor)
immediately after an eye toggle or a layer selection. The cursor resolution
itself SHALL keep the existing precedence: a transient Alt eyedropper wins, then
the invisible/locked refusal, then the tool cursor.

#### Scenario: Hiding the active layer shows the Block cursor [tf_visible_cursor_refresh]

- **WHEN** the active layer is made invisible while a pixel-editing tool is
  active and the pointer is over the canvas
- **THEN** the canvas cursor becomes the Block/Forbidden cursor without moving
  the pointer

#### Scenario: Showing the active layer restores the tool cursor [tf_visible_cursor_restore]

- **WHEN** the active layer is made visible again
- **THEN** the canvas cursor returns to the active tool's cursor without moving
  the pointer

### Requirement: Tool refusal follows the active layer

The tool refusal SHALL resolve the **active** single layer's lock state and
visibility — the same layer the paint and filter edits target — not the topmost
pixel layer. The brush/pencil blank-cursor affordance, the Block/Forbidden
refusal cursor, and the edit-refusal status message SHALL all follow this. When the active layer is pixel- or position-locked, or is
invisible, the refusal cursor and message SHALL appear even if the topmost pixel
layer is editable; when the topmost layer is locked but a lower layer is active
and editable, the cursor SHALL stay blank (brush) and the edit SHALL proceed. The
transient Alt-eyedropper precedence SHALL be unchanged.

#### Scenario: A locked active layer refuses with the Block cursor [tf_refusal_active_locked]

- **WHEN** the active layer is pixel-locked and is not the topmost pixel layer
- **THEN** the brush shows the Block/Forbidden cursor and refuses the edit

#### Scenario: A locked topmost layer does not block a lower active layer [tf_refusal_topmost_ignored]

- **WHEN** the topmost pixel layer is locked but the active layer is a lower,
  editable layer
- **THEN** the brush cursor stays blank and the edit targets the active layer

#### Scenario: A panel multi-selection prevents edits [tf_refusal_multi]

- **WHEN** the Layers panel has more than one row selected, or no row
- **THEN** tool edits are refused with no history state

### Requirement: Tools panel placement outcomes

A left-button drag on the Tools panel's custom title bar SHALL resolve to one of
three placements on release, through the panel's own gesture rather than Qt's
dock drag, so the outcome is deterministic. A release over or beside a widget
column SHALL host the panel as a central-splitter pane at that column's boundary.
A release in the workspace's outer left or right band SHALL dock the panel to the
corresponding left or right dock area. A release outside the main window SHALL
float the panel at the cursor. A title-bar double-click SHALL toggle the panel
between a floating overlay and its current left/right dock. The panel SHALL
remain re-draggable from each state, and the implemented tool set and active-tool
contract SHALL be unchanged.

#### Scenario: The title-bar drag keeps all three placements [ltf_tools_placements]

- **WHEN** the Tools panel's title bar is dragged and released over a widget
  column, in the workspace outer band, or outside the main window
- **THEN** it becomes a splitter pane beside that column, docks to that side, or
  floats at the cursor respectively

#### Scenario: Double-click floats and re-docks the panel [ltf_tools_dblclick]

- **WHEN** the Tools panel's title bar is double-clicked while docked or hosted
  as a splitter pane, then double-clicked again while floating
- **THEN** it floats on the first double-click and returns to a normal left/right
  dock on the second

