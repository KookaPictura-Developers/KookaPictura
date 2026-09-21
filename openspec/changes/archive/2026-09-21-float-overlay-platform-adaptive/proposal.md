# Platform-adaptive floating overlay hosting

## Why

Floating panels, columns, and toolbars do not follow the cursor when dragged.
The overlay (`PanelFloat`) is a frameless `Qt::Tool` top-level window, but on
Wayland a client may not position its own top-level windows, so every
`QWidget::move()` on the overlay is ignored. The overlay only follows the cursor
on platforms that let a client position top-levels (X11, offscreen). The earlier
"can float outside the app window" contract silently assumed one of those.

## What Changes

- `PanelFloat` chooses its window kind at runtime: a frameless `Qt::Tool`
  top-level where the platform permits client positioning, otherwise an
  in-window child of the owning frame.
- `PanelFloat::overlayUsesTopLevel()` reports the decision; a `*ForTest` override
  forces child mode so the branch is covered on a top-level platform.
- `PanelColumn::moveFloat` and `PanelColumn::floatBounds` branch on the same
  helper: top-level mode stays screen-clamped and moves by global coordinates;
  child mode clamps to the owning frame rect and moves by parent-relative
  coordinates (which is what `QWidget::move` expects for a child).
- In both modes the overlay stays parented to the owning frame, and `raise()`
  plus drag/follow still work.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `panel-column`: the floating overlay's hosting becomes platform-conditional in
  the floating-panel-overlay and whole-column-float requirements, with child
  mode clamped to the frame rect and following the cursor parent-relative.

## Impact

- **C++ app**: `panels/panel_float.cpp`, `panels/panel_column.h`,
  `panels/panel_column_float.cpp`, `panels/panel_column_test.cpp`, and the
  round-4 shell self-test.
- **No document-format change, no new dependency, no `docs/` edit.**
