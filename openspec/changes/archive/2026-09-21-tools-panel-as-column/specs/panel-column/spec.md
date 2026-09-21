## MODIFIED Requirements

### Requirement: Floating group top bar and icon mode

A floating panel group SHALL carry its own top bar with a normal/icon width
toggle and a close control, the toggle placed immediately to the left of the
close control on the bar's right side, and the bar SHALL drag the overlay. The
toggle SHALL collapse the group to its icon row and expand it again, and the
close control SHALL remain reachable while collapsed. While collapsed the overlay
SHALL snap to the height its icon row needs, at least the icon-row minimum, and
the icon row SHALL render as the docked icon strip's group box with the same
container styling, grip divider, and icon size. The overlay SHALL offer a resize
grip and keep a minimum size like a docked column. When the floating group holds
a single visible panel, dragging its tab SHALL move the whole overlay rather than
tear off a second overlay and leave a ghost. A dragged overlay SHALL be dimmed
for the whole drag, from the start of the gesture until it ends, not only while
it is over a valid drop target.

#### Scenario: The float top bar toggles and closes [pc_float_header]

- **WHEN** a group floats
- **THEN** its top bar shows a collapse toggle immediately left of a close
  control on the right, toggling collapses and expands the group, and dragging
  the bar moves the overlay

#### Scenario: The floating icon overlay snaps to the icon row [pc_float_icon_min]

- **WHEN** a floating group is collapsed to icons
- **THEN** the overlay height snaps to the icon-row height (at least the
  icon-row minimum), the icon row renders as a grouped panel-icon box, and the
  close control remains reachable

#### Scenario: The floating overlay is resizable [pc_float_resize]

- **WHEN** a floating group is shown
- **THEN** it carries a resize grip and a minimum size comparable to a docked
  column

#### Scenario: A single-panel float moves as a whole [pc_float_single_drag]

- **WHEN** the tab of a floating group that holds one visible panel is dragged
- **THEN** the whole overlay moves and no second overlay or empty source overlay
  is left behind

#### Scenario: A dragged overlay dims for the whole drag [pc_float_dim]

- **WHEN** an overlay is dragged
- **THEN** it is dimmed from the start of the drag through release, including
  while the target is invalid, and it returns to full opacity when the drag ends

## ADDED Requirements

### Requirement: Whole-column in-window float

The system SHALL allow a whole widget `PanelColumn` to be dragged by its header
and torn off the workspace into an in-window floating overlay, through the same
drag machinery and the same single insertion indicator the panel and group drags
use. The overlay SHALL be an in-window child widget of the frame, never an
operating-system top-level window. It SHALL keep a minimum width and SHALL offer
a resize grip. A release over a valid column or workspace edge SHALL re-place the
column; a release with no valid target SHALL leave the overlay floating.

#### Scenario: Dragging a whole column floats it in-window [pc_col_float]

- **WHEN** a widget column's header is dragged out of the workspace
- **THEN** the whole column floats in an in-window overlay that follows the
  cursor and is committed on release

#### Scenario: The floating column keeps a minimum width and a grip [pc_col_float_grip]

- **WHEN** a floating whole-column overlay is shown
- **THEN** it carries a resize grip and cannot be resized below a minimum width

#### Scenario: The floating column re-places [pc_col_redock]

- **WHEN** a floating whole-column overlay is dropped on a column or workspace
  edge
- **THEN** the whole column re-places at that position and the overlay is removed

### Requirement: Floating panel as a drop target

A floating panel overlay SHALL be a drop target in the same grammar as a column.
A dragged single panel dropped onto a floating overlay SHALL become a tab of that
overlay's group, and a dragged group dropped onto it SHALL merge into that
overlay's group. The overlay SHALL show the shared insertion indicator while it
is the resolved target.

#### Scenario: A dragged panel drops into a float [pc_float_drop_panel]

- **WHEN** a single panel is dragged onto an existing in-window floating panel
- **THEN** the panel becomes a tab of the floating group and the shared
  indicator marks the target

#### Scenario: A dragged group merges into a float [pc_float_drop_group]

- **WHEN** a whole group is dragged onto an existing in-window floating panel
- **THEN** the dragged group's panels join the floating group

### Requirement: Group-on-group tabify outline

Dropping a whole group onto another group SHALL merge the two groups, with the
dragged group's panels joining the target group as tabs. While a dragged group is
resolved to a target group, the target group SHALL be highlighted with a blue
outline drawn around its whole region rather than only the thin insertion line,
and the outline SHALL clear when the drag moves away or is cancelled.

#### Scenario: A group dropped on a group tabifies into it [pc_group_tabify]

- **WHEN** a whole group is dropped onto another group
- **THEN** the dragged group's panels become tabs of the target group

#### Scenario: The target group is outlined [pc_group_outline]

- **WHEN** a dragged group is resolved to a target group
- **THEN** a blue outline is drawn around the target group's whole region, and it
  clears when the drag leaves or is cancelled

### Requirement: Group header full-width background

The panel-group header background SHALL span the full width of the group,
including the area behind the right-hand per-widget context-menu corner button,
so the header reads as one continuous surface with no gap and no differently
coloured corner region.

#### Scenario: The header background covers the corner button [pc_header_full_width]

- **WHEN** a panel group's header is shown
- **THEN** the header background extends behind the corner context-menu button
  with no gap or separate corner colour

### Requirement: Whole-drag dim and cancel cleanup

A dragged panel, group, or column SHALL be dimmed for the entire drag, from the
start of the gesture until it ends, not only while the pointer is over a valid
drop target. When a drag is cancelled, the dragged source SHALL leave no ghost
overlay and the layout SHALL be unchanged.

#### Scenario: A dragged item dims for the whole drag [pc_whole_drag_dim]

- **WHEN** a panel, group, or column is dragged
- **THEN** it is dimmed from the start of the drag through release, including
  while the target is invalid

#### Scenario: A cancelled drag leaves no ghost [pc_drag_cancel_clean]

- **WHEN** a drag is cancelled
- **THEN** the dragged source returns to full opacity with no ghost overlay and
  the layout is unchanged

### Requirement: Floating icon row parity

A floating compact/icon row SHALL behave like the docked icon strip. Clicking an
icon in a floating icon row SHALL open the same `Qt::Popup` flyout hosting the
whole group with the clicked panel current, and the group's drag-handle grip
SHALL drag the whole group through the same drag grammar the docked strip uses.
The flyout SHALL open on the inner side of the floating row and SHALL restore the
group when it closes.

#### Scenario: Clicking a floating icon opens the flyout [pc_float_icon_click]

- **WHEN** an icon in a floating compact row is clicked
- **THEN** the same popup flyout as the docked strip opens with the clicked panel
  current

#### Scenario: The floating grip drags the group [pc_float_icon_grip]

- **WHEN** the drag-handle grip above a floating row's group is dragged
- **THEN** the whole group is dragged through the docked strip's drag grammar
