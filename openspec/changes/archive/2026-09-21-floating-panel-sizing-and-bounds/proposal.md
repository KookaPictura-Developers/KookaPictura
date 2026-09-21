# Floating panel sizing and screen bounds

## Why

The floating panel overlay (`PanelFloat`) has five sizing and windowing defects
that survived the tools-panel-as-column change:

1. **Resizing a float resizes the whole app.** The overlay's grip is a stock
   `QSizeGrip`, which resizes the top-level window — the main window — instead of
   the overlay it sits on.
2. **The floating Tools column can still be resized.** It has no meaningful
   content-driven size and should be fixed to the minimum its content needs.
3. **A floating widget column opens at its full docked height**, taller than the
   user needs and taller than the docked column it came from.
4. **A floating collapsed-to-icons group keeps a normal-width body**, because the
   collapse path only changes the height; whole-column floats never snap at all.
5. **A float cannot leave the main window and is not a decorated window either.**
   The overlay is an in-window child clipped to the frame, so it cannot be moved
   over other applications or across monitors, and it is not an independent
   operating-system top-level.

## What Changes

- The overlay's resize grip becomes a custom control that resizes the
  `PanelFloat` itself, clamped to its minimum size, and never touches the main
  window.
- `PanelFloat` gains a non-resizable mode: the grip is removed and the overlay
  takes the minimum its content needs. The floating Tools column uses it and
  sizes to the tool grid's content width and the content's minimum height.
- A floating widget column opens at roughly two thirds of its docked height
  (never below the overlay minimum).
- Collapsing a floating group to icons shrinks the overlay's width to the icon
  row's natural width as well as its height; a whole-column float hosting an
  iconic widget column snaps the same way.
- The overlay becomes a frameless `Qt::Tool` top-level window parented to (and
  transient for) the main window: no title bar, no taskbar entry, movable
  anywhere on the screen, clamped to the screen's available geometry rather than
  the main window rect.
- **BREAKING**: this reverses the previous "in-window child, never a top-level"
  contract for the floating overlay.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `panel-column`: the floating overlay's windowing contract (in-window child →
  frameless tool window), its resize grip (resizes the overlay, not the frame),
  a non-resizable mode for the floating Tools column, the collapsed-icon width
  snap, the floating-column default height, screen-clamped movement, and the
  tear-off rules for panels and groups.
- `tool-framework`: the floating Tools column's windowing contract (in-window
  overlay → frameless, non-resizable `Qt::Tool` window movable outside the main
  window and clamped to the screen) in the Tools panel and Tools panel column
  layout requirements.
- `application-shell`: the floating Tools panel's windowing contract in the
  standalone-dock requirement, reworded from an in-window overlay to a frameless
  `Qt::Tool` top-level window.

## Impact

- **C++ app**: `panels/panel_float.cpp`, `panels/panel_column.h`,
  `panels/panel_column_float.cpp`, `panels/panel_column_test.cpp`,
  `panels/panel_column_iconic.cpp`, and the shell self-test units.
- **No document-format change, no new dependency, no `docs/` edit.**
