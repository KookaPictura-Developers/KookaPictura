## 1. Brief and frozen asset contract (orchestrator)

- [x] 1.1 Write `docs/dev/m19-svg-icons.md` milestone brief
- [x] 1.2 Freeze the asset id set and loader signatures in `design.md`
- [x] 1.3 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Assets A: application + tools + cursors (sub-agent)

- [x] 2.1 `assets/icons/app.svg`
- [x] 2.2 Eight tool icons `assets/icons/tool.*.svg`
- [x] 2.3 Eight tool cursors `assets/cursors/tool.*.svg` with the action point at the hotspot (eye-dropper lower-left; others centred)

## 3. Assets B: File and Edit command icons (sub-agent)

- [x] 3.1 File icons: `file.new/open/save/saveAs/revert/close/closeAll/exit`
- [x] 3.2 Edit icons: `edit.undo/redo/stepForward/stepBackward`

## 4. Assets C: Image, Select, View, Window, Help command icons (sub-agent)

- [x] 4.1 Image icons: `image.rotate90cw/rotate90ccw/rotate180/flipHorizontal/flipVertical/crop`
- [x] 4.2 Select icons: `select.all/select.deselect`
- [x] 4.3 View icons: `view.zoomIn/zoomOut/fitOnScreen/actualPixels/screenMode.*/options`
- [x] 4.4 Window icons: `window.panels.layers/tools`; Help icon: `help.about`

## 5. Loader, resource, build (sub-agent)

- [x] 5.1 `assets/pictura.qrc` listing every icon and cursor with prefix `/`
- [x] 5.2 `icons.{h,cpp}` with `QIcon icon(id)` and `QCursor cursor(id)` (QSvgRenderer, hotspot table, DPR render)
- [x] 5.3 `CMakeLists.txt`: `Qt6::Svg`, `CMAKE_AUTORCC`, the `.qrc` and `icons.{h,cpp}`

## 6. Wiring (sub-agent)

- [x] 6.1 `main.cpp`: set the application/window icon from `app`
- [x] 6.2 Tools panel actions carry `tool.<tool>` icons
- [x] 6.3 Active tool sets its SVG cursor on the canvas
- [x] 6.4 Implemented menu actions carry their command icon
- [x] 6.5 Existing self-tests (M16–M18) still exit 0

## 7. Self-test, verification, close-out

- [x] 7.1 Extend `--self-test` (exit codes from 47): every icon id resolves to a non-null `QIcon`; every cursor id resolves; window icon set
- [x] 7.2 `cmake -S . -B build && cmake --build build`
- [x] 7.3 fixture and no-argument self-tests exit 0
- [x] 7.4 `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
- [x] 7.5 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [x] 7.6 Archive the change and commit
