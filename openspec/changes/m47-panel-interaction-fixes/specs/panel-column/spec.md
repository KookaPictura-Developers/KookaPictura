## MODIFIED Requirements

### Requirement: Compact and iconic mode

In `iconic` mode the column SHALL collapse to a narrow vertical strip
containing one icon button per panel with group dividers between groups, and
SHALL show the panel labels when the strip is widened. Each strip icon button
SHALL be larger than the M41 strip and SHALL render its icon at a larger pixmap
size. Clicking a panel icon SHALL open a frameless `Qt::Popup` flyout that hosts
the **same `PanelGroup`** as the docked group: the group's full tab set, with
the clicked panel set current, styled and behaving exactly as when docked or
floating, with no parity differences between the three presentations, and the
group SHALL be restored to its column position when the flyout closes. The
flyout SHALL open on the inner side of the column: to the left of the strip when
the column is on the right, and to the right of the strip when the column is on
the left. The icon whose flyout is open SHALL render in the active/pressed
state. The flyout SHALL close on click-away and SHALL NOT steal focus
permanently. The strip width SHALL be fixed while the column is iconic, so a
neighbouring pane's splitter handle drag SHALL NOT resize it. The strip icon
button and pixmap sizes SHALL be larger than the M42 sizes. The flyout placement
SHALL be derived from the actual button geometry and SHALL clamp only the
inner-side coordinate, so the flyout can never cross to the outer side of the
column. The compact strip SHALL show each group's icon buttons as one visual
unit, and SHALL expose a small drag-handle affordance above each group that
drags the whole group. The divider between compact groups SHALL be dark grey,
not white; the group container SHALL use the panel surface shade and the drag
dots SHALL be dark gray. Strip labels SHALL elide and appear as soon as any room
exists, rather than only once the full label width is available. A drop onto or
very close above/below a compact group SHALL insert into that group at that
place; a drop on a group's drag-handle grip SHALL create a new group directly
above that group; and a drop between groups, above the top group, or below the
bottom group SHALL create a new group at that boundary. Pressing and dragging a
compact icon SHALL tear that panel into a floating overlay that follows the
cursor until the mouse button is released and is committed on release, exactly
like a tab or group drag.

#### Scenario: Iconic mode collapses to a strip [m41_iconic]

- **WHEN** the column enters iconic mode
- **THEN** it shows a narrow icon strip with group dividers and does not show
  the full panel content

#### Scenario: The strip icons are larger [m42_iconic]

- **WHEN** the iconic strip is built
- **THEN** its icon buttons and icon pixmaps are larger than the M41 strip

#### Scenario: The popup hosts the whole group with the clicked panel active [m45_popup_group]

- **WHEN** a panel icon is clicked in the iconic strip
- **THEN** the popup hosts the whole `PanelGroup` with all of the group's tabs,
  the clicked panel is current, and the group is restored to its column position
  when the popup closes

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

#### Scenario: A compact icon drag follows the cursor [m47_compact_icon_float]

- **WHEN** a compact strip icon is pressed and dragged away from the strip
- **THEN** a floating overlay appears and follows the cursor until release, and
  the release commits the drop

#### Scenario: A compact panel drop on the grip creates a group above [m47_compact_grip_group]

- **WHEN** a single compact panel is dropped on a group's drag-handle grip
- **THEN** a new one-panel group is created directly above that group

#### Scenario: An iconic column is not resized by its neighbour [m47_iconic_fixed_width]

- **WHEN** a normal column and an iconic column are adjacent and the splitter
  handle between them is dragged
- **THEN** the iconic column keeps its strip width and only the normal column
  resizes

### Requirement: Panel column minimum width and no clipping

Every widget column in `normal` mode SHALL enforce a single shared minimum width
floor, equal for all widget columns, large enough that the column's content is
always fully visible horizontally, so no column can be resized below it or
disappear and no column ever displays a horizontal scrollbar or hides content on
its right edge. The floor SHALL be derived from the column content's minimum
size plus the scroll chrome, capped at a sane maximum, and SHALL be applied to
the column so a wider column — not internal scrolling — is what keeps content
visible. The column SHALL elide its tab text and keep the header corner action
button inside the header rather than cutting either off. The iconic strip SHALL
keep its own narrow, fixed minimum separate from the shared normal-mode floor.

#### Scenario: All widget columns share one minimum width [m45_min_width_floor]

- **WHEN** two or more widget columns are present and their minimum widths are
  queried
- **THEN** every normal-mode column reports the same shared floor and none can be
  resized below it or vanish

#### Scenario: A column never horizontally scrolls or clips its content [m47_no_hscroll]

- **WHEN** a widget column is at its minimum width
- **THEN** its horizontal scrollbar policy is off, its content fits within the
  viewport, and no right-side content is hidden

### Requirement: Floating panel overlay

A group torn off a column SHALL float in an in-window overlay that follows the
cursor and re-docks on release, and the overlay SHALL be able to cross and be
placed around the docked or pane-hosted Tools panel without being clamped at the
central widget's edge. The overlay SHALL show a close control at the rightmost
side of its header; closing SHALL hide the group's panels while keeping the
group restorable from `Window > Panels`, then remove the overlay.

#### Scenario: A torn-off group floats and re-docks [m41_tearoff]

- **WHEN** a group is dragged out of a column and dropped back on a column
- **THEN** it floats in an in-window overlay and re-docks

#### Scenario: A floating group can be closed [m47_float_close]

- **WHEN** a floating group's header close control is activated
- **THEN** the overlay is removed, its panels are hidden, and the group is still
  present in a column so `Window > Panels` can restore it

#### Scenario: The overlay can cross the Tools panel [m47_float_over_tools]

- **WHEN** a group is dragged toward the docked Tools panel
- **THEN** the overlay follows the cursor over the Tools panel instead of
  stopping at the central area edge

## ADDED Requirements

### Requirement: Empty columns and ghost groups are cleaned up

A column that has no group with visible content SHALL be removed (dynamic) or
hidden (primary host) after a drop, even while it owns a live floating overlay;
before removal its floats SHALL be re-homed to the primary column so they stay
re-dockable. A group whose tabs are all hidden SHALL be hidden rather than left
as a visible but un-grabbable shell, and SHALL remain restorable from
`Window > Panels`.

#### Scenario: The source column disappears after its last group floats [m47_empty_after_float]

- **WHEN** the last group of a dynamic column is dragged out and left floating
- **THEN** the empty source column is removed and the float remains re-dockable

#### Scenario: A group with no visible tabs is hidden [m47_ghost_group]

- **WHEN** the last visible tab of a group is moved elsewhere
- **THEN** the emptied group is hidden instead of showing an empty tab bar, and
  its hidden panels can be re-shown from `Window > Panels`
