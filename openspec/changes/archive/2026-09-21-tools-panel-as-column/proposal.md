# Tools panel as a first-class column

## Why

The Tools toolbar is still a `QDockWidget`: it floats as a separate OS top-level
window and is bolted on beside the `PanelColumn` system. That gives it different
drag, float, and stacking behaviour from every widget panel, and the gaps show —
a release in the workspace outer band previews a column but docks elsewhere, a
floating Tools window appears in the task list, and widget-panel drags lack
whole-column float, float drop targets, group tabify feedback, full-width header
backgrounds, whole-drag dimming, and floating-icon parity. Making the Tools
panel a first-class, tabless, atomic column removes the special case and closes
those widget-panel drag gaps in one grammar.

## What Changes

- The Tools panel becomes a tabless, atomic column hosted like a `PanelColumn`,
  with a column header and the tool grid as one plain content child — no tab bar
  and no `PanelGroup`.
- The tools column and widget panels cannot be combined in either direction: no
  tabify, no inserting a widget panel into the tools column, and no dropping the
  tools column into a widget group, panel, column, or float. They can coexist as
  sibling columns. No insertion indicator is shown for a forbidden combination.
- The tools header toggle shares the widget-column toggle style but switches the
  tool grid between one and two columns; the tools column has no iconic rail
  mode.
- The tools column floats as an in-window overlay only, never as an OS window.
  **BREAKING**: this reverses the previous "floats as an independent window"
  contract.
- A release in the workspace outer left/right band commits the tools column on
  that side, so the blue indicator matches the actual placement.
- Whole widget columns can be dragged and floated as an in-window overlay, and
  redocked, keeping a minimum width and a resize grip.
- An in-window floating panel is a drop target for a dragged panel or group.
- Dropping a whole group onto another group tabifies/merges them, and the target
  group shows a blue outline around its whole region.
- The panel-group header background spans the full group width, including behind
  the right-hand context-menu (`▾`) corner button.
- A dragged panel or group is dimmed for the whole drag and leaves no ghost on
  cancel.
- A floating compact/icon row behaves like a docked icon strip: clicking an icon
  opens the flyout and the group grip drags the group.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tool-framework`: the Tools panel is an atomic tabless column that floats
  in-window, its header toggle switches the tool-grid width, and an edge release
  commits the column; the "release outside the main window floats at the cursor"
  placement requirement is removed.
- `application-shell`: the Tools panel is no longer a standalone dock or a
  central-splitter pane, and the tools column fills its column height.
- `panel-column`: whole-column in-window float, floating panels as drop targets,
  group-on-group tabify with a blue outline, full-width group header background,
  whole-drag dim, and floating icon-row parity.

## Impact

- **C++ app**: `toolbox.cpp`/`toolbox.h`, `frame_columns.cpp`, `frame_build.cpp`,
  `frame.h`, a new `panels/panel_column_float.cpp`, `panels/panel_column_drag.cpp`,
  `panels/panel_float.cpp`, `session.cpp`/`session.h`, `frame_session.cpp`,
  `CMakeLists.txt`, and the self-test units.
- **No document-format change, no new dependency, no `docs/` edit.**
