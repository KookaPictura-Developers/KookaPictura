## Why

A second round of widget-panel/toolbar defects remains after M46. Root causes
were traced end-to-end (see design.md): an empty source column is kept alive by
its live float, groups with no visible tabs are left as un-draggable "ghosts", a
compact icon drag deletes the widget that holds its mouse grab, compact drops
can't create a group above, columns still clip/h-scroll, iconic columns are
resized by a neighbour, the floating Tools gesture only works after the dock is
already floating and can't land among columns, and a floating widget has no
close affordance.

## What Changes

- **Empty/ghost cleanup.** A column whose last group is dragged out and left
  floating disappears (its floats are re-homed to the primary column first). A
  group whose tabs are all hidden is hidden, not left as an empty, un-grabbable
  shell.
- **Compact strip interactions.** Dragging a compact icon follows the cursor as
  a float and commits on release (no more dead drag); dragging a single compact
  panel onto a group's grip creates a new group above it.
- **Sizing.** Normal widget columns are wide enough that content is always
  horizontally visible — no horizontal scrollbar and no right-side clipping. An
  iconic column has a fixed width and is not resized by a neighbouring pane's
  handle drag, on either side of the workspace.
- **Toolbar placement.** The floating Tools panel can be dropped to either side
  of any widget column, and between columns, by dragging its title bar; a
  docked/floating widget can also be dragged across and around the toolbar.
- **Floating widget chrome.** A floating widget group shows a close button at
  the rightmost side of its header; closing re-homes and hides the group so
  `Window > Panels` can restore it.
- **Compact chrome.** The compact group container uses the panel surface shade
  and its drag dots are dark gray.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `panel-column`: empty/ghost cleanup, compact drag fidelity and group creation,
  width floor with no horizontal overflow, fixed iconic width, floating overlay
  close control, and float bounds across the toolbar.
- `tool-framework`: the Tools panel is a movable central-splitter pane that can
  be placed on either side of any widget column, and is transparent as an
  obstacle to widget-panel drags.
- `application-shell`: the central splitter hosts the Tools pane and the compact
  panel chrome shades.

## Impact

- `crates/pictura-app/cpp/panels/panel_column.cpp` / `.h`
- `crates/pictura-app/cpp/panels/panel_group.cpp` / `.h`
- `crates/pictura-app/cpp/frame.cpp` / `.h`
- `crates/pictura-app/cpp/toolbox.cpp` / `.h`
- `crates/pictura-app/cpp/theme.cpp`
- `crates/pictura-app/cpp/main.cpp` (self-tests)
- No new dependencies.
