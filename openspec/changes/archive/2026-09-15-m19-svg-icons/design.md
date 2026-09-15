## Context

The M18 toolbox has a `QActionGroup` of text actions, the frame builds menus
from the command registry, the window has no icon, and `ToolController` sets
stock Qt cursors. There is no asset pipeline: no `assets/` directory, no `.qrc`,
and Qt6::Svg is not linked, so nothing can render SVG.

## Goals / Non-Goals

**Goals:**

- An original, independent-creation SVG icon set for the app, the tools, and every
  implemented command.
- An SVG cursor per tool with a defined hotspot.
- A resource bundle and a tiny loader, used by the window, Tools panel, options
  bar, menu actions, and the active tool's cursor.
- Self-test coverage that every asset id resolves.

**Non-Goals:**

- Icons for unimplemented tools or the full 500-entry menu tree (dead assets).
- A recolourable/themed icon system; M19 ships neutral light-gray stroke art.
- Animated or multi-state cursors; one cursor per tool.
- Replacing the CS6 visual theme work (colours/metrics) beyond icons.

## Decisions

### Original stroke art, id-named files

Icons are 24×24 viewBox SVGs using simple geometry and a stroke/fill in a
neutral light gray (`#c8c8c8`) with an optional highlight accent (`#3d6f99`).
File names equal the icon id: `assets/icons/<id>.svg`, and cursors are
`assets/cursors/<id>.svg`. Command icons use the command id verbatim (dots
included); tool icons/cursors use `tool.<tool>`. This lets the loader take an id
directly with no mapping table.

- *Why:* no mapping to drift; a command's icon is `icon(commandId)`.
- *independent-creation:* all paths are authored for this project; no Adobe assets.

### Frozen asset id set

Application: `app`.
Tools (icons and cursors): `tool.move`, `tool.marquee`, `tool.lasso`,
`tool.quickselection`, `tool.crop`, `tool.eyedropper`, `tool.hand`, `tool.zoom`.
Command icons: `file.new`, `file.open`, `file.save`, `file.saveAs`,
`file.revert`, `file.close`, `file.closeAll`, `file.exit`, `edit.undo`,
`edit.redo`, `edit.stepForward`, `edit.stepBackward`, `image.rotate90cw`,
`image.rotate90ccw`, `image.rotate180`, `image.flipHorizontal`,
`image.flipVertical`, `image.crop`, `select.all`, `select.deselect`,
`view.zoomIn`, `view.zoomOut`, `view.fitOnScreen`, `view.actualPixels`,
`view.screenMode.standard`, `view.screenMode.fullWithMenuBar`,
`view.screenMode.full`, `view.options`, `window.panels.layers`,
`window.panels.tools`, `help.about`.

### Qt resource + `Qt6::Svg` + `AUTORCC`

`assets/pictura.qrc` lists every SVG with prefix `/`, so `icon(id)` resolves
`:/icons/<id>.svg`. CMake adds `Qt6::Svg` (needed to render SVG via `QIcon` and
`QSvgRenderer`), enables `CMAKE_AUTORCC`, and links `Qt6::Svg`.

- *Why:* embedding avoids runtime file lookup; `Qt6::Svg` is the canonical Qt
  module for SVG rendering and is the only new build component. No third-party
  dependency is added.
- *Alternatives considered:* PNG raster assets (rejected: no HiDPI/scaling
  benefit, larger diffs); loading from a filesystem path (rejected: breaks
  installs).

### Loader API

```cpp
namespace pictura {
QIcon icon(const QString& id);      // QIcon(":/icons/" + id + ".svg")
QCursor cursor(const QString& id);  // rendered SVG pixmap + hotspot
}
```

`cursor` renders the SVG with `QSvgRenderer` into a 24×24 `QPixmap` (scaled by
the app device-pixel-ratio), sets the pixmap's device pixel ratio, and uses a
hotspot table. Hotspots (nominal 24×24): all tools `(12,12)` except
`tool.eyedropper` `(2,22)` (tip at lower-left) and `tool.hand` `(12,12)`. Cursor
art is drawn so its action point sits at the hotspot.

- *ponytail:* one fixed hotspot per tool and a single DPR render; a per-cursor
  multi-resolution set can come later if HiDPI cursors look soft.

### Wiring

- `main.cpp`: `QApplication::setWindowIcon(icon("app"))`.
- Toolbox: each tool `QAction` gets `icon("tool." + toolName(id))`.
- `ToolController`: on active-tool change, `canvas->setCursor(cursor(...))`.
- Frame: after `buildMenuBar`, iterate the implemented-command id list and call
  `registry_->action(id)->setIcon(icon(id))` when the action exists.
- Options bar: the combine-mode and tolerance controls keep text; the bar's
  window icon is not required.

## Risks / Trade-offs

- **Asset quality is subjective** → Keep icons simple and legible; the set is
  functional, and a later visual pass can refine paths without code changes.
- **`QSvgRenderer` needs the Qt SVG plugin at runtime** → Linking `Qt6::Svg`
  and embedding via qrc removes the file/plugin-discovery risk; the self-test
  asserts non-null icons/cursors.
- **New build component** → `Qt6::Svg` is a Qt module already present wherever
  `Qt6::Widgets` is; CMake fails clearly if it is missing.
- **HiDPI cursor softness** → Documented ceiling (single DPR render).

## Migration Plan

Additive. New `assets/`, `icons.{h,cpp}`; CMake and wiring edits. No behavior
change to documents, tools, or menus beyond added icons.

## Open Questions

- Exact CS6 tool-icon silhouettes are proprietary; M19 uses original geometry
  that reads the same at 16–24 px and does not copy Adobe artwork.
- Whether the options bar should gain icons; deferred unless cheap.
