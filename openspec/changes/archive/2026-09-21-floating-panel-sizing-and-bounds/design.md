# Design

## Context

`PanelFloat` is the overlay that hosts a torn-off panel group or a whole
`PanelColumn`. After the tools panel became a first-class column, the overlay is
still an in-window child (`setWindowFlags(Qt::Widget)`) with a stock `QSizeGrip`,
and it is clamped to the main window's central area. That contract now produces
five defects: the grip resizes the main window, the Tools column is resizable
when it should not be, a floating widget column opens at full docked height, a
collapsed icon row leaves a normal-width body, and a float cannot leave the main
window even though it is also not a real window.

## Goals / Non-Goals

**Goals:**

- The overlay's resize grip resizes the overlay, never the main window.
- A non-resizable overlay mode for the floating Tools column, sized to its
  content minimum.
- A floating widget column defaults to roughly two thirds of its docked height.
- A collapsed icon row snaps both its height and its width; a floating iconic
  column snaps too.
- The overlay is a frameless `Qt::Tool` top-level parented to the main window,
  movable anywhere on the screen but clamped to the screen's available geometry.

**Non-Goals:**

- No change to the drag grammar, drop resolution, redock, session shape, or the
  non-floating docked layout.
- No decorated window frame, title bar, taskbar entry, or window-manager
  interaction beyond what `Qt::Tool | Qt::FramelessWindowHint` gives.
- No new dependency and no `docs/` change.

## Decisions

### D1. A custom grip, not `QSizeGrip`

`QSizeGrip` is hard-wired to resize the top-level window, which is the main
window once the overlay is a child. Replace it with a small `QWidget` subclass
that records the press position and start size and resizes the owning
`PanelFloat` by the drag delta, clamped to `minimumSize()`. A `*ForTest` drag
driver is not added; the self-test synthesizes mouse events on the real grip, so
the production handler is what is exercised and the resized window cannot drift
from the assertion. The accessor `sizeGripForTest()` returns `QWidget*` because
the concrete grip type is private to the translation unit.

### D2. A non-resizable overlay mode

Add `PanelFloat::setResizable(bool)`. When false it hides the grip and sizes the
overlay to the minimum its content needs (`content_->minimumWidth` /
`minimumSizeHint`). The floating Tools column uses it: `floatColumn` detects
`isToolsColumn()`, drops the overlay minimum width to zero, and lets the content
pin the size, because the tool grid is narrower than the widget-column floor.
The tools overlay therefore cannot be dragged larger or smaller.

### D3. Floating column default height

`PanelColumn::floatColumn` currently uses the full docked height. A widget-column
overlay now opens at two thirds of the docked height, clamped to the overlay
minimum. Two thirds keeps enough context to read the panels while leaving the
desktop visible — a chosen proportion, not a sourced CS6 metric.

### D4. Collapse snaps width as well as height

`syncToContent`'s collapsed branch resizes to `resize(width(), target)`, keeping
the normal width. It now also sets the width to the icon row's natural width
(`group_->sizeHint().width()`, at least `kFloatMinWidth`). A whole-column float
has no `group_` and never snapped; it now snaps to the hosted column's content
when the column is iconic, driven from `setRailMode` so a rail toggle while
floating resizes the overlay.

### D5. Frameless tool window, screen-clamped

The overlay uses `Qt::Tool | Qt::FramelessWindowHint` and stays parented to the
main window, so it is transient for the frame (above it, hidden with it) without
a title bar and without a taskbar entry. Movement is by global coordinates and is
clamped to the available geometry of the screen under the target point (falling
back to the frame's screen, then the primary screen), replacing the
central-area/tools clamp. This is the one behavioural reversal: the overlay can
now sit outside the main window, and it is a top-level window — but never a
decorated one.

## Risks / Trade-offs

- **Tool windows can be hidden behind a full-screen frame.** `Qt::Tool` is
  transient for the main window, so it stays above the frame; it is not a
  free-standing `Qt::Window`. This is the intended "no taskbar, no decorations"
  trade-off.
- **The screen clamp is per-target-screen**, computed at move time. A float
  dragged toward a monitor edge follows the pointer screen; on a monitor that
  disappears mid-drag it falls back to the frame's screen.
- **Non-resizable Tools overlay width** is the tool grid content width, which is
  narrower than the widget-column floor. The overlay deliberately has no minimum
  width in that mode so it hugs the grid.
- The change reverses an archived contract (`panel-column`'s "in-window child,
  never a top-level"). The deltas record the new wording in `panel-column`, and
  also in the `tool-framework` and `application-shell` requirements that
  previously said the floating Tools column is an in-window overlay, never an
  operating-system window.
