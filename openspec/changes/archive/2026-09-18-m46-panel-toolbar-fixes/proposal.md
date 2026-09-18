## Why

The M45 "panel fixes" pass claimed to close the widget-panel and Tools-toolbar
reports but shipped change whose tests bypass the real interactive paths and
assert configuration instead of resulting geometry. The user, running the
current `build/pictura`, still sees every reported defect. This change closes
the remaining gaps and replaces the blind tests with checks that drive the
actual gesture/geometry under test.

## What Changes

- **Drop indicator correctness.** Right-side tab drops draw their line on the
  left; dragging between columns shows no line; a rightmost tab draws the
  leftmost line; a bottom-of-panel drop draws no line. The tab-insertion
  geometry is derived from visible tabs only, cross-column delegation accepts
  every valid target (not just `IntoGroup`), the new-column owner is the edge
  column, and the boundary line is clamped inside the scroll viewport.
- **Toolbar drop targets.** Dragging the floating Tools title bar never reaches
  the custom resolver because Qt's dock drag grabs the mouse; moves/releases
  are delivered to the dock, not the title bar. The gesture is captured on the
  dock while floating, and the resolver accepts a drop on any side of any
  widget column, not only the workspace edges. **BREAKING** (internal): the
  Tools panel can be re-hosted as a central-splitter pane at any column
  boundary.
- **Empty columns.** Closing every panel removes its column, including the
  primary column, instead of leaving a bare 180 px strip. Live floats are
  rehomed rather than blocking removal, and non-drag cleanup paths re-test
  emptiness.
- **Minimize.** A minimized group collapses to its tab bar regardless of its
  content minimum (min/max both clamped), and the menu reads `Expand Panel`
  while minimized (already partly landed; verified here).
- **Sizing.** Widget columns have one shared minimum floor and cannot be
  collapsed to zero; narrow columns scroll horizontally instead of clipping
  their right side.
- **Compact popup.** The compact-strip popup hosts the whole widget group with
  the clicked panel current (parity with the docked and floating presentations),
  and a dragged group's placement line is drawn above the group's grip.
- **Tests.** Self-tests drive real mouse events and assert widget/indicator
  geometry inside the viewport, hidden-tab groups, cross-column body/boundary
  targets, splitter collapsibility, and the primary-column removal path.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `panel-column`: drop-indicator geometry/ownership, cross-column delegation,
  empty-column removal (including the primary column), shared minimum width and
  non-clipping scroll, minimize collapse height, compact popup group parity,
  and the compact group-drag indicator position.
- `tool-framework`: the floating Tools title-bar drag gesture and the drop
  grammar that lets the floating panel dock beside any widget column.
- `application-shell`: the central splitter is non-collapsible and the primary
  widget column participates in empty-column removal.

## Impact

- `crates/pictura-app/cpp/panels/panel_group.cpp` / `.h` — tab insertion
  geometry, minimize min-height clamp.
- `crates/pictura-app/cpp/panels/panel_column.cpp` / `.h` — drop resolver,
  indicator geometry/ownership, width floor, float/empty cleanup.
- `crates/pictura-app/cpp/toolbox.cpp` / `.h` — floating title-bar gesture.
- `crates/pictura-app/cpp/frame.cpp` / `.h` — toolbar resolver fallback,
  splitter collapsibility, primary-column removal.
- `crates/pictura-app/cpp/main.cpp` — self-tests.
- No new dependencies; Qt6/QSplitter/QScrollArea only.
