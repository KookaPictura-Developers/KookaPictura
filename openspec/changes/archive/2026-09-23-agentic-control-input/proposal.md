## Why

The control server exposes state, vision, and engine actions, but not input, so
an agent cannot exercise the real event paths: a tool drag (marquee/lasso) or a
keyboard shortcut (`Ctrl+Z`) must currently be faked through the method-level
actions. Input synthesis closes P3 of the agentic-control plan, letting an agent
drive the UI the way a user does and then verify with `get_pixel`/`screenshot`.

## What Changes

- `pointer` method: `op` of `click`/`dblclick`/`move`/`drag`/`scroll`, in either
  `window` or `image` space, with `button`, `modifiers`, and `steps`. Synthesizes
  `QMouseEvent`/`QWheelEvent` and delivers them with `QApplication::sendEvent`.
- `key` method: a `sequence` such as `Ctrl+Z`, `B`, or `Shift+F2`, parsed into a
  modifier set and a key, synthesized as `QKeyEvent` press/release and delivered
  with `QApplication::sendEvent`. A synthesized (non-spontaneous) key press is
  routed through the shortcut machinery, so an installed `QAction`/`QShortcut`
  fires; a key no shortcut consumes reaches the widget handler.
- Both go through the application's real event/focus path (no `Qt6::Test` link,
  no parallel reimplementation).
- A self-test block drives a marquee drag with `pointer` and asserts the
  resulting selection, verifies a `key` widget-handler effect, and checks the
  `no_document` image-space path.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agentic-control`: add an **input synthesis** requirement (pointer + key) and
  an input-verification requirement/scenario.

## Impact

- `crates/pictura-app/cpp/control_server.{h,cpp}`, a new
  `control_server_input.cpp` (keeps `control_server.cpp` under its size ceiling),
  `CMakeLists.txt`, and `crates/pictura-app/cpp/selftest_control.cpp`.
- No new dependency, no Rust change.
