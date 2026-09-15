## Why

M23 grouped six panels but the right side is still not the CS6 workspace: the
reference (`docs/02-ui-ux/reference/cs6-workspace.png`) shows three tabbed groups
(Color/Swatches/Gradients/Patterns, Properties/Adjustments/Libraries,
Layers/Channels/Paths) plus a narrow right icon rail for panels that stay
collapsed until clicked — History, Actions, Info, and more reachable from
`Window > Panels`. Eight of those panels do not exist yet.

## What Changes

- Add minimal placeholder panels for Gradients, Patterns, Properties,
  Adjustments, Libraries, Channels, Paths, and Actions, each a dockable
  `QDockWidget` with a stable `objectName` and a CS6-appropriate empty state
  (e.g. Properties shows "No Properties").
- Regroup the default right docks into the three CS6 tabbed groups.
- Add a right **icon rail**: a narrow vertical column of buttons that toggles
  each collapsed panel (History, Actions, Info, Navigator, Histogram).
- Implement `Window > Panels` commands for the new panels so the rail and the
  menu share one toggle path.
- Placeholder panels contain no real functionality; their contents are deferred.

## Capabilities

### New Capabilities

- `panel-rail`: the right icon rail and the panel-toggle command set that the
  rail and the `Window` menu share.

### Modified Capabilities

- `application-shell`: the "Default dock grouping and canvas colour" requirement
  changes to the full CS6 groups and the collapsed rail panels.

## Impact

- New `crates/pictura-app/cpp/panels/placeholder_panel.{h,cpp}` and
  `panels/panel_rail.{h,cpp}`.
- `frame.{h,cpp}` registers the new panels, the three groups, and the rail;
  `commands.h`/`command_tree.cpp` gain the new panel ids and mark those
  `Window > Panels` entries implemented.
- `CMakeLists.txt` gains the panel sources.
- No Rust, document-format, or dependency changes.
