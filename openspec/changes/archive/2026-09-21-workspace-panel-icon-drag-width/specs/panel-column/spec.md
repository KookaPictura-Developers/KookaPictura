## MODIFIED Requirements

### Requirement: Drop insertion indicator

During any panel drag the target column or group SHALL draw a thick blue
insertion line at the candidate drop position: a boundary between groups for a
column/group drop, the target tab index for a tab/regroup drop, or a full-height
line at the workspace edge for a new-column drop. The indicator SHALL be drawn
by the drop target under the pointer and SHALL disappear when the drag leaves or
is cancelled. The M41/M42 indicator look SHALL be kept. The indicator SHALL be
positioned from the same resolved `DropTarget` the commit uses, in the column
that owns that target, so the line and the drop can never disagree: a drop on
the right of a group's tab bar SHALL draw the line at that tab index on the
right, including when the group has hidden tabs so the index maps past them; a
drop into another column's group, group body, or group boundary SHALL draw the
line in that target column; a drop as the rightmost tab SHALL draw the line at
the rightmost tab position and not at the far left; a drop at the bottom boundary
SHALL draw the line at the bottom of the last visible group, clamped inside the
scroll viewport so the line is never entirely clipped; and a whole-group drag in
compact mode SHALL draw the line above the group's drag-handle dots, not inside
the group below them. The column that owns a new-column line SHALL be the column
adjacent to the workspace edge the new column will occupy, not the drag source.
The same single indicator SHALL mark any-side docking targets beside the Tools
toolbar, beside another widget panel or column, and beside the workspace. The
new-column line SHALL be drawn above any floating overlay that follows the drag,
so an in-window overlay that tracks the cursor cannot hide it. The atomic Tools
column SHALL draw no line for a drag over its own body that resolves no
beside-column target; a drag that resolves a new column immediately beside the
Tools column SHALL draw the new-column mark at that edge like any other column.

#### Scenario: The blue line marks the target boundary [m41_drop]

- **WHEN** a drag hovers over the column between two groups
- **THEN** a thick blue insertion line is drawn at that boundary

#### Scenario: The indicator clears on cancel [m41_drop]

- **WHEN** a drag is cancelled or leaves the column
- **THEN** no insertion line is drawn

#### Scenario: The indicator marks a new-column target [m43_newcolumn]

- **WHEN** a drag hovers beyond the workspace edge on a side
- **THEN** the blue indicator marks the new-column position on that side

#### Scenario: The indicator marks an any-side dock target [m44_docksides]

- **WHEN** a whole widget column is dragged near a side of the Tools toolbar,
  another widget panel or column, or the workspace
- **THEN** the single blue indicator marks the target position on that side

#### Scenario: The indicator side matches the drop [m45_indicator_side]

- **WHEN** a tab is dragged onto the right side of a group's tab bar
- **THEN** the blue line is drawn on the right at the resolved insertion index,
  and the committed drop lands at that same index

#### Scenario: A group with hidden tabs still draws the right line [m46_indicator_hidden_tab]

- **WHEN** a tab is dragged onto the right side of a group that contains a hidden
  tab
- **THEN** the blue line is drawn at the right insertion position and the drop
  lands at that same index

#### Scenario: A cross-column drop shows the indicator in the target column [m45_indicator_cross_column]

- **WHEN** a tab is dragged from one widget column onto another widget column's
  tab bar, group body, or group boundary
- **THEN** the blue line is drawn in the target column at the resolved insertion
  point, and the committed drop places the tab there

#### Scenario: The rightmost-tab drop draws the line at its own index [m45_indicator_rightmost_tab]

- **WHEN** a tab is dropped as the rightmost tab of a group
- **THEN** the blue line is drawn at that tab's insertion position and not at the
  far left of the tab bar

#### Scenario: The bottom-boundary drop draws the line at the bottom [m45_indicator_bottom]

- **WHEN** a group or panel is dragged to the bottom boundary of the column
- **THEN** the blue line is drawn inside the scroll viewport at the bottom of the
  last visible group, not clipped below it

#### Scenario: The compact group drag draws the line above the dots [m45_compact_group_line]

- **WHEN** a whole group is dragged in compact mode and the target is a boundary
  before a group
- **THEN** the blue line is drawn above that group's drag-handle dots rather
  than inside the group below them

#### Scenario: The new-column mark stays above a following overlay [fp_edge_mark_above_overlay]

- **WHEN** a floating overlay is dragged over a dock spot while still floating
- **THEN** the new-column mark is drawn above the overlay and remains visible,
  instead of being hidden under the overlay that follows the cursor

#### Scenario: The atomic Tools column draws no widget drop line [fp_edge_mark_not_on_tools]

- **WHEN** a widget panel is dragged over the docked Tools column's body and
  resolves no beside-column target
- **THEN** no new-column mark is drawn on the Tools column

#### Scenario: A bare workspace-edge mark is drawn on the requested side [fp_edge_mark_workspace_side]

- **WHEN** a column is dragged to the leftmost workspace side while the only
  other column is on the right
- **THEN** the mark is drawn at the central area's left edge, not at the
  right-hand column's edge, and it is committed at the splitter head

#### Scenario: A drag beside the docked Tools column shows the mark [wpx_tools_side_mark]

- **WHEN** a widget panel or group is dragged into the new-column band
  immediately beside the docked Tools column
- **THEN** the thick blue new-column mark is drawn at that side of the Tools
  column, on the side the new column would occupy

### Requirement: Panel column minimum width and no clipping

Every widget column in `normal` mode SHALL enforce a single shared minimum width
floor of at least 300 device-independent pixels, equal for all widget columns and
for every floating widget overlay, large enough that the column's content is
always fully visible horizontally, so no column can be resized below it or
disappear and no column ever displays a horizontal scrollbar or hides content on
its right edge. The floor SHALL be raised further by content that needs more,
capped at the shared maximum, and SHALL be applied to the column so a wider
column — not internal scrolling — is what keeps content visible. The column
SHALL elide its tab text and keep the header corner action button inside the
header rather than cutting either off. The iconic strip SHALL keep its own
narrow, fixed minimum separate from the shared normal-mode floor.

#### Scenario: All widget columns share one minimum width [m45_min_width_floor]

- **WHEN** two or more widget columns are present and their minimum widths are
  queried
- **THEN** every normal-mode column reports the same shared floor and none can be
  resized below it or vanish

#### Scenario: A column never horizontally scrolls or clips its content [m47_no_hscroll]

- **WHEN** a widget column is at its minimum width
- **THEN** its horizontal scrollbar policy is off, its content fits within the
  viewport, and no right-side content is hidden

#### Scenario: The shared floor is at least 300 pixels [wpx_min_width_300]

- **WHEN** any normal-mode widget column and any floating widget overlay are at
  their minimum width
- **THEN** each reports a minimum width of at least 300 pixels

### Requirement: Floating group top bar and icon mode

A floating panel group SHALL carry its own top bar with a normal/icon width
toggle and a close control, the toggle placed immediately to the left of the
close control on the bar's right side, and the bar SHALL drag the overlay. The
toggle SHALL collapse the group to its icon row and expand it again, and the
close control SHALL remain reachable while collapsed. While collapsed the
overlay SHALL use the smallest size its icon row needs on both axes — snapping
to the icon row's height and shrinking its width to the icon row's own width so
no normal-width body remains — and SHALL NOT offer the resize grip; expanding
SHALL restore the resize grip and the shared normal minimum width. The icon row
SHALL render as the docked icon strip's group box with the same container
styling, grip divider, and icon size, and SHALL stack its icons in a vertical
column, one icon per row, matching the docked strip. Each icon in the floating
row SHALL be draggable through the same drag grammar the docked strip uses, so a
panel can be torn out of the floating group and the group can be dragged and
dropped elsewhere; a click below the drag threshold SHALL still open the panel.
The overlay SHALL offer a resize grip that resizes the overlay itself and keep a
minimum size like a docked column. When the floating group holds a single visible
panel, dragging its tab SHALL move the whole overlay rather than tear off a
second overlay and leave a ghost. A dragged overlay SHALL be dimmed for the whole
drag, from the start of the gesture until it ends, not only while it is over a
valid drop target.

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

#### Scenario: A collapsed float shows no resize grip [wpx_float_icon_no_grip]

- **WHEN** a floating group is collapsed to icons
- **THEN** the overlay does not show its resize grip, keeps the smallest width
  and height its icon row needs, and shows the grip again when expanded

#### Scenario: The floating icon row stacks vertically [wpx_float_icon_vertical]

- **WHEN** a floating group with more than one icon is collapsed to icons
- **THEN** its icons are arranged one per row in a vertical column

#### Scenario: A floating icon drags its panel [wpx_float_icon_drag]

- **WHEN** an icon in a floating collapsed group is pressed and dragged past the
  drag threshold
- **THEN** that panel starts a drag through the same grammar the docked strip
  uses and can be dropped elsewhere, while a click below the threshold opens the
  panel

#### Scenario: The floating overlay is resizable [pc_float_resize]

- **WHEN** a floating group is shown expanded and its corner grip is dragged
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

### Requirement: Group-on-group tabify outline

Dropping a whole group onto another group SHALL merge the two groups, with the
dragged group's panels joining the target group as tabs. While a dragged group is
resolved to a target group — whether the pointer is over that group's tab bar or
its body — the target group SHALL be highlighted with a blue outline drawn
around its whole region rather than only the thin insertion line, and the outline
SHALL clear when the drag moves away or is cancelled. A drop over the dragged
group's own body SHALL remain an above/below reorder boundary so groups can still
be reordered in place.

#### Scenario: A group dropped on a group tabifies into it [pc_group_tabify]

- **WHEN** a whole group is dropped onto another group
- **THEN** the dragged group's panels become tabs of the target group

#### Scenario: The target group is outlined [pc_group_outline]

- **WHEN** a dragged group is resolved to a target group
- **THEN** a blue outline is drawn around the target group's whole region, and it
  clears when the drag leaves or is cancelled

#### Scenario: A group body drop outlines the whole target group [wpx_group_body_outline]

- **WHEN** a whole group is dragged over another group's body, away from its tab
  bar
- **THEN** the blue outline is drawn around that whole group and releasing merges
  the dragged group into it

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

#### Scenario: A floating icon group merges into another float [wpx_float_drop_icon_group]

- **WHEN** a collapsed floating group is dragged by the drag grammar onto another
  floating panel
- **THEN** its panels join that floating group's tabs
