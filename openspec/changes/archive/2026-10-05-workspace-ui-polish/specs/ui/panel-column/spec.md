## MODIFIED Requirements

### Requirement: Panel groups and top tabs

Each `PanelGroup` SHALL present its panels in a `QTabWidget` with the tab
position explicitly set to `QTabWidget::North`, so the tabs render on top of the
group on every platform and stylesheet. The tab text SHALL be the panel's title,
there SHALL be no separate group-title label widget, and a group containing a
single panel SHALL still show that panel's tab. In `normal` mode a tab SHALL show
its text only and no icon; the panel icon SHALL be retained for the iconic/icon
strip and shown there, not baked into the normal tab.

#### Scenario: Tabs are on top [m41_tabs]

- **WHEN** any group is shown
- **THEN** its tab position is `QTabWidget::North`

#### Scenario: The tab is the only group chrome [m41_tabs]

- **WHEN** a group's tab bar is inspected
- **THEN** the group has no title label beyond the tabs, and each tab's text is
  the panel title

#### Scenario: A single-panel group shows its tab [m41_tabs]

- **WHEN** a group contains exactly one panel
- **THEN** that panel's tab is shown

#### Scenario: Normal tabs carry no icon [pc_tabs_iconless]

- **WHEN** a panel group is shown in `normal` mode
- **THEN** each tab shows its text only with no icon, while the iconic/icon strip
  still shows the panel icon

### Requirement: Corner action button visibility

Each panel group's header SHALL always show the full tab bar and the corner
context-action button together, including at the column's minimum width. Tab
text SHALL elide rather than push the corner button out, the header SHALL reserve
the corner button's width, and the column's minimum-width calculation SHALL
account for the corner button. The corner button SHALL NOT be clipped or pushed
outside the group header at the minimum width. The corner button SHALL use the
bundled Lucide `menu` glyph (asset id `panel.menu`) rather than a text character,
SHALL be vertically centred in the header row, and SHALL keep a right margin from
the group's right edge.

#### Scenario: The corner button stays visible at minimum width [m43_corner]

- **WHEN** the column is at its minimum width
- **THEN** the corner context-action button is fully inside the group's
  tab-header row and is not clipped

#### Scenario: Tab text elides instead of hiding the corner button [m43_corner]

- **WHEN** the group's available tab-bar width is less than the tab text needs
- **THEN** the tab text elides and the corner button remains visible

#### Scenario: The corner button uses the menu glyph [pc_corner_menu_glyph]

- **WHEN** a group header's corner button is shown
- **THEN** it draws the `panel.menu` Lucide glyph, centred vertically with a
  right margin from the group's right edge

### Requirement: Panel widget theme and borders

Panels and columns SHALL draw a darker grey border from the shared theme: the
Tools toolbar, and both the normal and the compact widget panels. The divider
between two widget groups SHALL be thicker and darker grey than the default
splitter handle. The line below a widget column's header and the last line at the
column's bottom SHALL each be 3 px. The column collapse toggle's double-chevron
icon SHALL match the action it performs: the collapse-to-icons icon in `normal`
mode and the expand icon in `iconic` mode, with the two icons not inverted.
Border and divider dimensions SHALL be single named constants rather than
per-site literals.

#### Scenario: Panels draw the darker grey border [m44_panelborder]

- **WHEN** the Tools toolbar and a normal-mode widget panel are shown
- **THEN** each draws a darker grey border from the shared theme

#### Scenario: The group divider is thicker and darker [m44_divider]

- **WHEN** two widget groups are stacked in a column
- **THEN** the handle between them is thicker and darker grey than the platform
  default

#### Scenario: The column header and bottom rules are 3 px [pc_column_rule_3px]

- **WHEN** a widget column is shown
- **THEN** the line below its header and the last line at its bottom are each
  3 px from the shared constant

#### Scenario: The collapse toggle icon matches its action [m44_chevrons]

- **WHEN** the column collapse toggle is inspected in `normal` and in `iconic`
  mode
- **THEN** each mode shows the icon for the action it performs and the two icons
  are not inverted

### Requirement: Group header full-width background

The panel-group header background SHALL match the inactive tab background and
SHALL span the full width of the group, including the area behind the right-hand
per-widget context-menu corner button, so the header reads as one continuous
surface with no gap, no differently coloured corner region, and no lighter
horizontal line above the tabs.

#### Scenario: The header background covers the corner button [pc_header_full_width]

- **WHEN** a panel group's header is shown
- **THEN** the header background matches the inactive tab background, extends
  behind the corner context-menu button with no gap or separate corner colour,
  and draws no lighter horizontal line

## ADDED Requirements

### Requirement: Column-edge self-anchor docking

A panel, group, or whole column dragged to the edge band immediately to the left
or right of its own source column SHALL be able to anchor on that column, so a
widget docked on one side of a column can be dropped on the other side and vice
versa. The whole-column move target resolution SHALL keep its existing behaviour.

#### Scenario: A widget docks to the opposite side of its own column [pc_self_anchor_dock]

- **WHEN** a panel or group docked on the left of a column is dragged to the
  right edge band of that same column, or a right-docked one to the left edge
  band
- **THEN** the edge anchor accepts the source column and on release the item
  lands on that opposite side

### Requirement: Fixed flush Tools column

The Tools column SHALL sit in a fixed left slot adjacent to the workspace without
being a splitter pane, so it abuts the workspace directly with no resize handle,
no gap, and no drag seam between them, and its width SHALL NOT change on a
pointer drag. The widget columns beside the workspace SHALL keep their resize
handle and SHALL stay width-draggable from their workspace-facing edge.

#### Scenario: No seam beside the Tools column [pc_tools_flush]

- **WHEN** the Tools column is shown beside the workspace
- **THEN** there is no resize handle, gap, or drag seam between the Tools column
  and the workspace, and its width does not change on a pointer drag

#### Scenario: Widget columns stay draggable [pc_tools_flush_widget_drag]

- **WHEN** a widget column beside the workspace is dragged by its
  workspace-facing handle
- **THEN** its width changes and the workspace absorbs the difference

### Requirement: Iconic strip group separators

In the iconic/icon-strip mode each widget group SHALL draw a 1 px separator line
at its bottom, SHALL NOT draw a line between its drag grip and its icon items,
and the strip SHALL NOT draw a thicker divider between adjacent groups.

#### Scenario: Group bottom line, no grip line, no inter-group divider [pc_iconic_group_lines]

- **WHEN** the iconic strip is shown with several widget groups
- **THEN** each group has a single 1 px line at its bottom, there is no line
  between a group's drag grip and its icons, and there is no thicker divider
  above a group

### Requirement: Iconic column workspace-edge resizing

An iconic/icon-strip column SHALL be resizable from its workspace-facing edge
when it is docked on either the left or the right of the workspace — a
right-docked column from its left edge and a left-docked column from its right
edge — and the resulting width SHALL persist.

#### Scenario: Resize from the workspace edge on either side [pc_iconic_resize]

- **WHEN** an iconic column is docked on the right and its left edge is dragged,
  or docked on the left and its right edge is dragged
- **THEN** its width changes in both cases and the new width is kept
