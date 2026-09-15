## 1. Brief and interface contracts (orchestrator)

- [x] 1.1 Write `docs/dev/m18-toolbox-tools.md` milestone brief
- [x] 1.2 Record the cross-agent contracts in `design.md` (bridge method list, `ImageView` event/overlay surface, `ToolController` surface, command ids)
- [x] 1.3 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Engine: selection shapes, crop, move, sample (workstream E)

- [x] 2.1 `pictura-select`: add `rect`, `ellipse`, and `polygon` coverage rasterizers
- [x] 2.2 `pictura-select`: add `combine(mode)` for New/Add/Subtract/Intersect and Quick Selection (wand union) + unit tests
- [x] 2.3 `pictura-render`: add `crop_document(doc, x, y, w, h)` over a document (reuse M12 canvas offset) + test
- [x] 2.4 `pictura-render`: add `translate_layer(doc, dx, dy)` for the topmost pixel layer + test
- [x] 2.5 Bridge: `select_rect`, `select_ellipse`, `begin_lasso`/`lasso_add_point`/`end_lasso`, `quick_select`, `crop`, `translate_layer`, `sample_argb`, `selection_bounds`
- [x] 2.6 Bridge: dirty/history capture on the new mutating operations
- [x] 2.7 `cargo test -p pictura-select -p pictura-render`, clippy clean

## 3. Canvas event and overlay plumbing (workstream V)

- [x] 3.1 `image_view.{h,cpp}`: emit `pressed/moved/released` with image-space coordinates
- [x] 3.2 `image_view.{h,cpp}`: `setPanEnabled(bool)`, `setOverlayPolygon(QPolygonF)`, `clearOverlay()`, and `widgetToImage`
- [x] 3.3 Preserve wheel zoom and the existing zoom/pan self-test behavior

## 4. Tool framework and tools (workstream T)

- [x] 4.1 `tools.{h,cpp}`: `ToolId`, tool metadata table, `ToolController` (active tool, drag state, options widgets)
- [x] 4.2 `toolbox.{h,cpp}`: Tools dock with a `QActionGroup`, letter shortcuts (`V M L W C I H Z`), active-tool highlight
- [x] 4.3 `options_bar.{h,cpp}`: `QToolBar` with a stacked page per tool (combine modes, tolerance); `Window > Options` toggle
- [x] 4.4 Implement Move, Marquee, Lasso, Quick Selection, Crop, Eyedropper, Hand, Zoom in the controller
- [x] 4.5 `frame.{h,cpp}`: host the toolbox and options bar, connect the active canvas events, `activeTool()`/`setActiveTool()` test hooks, status hint, `foregroundColor()`
- [x] 4.6 `commands.h`/`command_tree.cpp`: tool commands + `Image > Crop`; `View > Options` toggle

## 5. Integration and self-test (orchestrator-led, delegated)

- [x] 5.1 Add the new sources to `CMakeLists.txt`
- [x] 5.2 Extend `--self-test`: active-tool switching and options follow the tool
- [x] 5.3 Extend `--self-test`: marquee rect/ellipse coverage, lasso polygon, Quick Selection, and the combine modes
- [x] 5.4 Extend `--self-test`: crop remaps pixels and resizes; move translates a layer; eyedropper samples a known pixel
- [x] 5.5 Assign self-test exit codes from 39 and keep M16/M17 checks passing

## 6. Verification and close-out

- [x] 6.1 `cmake -S . -B build && cmake --build build`
- [x] 6.2 `xvfb-run -a ./build/pictura --self-test` and the fixture self-test pass
- [x] 6.3 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- [x] 6.4 `openspec validate --all --strict` and `bash scripts/guard.sh`
- [x] 6.5 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [x] 6.6 Archive the change and commit
