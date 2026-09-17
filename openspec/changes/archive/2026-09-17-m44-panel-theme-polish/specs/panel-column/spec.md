## MODIFIED Requirements

### Requirement: Panel column host

The system SHALL host the right-hand panels in a custom `PanelColumn` widget
instead of the `QDockWidget` area, laid out as a vertical stack of `PanelGroup`s
whose heights the user can drag with splitter handles. In `normal` mode the
column SHALL enforce a sensible minimum width so it cannot be squeezed to
nothing. The column SHALL impose no hard panel minimum that forces the main
window taller, and when the available height is less than the groups' size hints
the column SHALL scroll rather than grow the window. Panel resize and scroll
state SHALL survive a relayout. The workspace SHALL support an ordered set of
`PanelColumn`s: zero or more on the left of the document tab area, the document
tabs, and zero or more on the right. A `PanelColumn` SHALL be creatable
dynamically as the result of a drop and SHALL be removed when it holds no groups;
the document tabs SHALL keep the splitter stretch. A column's side SHALL be
determined by its order in the layout relative to the document tabs, not by its
on-screen geometry. A whole widget column SHALL be dockable to any side of the
Tools toolbar, of another widget panel or column, and of the workspace, not only
to the far right, and SHALL use the existing drop grammar and insertion
indicator rather than a second drag system.

#### Scenario: The column hosts panel groups [m41_tabs]

- **WHEN** the frame starts with a fresh session
- **THEN** the right-hand area contains a `PanelColumn` of `PanelGroup`s and no
  right-hand panel `QDockWidget`

#### Scenario: A short window scrolls instead of forcing a minimum [m41_minwidth]

- **WHEN** the main window is shrunk below the column's size hint
- **THEN** the window shrinks and the column scrolls, and no panel forces a
  larger height minimum

#### Scenario: Group heights are draggable [m41_tabs]

- **WHEN** the user drags the handle between two groups
- **THEN** the groups' heights change accordingly

#### Scenario: Normal mode enforces a minimum width [m42_minwidth]

- **WHEN** the splitter handle is dragged to squeeze the column in `normal` mode
- **THEN** the column does not shrink below its normal-mode minimum width

#### Scenario: A new column is created on a side drop [m43_newcolumn]

- **WHEN** a floating group is dropped beside a column, beside a group at the
  workspace edge, or beside the Tools toolbar
- **THEN** a new `PanelColumn` is created on that side of the workspace and the
  group is placed in it

#### Scenario: An emptied column is removed [m43_newcolumn]

- **WHEN** the last group leaves a dynamically created column
- **THEN** the column is removed from the workspace and the document tabs keep
  the stretch

#### Scenario: A whole widget column docks to any side [m44_docksides]

- **WHEN** a whole widget column is dragged to a side of the Tools toolbar, of
  another widget panel or column, or of the workspace
- **THEN** a target on that side is resolved through the existing drop grammar
  and the column is placed there

### Requirement: Compact and iconic mode

In `iconic` mode the column SHALL collapse to a narrow vertical strip
containing one icon button per panel with group dividers between groups, and
SHALL show the panel labels when the strip is widened. Each strip icon button
SHALL be larger than the M41 strip and SHALL render its icon at a larger pixmap
size. Clicking a panel icon SHALL open a frameless `Qt::Popup` flyout
containing that panel's content. The flyout SHALL open on the inner side of the
column: to the left of the strip when the column is on the right, and to the
right of the strip when the column is on the left. The icon whose flyout is open
SHALL render in the active/pressed state. The flyout SHALL be styled as a panel
group: a header naming the panel followed by the panel content, with a close
icon button (a double right chevron) at the right end of the header. The flyout
SHALL present the same tab bar, background, and borders as the docked widget, so
a panel has the same style whether it is docked, shown in a compact flyout, or
floating. The flyout SHALL close on click-away and SHALL NOT steal focus
permanently. The strip width SHALL be a stored value and SHALL be user-resizable.
The strip icon button and pixmap sizes SHALL be larger than the M42 sizes. The
flyout placement SHALL be derived from the actual button geometry and SHALL clamp
only the inner-side coordinate, so the flyout can never cross to the outer side
of the column. The compact strip SHALL show each group's icon buttons as one
visual unit, and SHALL expose a small drag-handle affordance above each group
that drags the whole group. The divider between compact groups SHALL be dark
grey, not white. Strip labels SHALL elide and appear as soon as any room exists,
rather than only once the full label width is available. A drop onto or very
close above/below a compact group SHALL insert into that group at that place,
and a drop between groups, above the top group, or below the bottom group SHALL
create a new group at that boundary.

#### Scenario: Iconic mode collapses to a strip [m41_iconic]

- **WHEN** the column enters iconic mode
- **THEN** it shows a narrow icon strip with group dividers and does not show
  the full panel content

#### Scenario: The strip icons are larger [m42_iconic]

- **WHEN** the iconic strip is built
- **THEN** its icon buttons and icon pixmaps are larger than the M41 strip

#### Scenario: An icon opens a popup flyout [m41_iconic]

- **WHEN** a panel icon in the iconic strip is clicked
- **THEN** a `Qt::Popup` flyout opens with that panel's content

#### Scenario: The flyout opens on the inner side [m42_flyout]

- **WHEN** a panel icon is clicked while the column is on the right
- **THEN** the flyout opens to the left of the strip, and when the column is on
  the left it opens to the right of the strip

#### Scenario: The compact icon grows again [m43_icon]

- **WHEN** the iconic strip is built
- **THEN** its icon buttons and icon pixmaps are larger than the M42 sizes

#### Scenario: The flyout uses the actual button geometry [m43_flyout]

- **WHEN** a panel icon's flyout is placed
- **THEN** the flyout's inner edge meets the clicked button's actual edge and the
  flyout is never placed on the outer side of the column

#### Scenario: The open panel's icon is active [m42_iconic]

- **WHEN** a panel icon's flyout is open
- **THEN** that icon renders in the active/pressed state

#### Scenario: The flyout is group-styled with a header close button [m42_flyout]

- **WHEN** a panel icon's flyout is shown
- **THEN** it presents a header naming the panel above the panel content and a
  close icon button at the right end of the header

#### Scenario: The flyout closes on click-away [m41_iconic]

- **WHEN** a popup flyout is open and the user clicks outside it
- **THEN** the flyout closes

#### Scenario: The compact flyout matches the docked widget [m44_popupstyle]

- **WHEN** a panel is shown in a compact flyout and the same panel is docked
- **THEN** both present the same tab bar, background, and border styling

#### Scenario: The compact divider is dark grey [m44_compactdivider]

- **WHEN** the compact strip is built with more than one group
- **THEN** the divider between groups is dark grey and not white

#### Scenario: Strip labels elide as soon as there is room [m44_elide]

- **WHEN** the compact strip is made slightly wider than the icon button alone
- **THEN** the labels appear and elide to the available width rather than
  staying hidden until the full label width is available

#### Scenario: A compact drop onto a group inserts into it [m44_compactdrop]

- **WHEN** a dragged icon is dropped onto a compact group, or just above or
  below it
- **THEN** the panel is inserted into that group at that place

#### Scenario: A compact drop between or beyond groups creates a group [m44_compactdrop]

- **WHEN** a dragged icon is dropped between two compact groups, above the top
  group, or below the bottom group
- **THEN** a new group is created at that boundary containing the panel

#### Scenario: A group drag handle drags the whole group [m44_draghandle]

- **WHEN** the compact drag-handle above a group's icons is dragged
- **THEN** the whole group is dragged, not a single icon

### Requirement: Panel drag and drop rules

The system SHALL support dragging a panel tab and dragging a group, and SHALL
distinguish them for the whole drag. Dragging a tab SHALL drag only that panel;
dragging the empty header area to the right of the tabs SHALL drag the whole
group. Dragging a tab within its own group SHALL reorder it; dragging a tab onto
another group's tab bar SHALL move the panel into that group at the target index;
dragging onto the column background or between groups SHALL insert a new group at
that boundary; and dragging past the workspace edge SHALL tear the dragged panel
or group off into an in-window floating overlay rather than an operating-system
window. A single-panel drag SHALL float a one-panel float, and a group drag SHALL
float the group. The same tab-versus-header distinction SHALL hold for a floating
group. A floating overlay SHALL be re-dockable into a column, into another group,
or as a new column by dropping it on the target. When a tab is dragged outward to
float, the drag SHALL continue until the mouse button is released: the float
SHALL keep following the cursor, exactly like a group or column drag, and SHALL
only be committed or re-docked on release. A whole widget column SHALL be
draggable to any side of the Tools toolbar, of another widget panel or column,
and of the workspace.

#### Scenario: A tab reorders within its group [m41_drag]

- **WHEN** a tab is dragged to a different index in the same group
- **THEN** the panel order in that group changes

#### Scenario: A tab regroups [m41_drag]

- **WHEN** a tab is dropped on another group's tab bar
- **THEN** the panel joins that group at the drop index and leaves its old group

#### Scenario: A drop between groups inserts a new group [m41_drag]

- **WHEN** a dragged item is dropped on the column background between two groups
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: A group tears off into an overlay and re-docks [m41_tearoff]

- **WHEN** a group is dragged past the column edge and then dropped back on the
  column
- **THEN** it floats in an in-window overlay and, on the drop, re-docks into the
  column

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

### Requirement: Drop insertion indicator

During any panel drag the target column or group SHALL draw a thick blue
insertion line at the candidate drop position: a boundary between groups for a
column/group drop, the target tab index for a tab/regroup drop, or a full-height
line at the workspace edge for a new-column drop. The indicator SHALL be drawn
by the drop target under the pointer and SHALL disappear when the drag leaves or
is cancelled. The M41/M42 indicator look SHALL be kept. The same single
indicator SHALL mark any-side docking targets beside the Tools toolbar, beside
another widget panel or column, and beside the workspace.

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

### Requirement: Panel tab bar active colour

The panel-group tab bar SHALL carry an `objectName` (`panelTabBar`) and SHALL be
styled by a selector scoped to that name. The active tab's background SHALL match
the panel/widget background (the `QTabWidget::pane` `${base}` colour), and an
inactive tab's background SHALL NOT match it (using `${window}` with `${hover}`
on hover). The direction of this mapping is a correctness requirement: the
previous implementation painted the active tab with the inactive `${window}`
colour and vice versa, and that inversion SHALL be corrected rather than
reintroduced. The document tab bar SHALL be unaffected unless its styling is
deliberately changed.

#### Scenario: The active tab matches the pane background [m43_tabcolors]

- **WHEN** a panel group's active tab is inspected
- **THEN** its background is the pane `${base}` colour

#### Scenario: An inactive tab does not match the pane background [m43_tabcolors]

- **WHEN** a panel group's inactive tab is inspected
- **THEN** its background is the `${window}` colour and differs from the pane
  `${base}` colour

#### Scenario: The document tabs are unaffected [m43_tabcolors]

- **WHEN** the panel tab styling is applied
- **THEN** the document tab bar keeps its existing styling

#### Scenario: The active and inactive colours are not swapped [m44_tabswap]

- **WHEN** a panel group's active tab and an inactive tab are compared
- **THEN** the active tab's background matches the pane background and the
  inactive tab's does not, with neither using the other's colour

### Requirement: Multi-column drop targets

The drop resolver SHALL be extended, not duplicated, to resolve drops against
the ordered column set: a drop left or right of an existing column, of a group
at the workspace edge, or of the Tools toolbar SHALL resolve to a new column on
that side; a whole widget column SHALL resolve to a new-column or boundary target
on any side of the Tools toolbar, another widget panel or column, and the
workspace; a drop inside an existing widget group SHALL resolve to a tab insert
into that group, including in compact mode; and a drop above or below an
existing group SHALL resolve to a boundary insert, including in compact mode. In
compact mode a drop onto a group or within a small band just above or below it
SHALL resolve to an insert into that group, and a drop between groups or beyond
the top/bottom group SHALL resolve to a new group at that boundary. The resolver
SHALL cover both normal and iconic modes and both floating and non-floating
drags, and SHALL use one insertion indicator rather than a second indicator
system.

#### Scenario: Dropping left of a column creates a left column [m43_newcolumn]

- **WHEN** a dragged group is dropped to the left of an existing column
- **THEN** a new column is created on the left and the group is placed in it

#### Scenario: Dropping right of a column creates a right column [m43_newcolumn]

- **WHEN** a dragged group is dropped to the right of an existing column
- **THEN** a new column is created on the right and the group is placed in it

#### Scenario: Dropping beside the Tools toolbar creates a column [m43_newcolumn]

- **WHEN** a dragged group is dropped beside the Tools toolbar
- **THEN** a new column is created on that side and the group is placed in it

#### Scenario: Dropping inside a group inserts a tab [m43_intogroup]

- **WHEN** a dragged panel is dropped inside an existing widget group
- **THEN** the panel becomes a tab of that group

#### Scenario: Dropping inside a compact group inserts a tab [m43_intogroup]

- **WHEN** the workspace is in compact mode and a dragged panel is dropped
  inside an existing group
- **THEN** the panel becomes a tab of that group

#### Scenario: Dropping above or below a group inserts at the boundary [m43_boundary]

- **WHEN** a dragged item is dropped above or below an existing group
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: Dropping above or below a compact group inserts at the boundary [m43_boundary]

- **WHEN** the workspace is in compact mode and a dragged item is dropped above
  or below an existing group
- **THEN** a new group is inserted at that boundary containing the dragged item

#### Scenario: A whole widget column resolves to any-side targets [m44_docksides]

- **WHEN** a whole widget column is dragged to a side of the Tools toolbar,
  another widget panel or column, or the workspace
- **THEN** the resolver returns a target on that side and the column is docked
  there through the existing commit path

#### Scenario: A compact group-relative drop is resolved [m44_compactdrop]

- **WHEN** the workspace is in compact mode and an icon is dropped onto a group
  or just above/below it, or between/beyond groups
- **THEN** the resolver returns an into-group target for the former and a new
  boundary group for the latter

## ADDED Requirements

### Requirement: Panel widget theme and borders

Panels and columns SHALL draw a darker grey border from the shared theme: the
Tools toolbar, and both the normal and the compact widget panels. The divider
between two widget groups SHALL be thicker and darker grey than the default
splitter handle. The column collapse toggle's double-chevron icon SHALL match the
action it performs: the collapse-to-icons icon in `normal` mode and the expand
icon in `iconic` mode, with the two icons not inverted. Border and divider
dimensions SHALL be single named constants rather than per-site literals.

#### Scenario: Panels draw the darker grey border [m44_panelborder]

- **WHEN** the Tools toolbar and a normal-mode widget panel are shown
- **THEN** each draws a darker grey border from the shared theme

#### Scenario: The group divider is thicker and darker [m44_divider]

- **WHEN** two widget groups are stacked in a column
- **THEN** the handle between them is thicker and darker grey than the platform
  default

#### Scenario: The collapse toggle icon matches its action [m44_chevrons]

- **WHEN** the column collapse toggle is inspected in `normal` and in `iconic`
  mode
- **THEN** each mode shows the icon for the action it performs and the two icons
  are not inverted

### Requirement: Widget presentation parity

A widget SHALL use the same colour scheme and style whether it is docked, shown
in a compact flyout, or floating. The three presentations SHALL share the same
scoped stylesheet selectors and the same container chrome (tab bar, background,
borders) rather than per-presentation colour literals.

#### Scenario: The flyout and float match the docked widget [m44_popupstyle]

- **WHEN** the same panel is presented docked, in a compact flyout, and in a
  float
- **THEN** all three use the same background, borders, and tab-bar styling

### Requirement: Default active panel is first

On a fresh session the first panel SHALL be the active one, not the last, in
whichever artefact carries the default active state — the default active tab of
a group, the default current panel of a column, or the default current page. The
first tab of each default group SHALL be current, and the column's current panel
SHALL be the first visible panel.

#### Scenario: The first panel is active by default [m44_defaultactive]

- **WHEN** the frame starts with a fresh session
- **THEN** the first tab of each default group is current and the first visible
  panel is the column's current panel
