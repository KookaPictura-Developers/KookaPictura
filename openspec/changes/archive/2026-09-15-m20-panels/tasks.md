## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m20-panels.md` milestone brief
- [x] 1.2 Freeze the panel bind/refresh interfaces and the bridge additions in `design.md`
- [x] 1.3 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Wave 1a: labeled history + layer properties (sub-agent, Rust)

- [x] 2.1 `history.rs`: store a label per captured state; `count`, `index`, `label(i)`, `jump(i)`, `add_snapshot(label)`
- [x] 2.2 Bridge: `history_count`, `history_index`, `history_label(i)`, `history_jump(i)`, `history_add_snapshot(label)`
- [x] 2.3 Bridge: `layer_blend(i)` / `set_layer_blend(i, key)`, `layer_opacity(i)` / `set_layer_opacity(i, v)`, `set_layer_name(i, name)`, `move_layer(i, delta)`
- [x] 2.4 Bridge: `layer_thumbnail(i, size) -> QImage` (null for group/adjustment)
- [x] 2.5 Label every existing history capture site; setters capture history, mark dirty, recomposite
- [x] 2.6 `cargo test -p pictura_app`, clippy clean

## 3. Wave 1b: Navigator, Color/Swatches, Info/Histogram panels (sub-agent, C++, existing APIs)

- [x] 3.1 `panels/navigator_panel.{h,cpp}` — thumbnail, proxy rectangle, zoom slider (0.01–32), Fit, 100%
- [x] 3.2 `panels/color_panel.{h,cpp}` — `ColorState` plus Color panel (RGB/HSB sliders, hex, spectrum)
- [x] 3.3 `panels/swatches_panel.{h,cpp}` — default swatch grid setting the foreground
- [x] 3.4 `panels/info_panel.{h,cpp}` — cursor position, colour under cursor, selection size, document dimensions
- [x] 3.5 `panels/histogram_panel.{h,cpp}` — 256-bin RGBA/luminance histogram computed from the composite image
- [x] 3.6 Syntax-check each new `.cpp`

## 4. Wave 2: Layers and History panels (sub-agent, C++, uses Wave 1a bridge)

- [x] 4.1 `panels/layers_panel.{h,cpp}` — `QTreeView` + model: visibility, thumbnail, name, blend, opacity; selection; Add Adjustment/Delete/Move Up/Down
- [x] 4.2 `panels/history_panel.{h,cpp}` — labeled states, current highlight, jump, snapshot capture/list
- [x] 4.3 Syntax-check each new `.cpp`

## 5. Wave 3: integration (sub-agent)

- [x] 5.1 `frame.{h,cpp}`: replace the debug dock with the Layers panel; register Navigator, History, Color, Swatches, Info, Histogram docks with stable `objectName`s
- [x] 5.2 Bind panels on active-document/tab change and on document change; set cursor position from `ImageView::mouseMoved`
- [x] 5.3 Own a `ColorState`; route the Eyedropper sample to its foreground
- [x] 5.4 `commands.h`/`command_tree.cpp`: mark `Window > Panels` entries implemented (Layers/History/Navigator/Color/Swatches/Info/Histogram) and toggle them
- [x] 5.5 `CMakeLists.txt`: add the panel sources
- [x] 5.6 Build green and M16–M19 self-tests still exit 0

## 6. Self-test and close-out

- [x] 6.1 Extend `--self-test` (exit codes from 50): layer count/name/blend/opacity round-trip and dirty; history label + jump; panel docks exist and toggle
- [x] 6.2 `xvfb-run` self-tests exit 0 (fixture and no-argument)
- [x] 6.3 `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
- [x] 6.4 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [x] 6.5 Archive the change and commit
