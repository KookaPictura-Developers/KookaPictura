## Why

The shell has a working toolbox, menu bar, and status bar, but no imagery: tool
buttons are text-only, menu actions carry no icons, the window uses the platform
default icon, and the tools set stock Qt cursors. A professional editing app is
read at a glance, and CS6's tools/cursors are immediately recognizable. M19 adds
an original SVG icon set and SVG cursors, and wires them through the shell.

All artwork is **independent-creation original** — geometric stroke art in the project's
own style, not derived from Adobe's assets.

## What Changes

- Add an **original SVG icon set** under `assets/icons/` covering the
  application icon, every implemented tool, and every command that has a working
  handler (File, Edit, Image, Select, View, Window, Help).
- Add an **SVG cursor set** under `assets/cursors/`, one per tool, each with a
  defined hotspot.
- Bundle the assets in a **Qt resource** (`assets/pictura.qrc`) so they ship in
  the binary, and add a small loader (`icons.{h,cpp}`) with
  `QIcon icon(const QString& id)` and `QCursor cursor(const QString& id)`.
- Use the icons: the application/window icon, the Tools panel buttons, the
  options bar, and menu actions; use the cursors for the active tool.
- Add the **Qt6::Svg** module to the build (required to render SVG into
  `QIcon`/`QCursor`) and enable `AUTORCC` for the resource.
- Extend `--self-test` to assert every icon and cursor id resolves to a
  non-null `QIcon`/`QCursor` and that the window icon is set.

## Capabilities

### New Capabilities

- `icon-assets`: the SVG icon set, the resource bundle, the icon loader, and the
  application of icons to the window, the Tools panel, the options bar, and menu
  actions.
- `svg-cursors`: the per-tool SVG cursor set with hotspots, and the active tool
  applying its cursor to the canvas.

### Modified Capabilities

_None._

## Impact

- Affected crate: `pictura-app` (new `assets/`, `icons.{h,cpp}`, toolbox/tool/
  frame/main wiring).
- Build: `CMakeLists.txt` gains `Qt6::Svg`, `CMAKE_AUTORCC`, and the `.qrc`. This
  is a new Qt module (not a third-party dependency); it is required to render
  SVG into `QIcon`/`QCursor`.
- Licensing: all SVGs are original, hand-authored paths. No Adobe or third-party
  icon assets are used.
- Verification: `--self-test` gains icon/cursor resolution checks (exit codes
  from 47); existing behavior unchanged.
