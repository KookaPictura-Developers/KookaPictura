# Design

## Context

`PanelFloat` hosts a torn-off panel group or whole column. It was made a
frameless `Qt::Tool` top-level window so it could cross outside the app window.
That works on X11 and offscreen, but Wayland does not allow a client to position
its own top-level windows, so `QWidget::move()` on the overlay is ignored and the
overlay never follows the cursor. The hosting therefore cannot be a single
compile-time choice.

## Goals / Non-Goals

**Goals:**

- The overlay follows the cursor on every platform.
- Where client positioning of top-levels is permitted, keep the existing
  frameless `Qt::Tool` window and its ability to cross outside the app window.
- On Wayland (or wherever client positioning is refused), host the overlay as an
  in-window child of the frame, clamped to the frame rect.
- Keep the drag path, drop grammar, redock, session shape, and the existing
  top-level checks unchanged.

**Non-Goals:**

- No change to drop resolution, the insertion indicator, or the non-floating
  layout.
- No new dependency, no `docs/` change, no decorated window.

## Decisions

### D1. One runtime switch, one helper

`PanelFloat::overlayUsesTopLevel()` returns false when
`QGuiApplication::platformName()` contains `wayland` (case-insensitive), or when
a test override forces child mode; otherwise true. The override
(`setForceChildOverlayForTest`) exists only because the offscreen platform runs
in top-level mode, so child mode would otherwise be untested.

### D2. The constructor picks the window kind

When `overlayUsesTopLevel()` the constructor keeps
`Qt::Tool | Qt::FramelessWindowHint`; otherwise it uses `Qt::Widget` so the
overlay is an in-window child of its frame. Everything else (styled background,
minimum width, opacity effect, custom grip, no decorations) is shared.

### D3. Movement branches on the same helper

`PanelColumn::moveFloat` and `PanelColumn::floatBounds` branch on
`overlayUsesTopLevel()`:

- **Top-level mode**: clamp to the available geometry of the screen under the
  point and move by global coordinates — the existing behaviour, unchanged.
- **Child mode**: clamp to the owning frame's rect and move by parent-relative
  coordinates via `host->mapFromGlobal(globalTopLeft)`, which is what
  `QWidget::move` expects for a child.

The overlay stays parented to the owning frame in both modes, so `raise()` and
the drag/follow path work unchanged.

## Risks / Trade-offs

- **Child mode cannot leave the app window.** That is inherent: a child is
  clipped to its parent. The trade-off is cursor-following over crossing outside
  the window, and it is the only option Wayland permits.
- **The platform string check is a heuristic.** `QGuiApplication::platformName()`
  is `"wayland"` on the Wayland QPA plugin today; the substring match also
  catches `"wayland-egl"`. If a future plugin refuses client positioning under
  another name, the test override gives the branch an escape hatch.
- **Environment override.** A user can force the Wayland plugin off with
  `QT_QPA_PLATFORM=xcb`; then top-level mode is chosen, which is correct because
  X11 permits client positioning.
