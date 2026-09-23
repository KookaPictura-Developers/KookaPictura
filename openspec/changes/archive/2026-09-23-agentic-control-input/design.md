## Context

Actions (`set_tool`, `selection`, `filter`, …) call methods directly and never
touch the widget event path, so a shortcut (`.ui` `QAction` / `QShortcut`) or a
tool drag cannot be exercised. The plan
(`docs/dev/mcp-agentic-control-plan.md` §5.6) specifies two methods, `pointer`
and `key`, delivered as synthesized Qt events. `ImageView` exposes
`widgetToImage` (`image_view.cpp:492`, `(widget - offset) / zoom`) with public
`zoom()`/`offset()`, so the inverse is `widget = image * zoom + offset`.

## Goals / Non-Goals

- **Goal:** drive the real event/focus path so a drag and a shortcut behave as
  they do for a user, then verify with `get_pixel`/`screenshot`.
- **Non-goal:** no `Qt6::Test` link, no `QTest::mouseClick` — those bring a test
  framework into the shipped binary.
- **Non-goal:** global/desktop coordinates. `ui_tree` reports window-local rects,
  so `pointer space=window` is frame-local, avoiding Wayland global-coordinate
  problems.

## Decisions

### Coordinates and the target widget

- `space = "image"`: the point is document pixels on the active `ImageView`. Map
  `widget = image * zoom + offset` and send the event to that `ImageView`. No
  active document → `no_document`.
- `space = "window"`: the point is frame-local (what `ui_tree` reports). The
  target is `frame_->childAt(point)` when one exists, else the frame; the event
  position is `target->mapFrom(frame_, point)`, and `globalPos` is
  `frame_->mapToGlobal(point)`.

### Event shapes

| `op` | events |
|---|---|
| `click` | press, release |
| `dblclick` | press, release, `MouseButtonDblClick`, release |
| `move` | one mouse-move |
| `drag` | press at (x,y), `steps` interpolated moves, release at (x2,y2) |
| `scroll` | one `QWheelEvent` with `angleDelta().y() = 120 * steps` |

`button` defaults to left; `modifiers` accepts a small set of names
(`shift`/`ctrl`/`alt`/`meta`, or an int bitmask). `steps` is bounded (drag
1..256, default 8; scroll default 1) so a hostile value cannot spin the GUI
thread. Modern `QMouseEvent`/`QWheelEvent` constructors take `QPointF` positions;
every event is delivered with `QApplication::sendEvent`.

### Key parsing and focus

A `sequence` such as `Ctrl+Z` is split on `+`: the last token is the key, the
earlier tokens are modifiers. Key names map to `Qt::Key` (single letters/digits,
`F1`..`F35`, and named keys: `Enter`/`Return`, `Escape`/`Esc`, `Tab`, `Space`,
`Delete`, `Backspace`, `Home`, `End`, `PageUp`, `PageDown`, the arrows). An
unknown key or modifier is `invalid_param`. The press (and release) is sent to
`QApplication::focusWidget()` when present, else `frame_`.

**Shortcut routing (Qt 6.11):** `QApplication::sendEvent` produces a
non-spontaneous `KeyPress`, and Qt deliberately routes such an event through the
shortcut machinery: `QApplication::notify` sends a `ShortcutOverride` first "to
ensure that any matching shortcut is triggered first"
(`src/widgets/kernel/qapplication.cpp`). So an installed `QAction`/`QShortcut`
whose shortcut matches fires, and the widget never sees the key; a key no
shortcut consumes reaches `keyPressEvent`.

The caveat is window activation: a `WindowShortcut` matches only while its window
is active. The in-process self-test runs before `app.exec()` activates the
frame, so only the widget-handler path is observable there; a live run (and the
P5 `verify-control.sh`) exercises the shortcut path. A probe that tested before
activation wrongly concluded shortcuts never fire.

**Ceiling:** `key` is delivered by `sendEvent`, not by faking the platform
input, so it cannot drive native/global shortcuts (menu-bar `Alt` accelerators
handled by the platform, `QWindow`-level grabs). Those remain out of scope.

### Placement

`control_server.cpp` is 872 lines and the repo has a 1200-line cap; the two
methods go in a new `control_server_input.cpp` (translation unit of the same
class, mirroring `control_server_actions.cpp`). The declarations and dispatch
entries are added to `control_server.h`/`control_server.cpp`; `CMakeLists.txt`
lists the new source.

## Risks / Trade-offs

- A shortcut matches only in an active window: the in-process self-test observes
  the widget-handler path; a live run observes the shortcut path. The agent
  verifies any key effect with a readback rather than assuming.
- Offscreen `QApplication::focusWidget()` may be null; the frame fallback keeps
  widget key handlers working headless, which the self-test exercises.
