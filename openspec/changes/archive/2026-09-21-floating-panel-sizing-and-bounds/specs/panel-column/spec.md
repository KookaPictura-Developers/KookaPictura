## MODIFIED Requirements

### Requirement: Floating panel overlay

A group torn off a column SHALL float in a frameless overlay that follows the
cursor and re-docks on release, and the overlay SHALL be able to cross and be
placed around the docked or pane-hosted Tools panel. The overlay SHALL be a
frameless `Qt::Tool` top-level window parented to (transient for) the main
window — no title bar, no window decorations, and no taskbar entry — rather than
an in-window child widget, so it SHALL NOT be clipped to the main window. Its
movement SHALL be clamped to the available geometry of the screen under the
target point (falling back to the frame's screen, then the primary screen) and
SHALL NOT be clamped to the main window rect, so the overlay can be placed
outside the main window yet cannot be lost off-screen. The overlay SHALL offer a
resize grip in its bottom-right corner that resizes the overlay itself by the
drag delta, clamped to the overlay's minimum size, and SHALL NOT resize the main
window. The overlay SHALL show a close control at the rightmost side of its
header; closing SHALL hide the group's panels while keeping the group restorable
from `Window > Panels`, then remove the overlay. The overlay SHALL support a
non-resizable mode in which the grip is absent and the overlay takes the minimum
its content needs; the floating Tools column SHALL use that mode.

#### Scenario: A torn-off group floats and re-docks [m41_tearoff]

- **WHEN** a group is dragged out of a column and dropped back on a column
- **THEN** it floats in a frameless overlay and re-docks

#### Scenario: A floating group can be closed [m47_float_close]

- **WHEN** a floating group's header close control is activated
- **THEN** the overlay is removed, its panels are hidden, and the group is still
  present in a column so `Window > Panels` can restore it

#### Scenario: The overlay can cross the Tools panel [m47_float_over_tools]

- **WHEN** a group is dragged toward the docked Tools panel
- **THEN** the overlay follows the cursor over the Tools panel instead of
  stopping at the central area edge

#### Scenario: The resize grip resizes the overlay [fp_grip_resize]

- **WHEN** the floating overlay's corner grip is dragged
- **THEN** the overlay's size changes by the drag delta and is not shrunk below
  its minimum, while the main window's size does not change

#### Scenario: The floating Tools column is not resizable [fp_tools_float_fixed]

- **WHEN** the Tools column is floating
- **THEN** no resize grip is offered and the overlay is sized to the minimum the
  tool grid's content needs, taking its content width and minimum height

#### Scenario: The overlay is clamped to the screen [fp_float_screen_bounds]

- **WHEN** a floating overlay is moved toward or past the edge of its screen
- **THEN** it is clamped inside that screen's available geometry and may sit
  outside the main window's rect

### Requirement: Floating group top bar and icon mode

A floating panel group SHALL carry its own top bar with a normal/icon width
toggle and a close control, the toggle placed immediately to the left of the
close control on the bar's right side, and the bar SHALL drag the overlay. The
toggle SHALL collapse the group to its icon row and expand it again, and the
close control SHALL remain reachable while collapsed. While collapsed the overlay
SHALL snap to the height its icon row needs, at least the icon-row minimum, and
SHALL also shrink its width to the icon row's natural width (at least the
overlay minimum) so no normal-width body remains. The icon row SHALL render as
the docked icon strip's group box with the same container styling, grip divider,
and icon size. The overlay SHALL offer a resize grip that resizes the overlay
itself and keep a minimum size like a docked column. When the floating group
holds a single visible panel, dragging its tab SHALL move the whole overlay
rather than tear off a second overlay and leave a ghost. A dragged overlay SHALL
be dimmed for the whole drag, from the start of the gesture until it ends, not
only while it is over a valid drop target.

#### Scenario: The float top bar toggles and closes [pc_float_header]

- **WHEN** a group floats
- **THEN** its top bar shows a collapse toggle immediately left of a close
  control on the right, toggling collapses and expands the group, and dragging
  the bar moves the overlay

#### Scenario: The floating icon overlay snaps to the icon row [pc_float_icon_min]

- **WHEN** a floating group is collapsed to icons
- **THEN** the overlay snaps to the icon-row height (at least the icon-row
  minimum), shrinks its width to the icon row's natural width so no
  normal-width body remains, the icon row renders as a grouped panel-icon box,
  and the close control remains reachable

#### Scenario: The floating overlay is resizable [pc_float_resize]

- **WHEN** a floating group is shown and its corner grip is dragged
- **THEN** it carries a resize grip that resizes the overlay itself and it keeps
  a minimum size comparable to a docked column

#### Scenario: A single-panel float moves as a whole [pc_float_single_drag]

- **WHEN** the tab of a floating group that holds one visible panel is dragged
- **THEN** the whole overlay moves and no second overlay or empty source overlay
  is left behind

#### Scenario: A dragged overlay dims for the whole drag [pc_float_dim]

- **WHEN** an overlay is dragged
- **THEN** it is dimmed from the start of the drag through release, including
  while the target is invalid, and it returns to full opacity when the drag ends

### Requirement: Whole-column in-window float

The system SHALL allow a whole widget `PanelColumn` to be dragged by its header
and torn off the workspace into a frameless floating overlay, through the same
drag machinery and the same single insertion indicator the panel and group drags
use. The overlay SHALL be a frameless `Qt::Tool` top-level window parented to
the main window, rather than an in-window child widget, and SHALL NOT be a
decorated operating-system window. It SHALL keep a minimum width and SHALL offer
a resize grip. A widget-column overlay SHALL open at about two thirds of the
column's docked height, clamped to the overlay minimum. When the hosted column
is iconic the overlay SHALL snap to the column's content height and its narrow
content width. A release over a valid column or workspace edge SHALL re-place the
column; a release with no valid target SHALL leave the overlay floating.

#### Scenario: Dragging a whole column floats it in-window [pc_col_float]

- **WHEN** a widget column's header is dragged out of the workspace
- **THEN** the whole column floats in a frameless overlay that follows the
  cursor and is committed on release

#### Scenario: The floating column keeps a minimum width and a grip [pc_col_float_grip]

- **WHEN** a floating whole-column overlay is shown
- **THEN** it carries a resize grip and cannot be resized below a minimum width

#### Scenario: The floating column opens below its docked height [pc_col_float_height]

- **WHEN** a widget column is torn off and floats
- **THEN** its default height is strictly less than its docked height

#### Scenario: An iconic column float snaps narrow [pc_col_float_iconic_snap]

- **WHEN** a floating whole-column overlay hosts a column that is collapsed to
  its iconic strip
- **THEN** the overlay snaps to the column's content height and narrow content
  width instead of keeping the normal-width body

#### Scenario: The floating column re-places [pc_col_redock]

- **WHEN** a floating whole-column overlay is dropped on a column or workspace
  edge
- **THEN** the whole column re-places at that position and the overlay is removed

### Requirement: Panel drag and drop rules

The system SHALL support dragging a panel tab and dragging a group, and SHALL
distinguish them for the whole drag. Dragging a tab SHALL drag only that panel;
dragging the empty header area to the right of the tabs SHALL drag the whole
group. Dragging a tab within its own group SHALL reorder it; dragging a tab onto
another group's tab bar SHALL move the panel into that group at the target index;
dragging onto the column background or between groups SHALL insert a new group at
that boundary; and dragging past the workspace edge SHALL tear the dragged panel
or group off as a frameless `Qt::Tool` top-level window parented to (transient
for) the main window — no title bar, no window decorations, and no taskbar entry
— rather than an in-window overlay, movable outside the main window with its
movement clamped to the available screen geometry. A single-panel drag SHALL
float a one-panel float, and a group drag SHALL float the group. The same
tab-versus-header distinction SHALL hold for a floating group. A floating overlay
SHALL be re-dockable into a column, into another group, or as a new column by
dropping it on the target. When a tab is dragged outward to float, the drag SHALL
continue until the mouse button is released: the float SHALL keep following the
cursor, exactly like a group or column drag, and SHALL only be committed or
re-docked on release. A whole widget column SHALL be draggable to any side of the
Tools toolbar, of another widget panel or column, and of the workspace. Every
drop that moves the last panel or group out of a widget column SHALL leave no
empty column behind, and this SHALL apply to the primary column as well as to
drop-created columns: when the last visible panel is removed or moved out, the
column SHALL be torn down, and a later drop or `Window`-menu show SHALL recreate a
host column rather than leaving a bare strip.

#### Scenario: A tab reorders within its group [m41_drag]

- **WHEN** a tab is dragged to a different index in the same group
- **THEN** the panel order in that group changes

#### Scenario: A tab regroups [m41_drag]

- **WHEN** a tab is dropped on another group's tab bar
- **THEN** the panel joins that group at the drop index and leaves its old group

#### Scenario: A drop between groups inserts a new group [m41_drag]

- **WHEN** a dragged item is dropped on the column background between two groups
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: A group tears off into a frameless overlay and re-docks [m41_tearoff]

- **WHEN** a group is dragged past the column edge and then dropped back on the
  column
- **THEN** it floats in a frameless `Qt::Tool` top-level window and, on the drop,
  re-docks into the column

#### Scenario: A single-tab drag floats only that panel [m43_tabdrag]

- **WHEN** a tab title is dragged out of the column and torn off
- **THEN** a one-panel float is created for that panel only, and the source group
  keeps its other panels

#### Scenario: A header drag floats the whole group [m43_tabdrag]

- **WHEN** the empty header area to the right of the tabs is dragged out and torn
  off
- **THEN** the whole group floats, containing all of its panels

#### Scenario: A floating group separates a tab drag from a header drag [m43_tabdrag]

- **WHEN** a floating group's tab is dragged
- **THEN** only that panel moves, and when its empty header is dragged the whole
  group moves

#### Scenario: A floated tab keeps following until release [m44_floatdrag]

- **WHEN** a tab is dragged outward until a float is created and the mouse keeps
  moving before the button is released
- **THEN** the float keeps following the cursor, and it is committed or re-docked
  only on release

#### Scenario: The primary column is removed when its last panel closes [m46_primary_empty]

- **WHEN** the primary widget column's last visible panel is closed or moved out
- **THEN** the column is removed, and a later show or drop recreates a host
  column instead of leaving a bare empty strip
