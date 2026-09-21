# Workspace panel icon mode, drop reach, and minimum width

## Why

A round of workspace-panel defects remains after the column/float refactor:

- **Drop beside the docked Tools column draws no mark.** A widget panel or
  group dragged next to the docked Tools toolbar resolves a new-column target
  but `showColumnEdgeIndicator` suppresses every mark whose owner is the Tools
  column, so the user sees no line on the side the new column would land.
- **Floating icon mode still looks like a normal panel.** A collapsed floating
  group keeps the resize grip, keeps the normal-width floor, and lays its icons
  out horizontally, so it neither hugs its content nor reads like the docked
  strip's vertical icon column.
- **Floating icon widgets cannot be dragged.** Only the group grip starts a
  drag; the icons themselves are inert, so a panel in a floating icon row cannot
  be torn out and an icon-mode group cannot be dropped onto another floating
  panel.
- **Dragging a group in front of a group only shows a top/bottom line.** A
  whole-group drag over another group's body resolves an above/below boundary,
  not a tabify target, so the blue region outline never appears and the drop
  inserts a new group instead of merging.
- **The Layers blend-mode control hogs the control row.** The blend combo takes
  all the row's stretch factor while Opacity is fixed, so the blend input is far
  wider than the CS6 proportion.
- **Widget columns are too narrow.** The shared normal-mode minimum width is
  180px, below the 300px a widget panel needs.

## What Changes

- `PanelColumn::resolveDrop`/`showColumnEdgeIndicator`: a new-column target
  resolved *beside* the Tools column draws the mark at that edge; only a drag
  over the Tools column body that resolves no beside-column target stays
  mark-free (the atomic rule).
- `PanelFloat::syncToContent`/`setResizable`: a collapsed-to-icons overlay hides
  the grip and shrink-wraps on both axes; expanding restores the grip and the
  normal minimum width.
- `PanelGroup::rebuildIconRow`: the floating icon row stacks its icons
  vertically; `makeIconButton` makes each icon a drag source through the same
  `tabDragStarted`/`dragMoved`/`dragFinished` grammar the docked strip uses.
- `PanelColumn::resolveLocalDrop`/`updateDrag`: a whole-group drag over a
  different group's body resolves a tabify target, so the blue region outline is
  drawn and the drop merges the groups.
- `layers_panel.cpp`: the blend combo and the opacity field share the control
  row proportionally instead of the combo taking all the slack.
- `panel_column.cpp`/`panel_column.h`: the shared normal-mode minimum width
  floor rises to 300px, and the floating overlay minimum matches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `panel-column`: the Drop insertion indicator rule for the Tools column, the
  floating icon mode (size, grip, vertical icons, draggable icons), the
  group-on-group tabify outline for body drops, the floating drop target for an
  icon-mode group, and the 300px shared minimum width.
- `layers-panel`: the property control row proportions.

## Impact

- **C++ app**: `panels/panel_column_drag.cpp`, `panels/panel_column_indicator.cpp`,
  `panels/panel_column_float.cpp`, `panels/panel_float.cpp`,
  `panels/panel_group.cpp`, `panels/panel_group.h`, `panels/panel_column.cpp`,
  `panels/panel_column.h`, `panels/layers_panel.cpp`, the panel/group test hooks,
  and the round-4 shell self-test.
- **No document-format change, no new dependency, no `docs/` edit.**
