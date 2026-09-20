## 1. Batch 1 — Persistence, tab name, Background lock, scrollbars, numeric (items 1, 2, 3, 4, 16)

- [ ] 1.1 Item 1: in `crates/pictura-app/cpp/session.{h,cpp}` advance the schema to **8** and add a `railMode` to each per-column entry; in `crates/pictura-app/cpp/frame_session.cpp` save each column's actual mode (around `:63-64`), load a missing per-column `railMode` from the legacy top-level `panelRailMode` (`:79-88`), and restore each column's own mode instead of stamping the primary column's value on every column (`:210-215`).
- [ ] 1.2 Item 1 regression: add `crates/pictura-app/cpp/selftest_ui_persistence.cpp` (register in `CMakeLists.txt`, invoke from `runSelfTest()`, first free failure code **324**) asserting a non-primary column toggled `iconic`→`normal` round-trips, a v7 store seeds every column from the global, and a v6 store still loads the default width. Do not grow `selftest.cpp` (6729 LOC, ceiling 6730).
- [ ] 1.3 Item 2: in `crates/pictura-app/cpp/frame.cpp` change `displayName` from `QFileInfo(path).baseName()` to `fileName()` (`:424,445`) and update the `documentName` preference (`:271-272`) so `updateTabTitle` yields `image01.jpg (RGB/8)`; keep the `Untitled-N` fallback and the dirty marker.
- [ ] 1.4 Item 2/3 regression: extend the tab check to assert the extension is present, and add a Rust unit test in `crates/pictura-app/src/cxxqt_object/tests.rs` for the import post-process.
- [ ] 1.5 Item 3: in `crates/pictura-app/src/cxxqt_object/impl_core.rs:25` change `finalize_import` from `LockFlags::all()` to `LockFlags::TRANSPARENCY | LockFlags::POSITION` (`0x05`); leave `background = true` and the dropped `-1` channel unchanged.
- [ ] 1.6 Item 3 regression: update the `lim_opaque_background` check (code 290) to assert only the transparency and position bits, and add assertions that the background flag is set and the `-1` channel is absent.
- [ ] 1.7 Item 4: in `crates/pictura-app/cpp/canvas_scrollbars.cpp` remove the content-vs-viewport visibility computation and the visibility-settling two-pass loop (`:93-103`); keep both bars always visible and keep the shared range projection. Add a `ponytail:` note that visibility is fixed on.
- [ ] 1.8 Item 4 regression: in `selftest_ui_persistence.cpp` replace the "bars hide when the document fits" assertion with one that both bars are visible when the document fits and that a pan within the shared range still moves the canvas.
- [ ] 1.9 Item 16(a): in `crates/pictura-app/cpp/options_bar.cpp` give the `buildPaintPage::addField` Hardness, Opacity, and Flow controls a `%` suffix instead of the hardcoded empty suffix (`:338`).
- [ ] 1.10 Item 16(b): pass `popup=false` for the Feather control (`:216-220`) so no slider popup opens.
- [ ] 1.11 Item 16(c): change the `px` suffix literal from `" px"` to `"px"` (`:218`); do not touch `PercentField`'s `"%"`.
- [ ] 1.12 Item 16 regression: add a check in `selftest_ui_persistence.cpp` (or `selftest_numeric.cpp` if closer to its cap) that the paint fields render `%`, Feather opens no popup, and a pixel field renders `12px` with no space.
- [ ] 1.13 Batch 1 gate: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace`, `bash scripts/verify-full.sh`, `./build/pictura --headless --self-test`; commit.

## 2. Batch 2 — Layers panel drag, Background convert, thumbnail selection, nesting (items 5, 6, 7, 8)

- [ ] 2.1 Item 5: in `crates/pictura-app/cpp/panels/layers_panel_internal.h` implement `LayersModel::mimeTypes()`, `canDropMimeData()`, and `supportedDropActions()` so `QAbstractItemViewPrivate::canDrop()` is true and `QTreeView::dragMoveEvent` computes a real `dropIndicatorPosition`; in `crates/pictura-app/cpp/layers_panel.cpp` call `setState(QAbstractItemView::DraggingState)` in `dragEnterEvent` (`:423-430`) so the indicator paints. Confirm `dropTargetFor` (`:480-495`) now resolves above/below reorder modes instead of always mode 2.
- [ ] 2.2 Item 5 regression: add `crates/pictura-app/cpp/selftest_layers_interactions.cpp` (register, codes **328+**) that synthesizes a drag over a sibling gap and asserts the drop indicator is above/below and one reorder history step, and that an invalid target (own descendant / Background) shows no indicator and changes nothing.
- [ ] 2.3 Item 6 double-click: in `crates/pictura-app/cpp/layers_panel.cpp` route a double-click outside the name on the Background row to the existing `layer_from_background` conversion instead of `openLayerStyle` (`:630-653`); keep the no-op affordance for other layers.
- [ ] 2.4 Item 6 drop: in the New Layer button drop handler (`:291,585-592`) branch on the dragged row: a Background converts in place via `layer_from_background`, every other row duplicates as today.
- [ ] 2.5 Item 6 regression: add checks that a Background double-click and a Background drop on New Layer each clear the flag and unlock in one undo step with no `Background copy`.
- [ ] 2.6 Item 7: add a `thumbnailRect(path)` hit-test to `LayerRowDelegate` (`crates/pictura-app/cpp/panels/layers_panel_internal.h`, alongside `eyeRect`/`lockRect`/`chevronRect`) and, in the viewport event filter, handle a `Ctrl`+left click inside it by calling a new bridge op. Implement the op in `crates/pictura-app/src/cxxqt_object/impl_selection.rs` reusing the private `apply_selection` path (`:302,363`) and `pictura_select::Selection::from_channel` (`crates/pictura-select/src/lib.rs:175`): build a document-sized selection from the layer's `-1` alpha (missing ⇒ `255`, outside `layer.rect` ⇒ `0`) and combine as New. Consume the click so it starts no drag and opens no editor.
- [ ] 2.7 Item 7 regression: add a Rust unit test for the alpha-to-selection builder (offset, missing channel, outside-rect) and a check that a thumbnail `Ctrl`-click changes the selection and starts no drag.
- [ ] 2.8 Item 8: in `crates/pictura-render/src/document_ops/layer_ops/properties.rs` check `NESTING` on the destination group in `move_path_to_dest` (`:413`) and on the source parent in `group_paths` (`:268`) and `ungroup_paths` (`:314`), so a drop into or out of a nesting-locked group is refused while a within-container reorder stays allowed.
- [ ] 2.9 Item 8 regression: add Rust unit tests for the destination and source-parent refusals and the allowed reorder, and a panel check that a nesting-locked group refuses a reparent drop.
- [ ] 2.10 Batch 2 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`, headless self-test; commit.

## 3. Batch 3 — Active-layer gating and transparency lock (items 9, 10)

- [ ] 3.1 Item 9 resolver: add one shared active-layer resolver in `crates/pictura-app/src/cxxqt_object/` returning the exactly-one active layer path or a typed refusal; have the Layers panel push its selection to the view on selection change, and have `open_image` set the imported layer active (`impl_core.rs`).
- [ ] 3.2 Item 9 entry points: replace the topmost-raster fallback with the resolver in paint (`crates/pictura-app/src/cxxqt_object/impl_paint.rs`, `crates/pictura-paint/src/helpers.rs:593`, `stroke.rs:45-46`), filters (`impl_filters.rs:55`), Free Transform (`impl_transform.rs:757`), and content move (`impl_selection.rs:143`); zero or multiple selected layers refuse with a user-visible error and no history state.
- [ ] 3.3 Item 9 regression: add Rust tests for the resolver (exactly one, zero, multiple, non-raster) and `selftest_layer_locks.cpp` checks that a tool edit targets the panel's active layer and is refused with no single selection.
- [ ] 3.4 Item 10: remove the blanket `TRANSPARENCY` refusal at `crates/pictura-paint/src/stroke.rs:52` and `crates/pictura-render/src/document_ops/.../move_content.rs:35`; make `composite_pixel`/`write_pixel` (`stroke.rs:196-231,310-326`) carry the pre-existing alpha and write it back for pixels whose alpha is greater than zero; leave fully transparent pixels untouched.
- [ ] 3.5 Item 10 filter: apply the same per-pixel rule in the filter path so `filter.rs` preserves each pixel's alpha; keep a `Clear` paint mode / erase refused because it lowers alpha.
- [ ] 3.6 Item 10 regression: add Rust tests that a transparency-locked paint and filter change color where alpha `>0`, keep alpha bit-identical (e.g. `180` stays `180`), and leave alpha `0` untouched; add a `selftest_layer_locks.cpp` check for the same through the bridge.
- [ ] 3.7 Batch 3 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`, headless self-test; commit.

## 4. Batch 4 — Cursor, ring outside, Space pan, hint bar (items 11, 12, 14, 15)

- [ ] 4.1 Item 11: in `crates/pictura-app/cpp/tools.cpp:565-568` set `Qt::BlankCursor` for Brush/Pencil instead of the `tool.brush`/`tool.pencil` cursor assets; keep the drawn ring and restore the normal cursor when the tool changes or the pointer leaves.
- [ ] 4.2 Item 12: in `crates/pictura-app/cpp/image_view.cpp` scope the document clip in `paintEvent` with `save()`/`restore()` around the image rect (`:507`) so the ring (`:547`) draws with the full-widget clip while keeping the existing image-space transform.
- [ ] 4.3 Item 14: add the Space transient pan: handle key press/release in `image_view.cpp`/`frame.cpp`, set `Qt::OpenHand` and enable pan while held, pan on drag through the shared range helper, and restore the previous cursor/pan on release. Rework the `QKeySequence(Qt::Key_Space, Qt::Key_F)` shortcut at `frame.cpp:152` so Space is not consumed as a sequence prefix.
- [ ] 4.4 Item 15: replace the flat `hintLabel_` in `crates/pictura-app/cpp/frame_build.cpp:326-358` with a hint-bar widget of bordered keycaps plus descriptions, fed by `updateToolHint` (`frame.cpp:924-935`) from `tools_->activeTool()`. Source command keys from `CommandRegistry::action(id)->shortcut()` and tool letters from `toolShortcutKeys()` (`tools.cpp:225-235`). Include `Shift` = Add to selection and `Alt` = Subtract for selection tools, and highlight a keycap while its key is held. Fall back to the text hint for a tool with no keycaps.
- [ ] 4.5 Item 11/12/14/15 regression: add `crates/pictura-app/cpp/selftest_tool_canvas.cpp` (register, codes **337+**) for the blank paint cursor, the ring drawn outside the document clip, a Space pan that leaves the active tool unchanged, and the hint bar's context and pressed-key highlight.
- [ ] 4.6 Batch 4 gate: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace`, `bash scripts/verify-full.sh`, headless self-test; commit.

## 5. Batch 5 — Brush performance (item 13)

- [ ] 5.1 Item 13 dirty: in `crates/pictura-paint/src/stroke.rs:139-141` make `Stroke::dirty()` return and clear the per-dab dirty rectangle instead of accumulating the whole stroke; assert the reported rect covers the latest dab only.
- [ ] 5.2 Item 13 region compositor: in `crates/pictura-render/src/gpu/mod.rs:185-205` make `composite_cpu_region` composite only the requested region rather than full-compositing then slicing; preserve the existing composite result within tolerance.
- [ ] 5.3 Item 13 present cache: in `crates/pictura-app/cpp/image_view.cpp:148` update only the region of the scaled present image instead of invalidating the whole cache per dab.
- [ ] 5.4 Item 13 regression: add a Rust region-compositor test comparing a region composite against the full composite, and `crates/pictura-app/cpp/selftest_paint_latency.cpp` (register, codes **342+**) asserting per-dab dirty is incremental; measure a 4000² dab against the canvas-view budget (≤16 ms input-to-first-pixel, ≥60 FPS sustained) and record the result.
- [ ] 5.5 Batch 5 gate: fmt/clippy/nextest/doctests, `scripts/verify-full.sh`, headless self-test; commit.

## 6. Verification and gates

- [ ] 6.1 `cargo fmt --all` (CI runs `--check`), `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 6.2 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [ ] 6.3 `bash scripts/verify-full.sh` and confirm every `scripts/file-size-allowlist.txt` ceiling holds; each new `selftest_*.cpp` stays under the code cap and `selftest.cpp` stays at 6729 LOC.
- [ ] 6.4 `./build/pictura --headless --self-test` and record the passed/failed/skipped counts; confirm new codes start at 324 and no existing code was reused.
- [ ] 6.5 `openspec validate fix-app-ui-interactions --strict` and `openspec validate --all --strict`; both must report valid.
- [ ] 6.6 No `docs/` change in this change; if one is needed it goes in a separate `TASK-ALLOWS-DOCS` commit.

## 7. Explicitly not done (ceilings)

- [ ] 7.1 No artboards or frames are added; item 8's nesting rule applies to groups only, as the proposal states.
- [ ] 7.2 No Layer Style dialog is built; the double-click affordance stays a documented no-op for non-Background layers.
- [ ] 7.3 Zoom/offset (and therefore scrollbar position) persistence remains out of scope; only the per-column rail mode is added at schema v8.
- [ ] 7.4 The hint-bar keycap table is limited to the documented context per tool (selection `Shift`/`Alt` plus command shortcuts); a complete Adobe shortcut transcription is not attempted.
- [ ] 7.5 No new crate, dependency, or `docs/` change; `selftest.cpp` does not grow.
