## 1. Engine: move selection content

- [ ] 1.1 Add `crates/pictura-render/src/document_ops/layer_ops/move_content.rs` with `move_selection_content(doc, source_path, mask, dx, dy, duplicate) -> bool`: `layer_via_copy` when duplicate else `layer_via_cut`, translate the new layer's `rect`/`mask` by `(dx, dy)`, and for non-duplicate `merge_scope(MergeScope::Down(&new_path))`.
- [ ] 1.2 Re-export it from the `layer_ops` module and `pictura_render`'s crate surface alongside `layer_via_copy`/`layer_via_cut`.
- [ ] 1.3 Rust tests in the new module: non-duplicate clears the origin and leaves the moved pixels at the destination with the same layer count; duplicate leaves the source intact and adds one layer; a zero-size/no-coverage mask moves nothing meaningfully.

## 2. Bridge + controller declaration

- [ ] 2.1 Add `move_selection_content(self, dx: i32, dy: i32, duplicate: bool) -> bool` to `impl_selection.rs`: use `selection_move_origin` (falling back to the current selection) as the mask, resolve the topmost pixel-layer path, call the engine, clear the origin, recomposite, and `record("Move Selection")`; refuse (`false`, no state) on `dx == dy == 0`, missing doc/selection, or engine refusal.
- [ ] 2.2 Declare the `#[qinvokable]` in `cxxqt_object.rs` and trim duplicated doc comments so the file stays under the 1000-LOC cap.
- [ ] 2.3 Add a Rust test for the bridge guard (zero delta / no selection records nothing) if it can run without a GUI; otherwise cover it in the C++ self-test (task 5.3).

## 3. ImageView: tracking, solid preview, size readout

- [ ] 3.1 Enable `setMouseTracking(true)` so hover moves are delivered without a button.
- [ ] 3.2 Extend `setSelectionPreview(loops, closed, solid)` (default `solid = false`) and paint a solid two-pass polyline (no dash offset) when `solid` is set; keep marching ants otherwise. Add `selectionPreviewSolidForTest()`.
- [ ] 3.3 Add `setDragSizeHint(text, imagePos)` / `clearDragSizeHint()` and paint a small rounded tooltip near the mapped cursor (clamped to the widget); add `hasDragSizeHintForTest()`.
- [ ] 3.4 Add a self-test asserting mouse tracking is enabled and the solid/size-hint flags round-trip.

## 4. ToolController: modifiers, geometry, polygonal, content move

- [ ] 4.1 Add `selectionModeForModifiers(base, mods, hasExisting)` and use it in `handlePressed` to set `dragMode_`; replace every `selectionModeString(mode_)` commit/apply with `dragMode_` for the marquee, ellipse, lasso, polygonal lasso, wand, and quick selection.
- [ ] 4.2 Apply marquee geometry in `marqueeDragRect` from live modifiers (`queryKeyboardModifiers()` in `handleMoved`/`updateMarqueeOverlay`): Alt centres on the press point, Shift squares.
- [ ] 4.3 Push the `W × H` readout through `ImageView::setDragSizeHint` while a marquee/ellipse drag is active and clear it on release.
- [ ] 4.4 Render the polygonal preview as `polygonPoints_ + cursor`, open and solid, on both press and move.
- [ ] 4.5 Split `maybeBeginSelectionMove`: Ctrl inside a selection begins a content-move drag (`duplicate = Alt`); plain (no Shift/Alt) keeps the outline move; Shift/Alt falls through to a combine marquee. The Move tool with a selection begins a content move regardless of press point.
- [ ] 4.6 On release of a content-move drag call `move_selection_content(dx, dy, duplicate)` (once, when `dx`/`dy` are non-zero) and emit `selectionCommitted`; clear the origin on cancel. Reuse the existing outline preview during the drag.
- [ ] 4.7 Make `refreshCursor` show the move cursor when Ctrl is held with a selection tool, and wire Ctrl into the frame's key press/release `refreshCursor()` calls.

## 5. Verification

- [ ] 5.1 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -p pictura-render -p pictura-app`.
- [ ] 5.2 Add C++ self-tests in `selftest_tools_selection.cpp` (next free codes): modifier→mode mapping (Shift/Alt/both/none, with and without an existing selection), Shift-square and Alt-centre geometry, polygonal preview point count includes the cursor, and the content-move commit path.
- [ ] 5.3 Add a self-test that the content move refuses (history unchanged) with no selection / zero delta.
- [ ] 5.4 `TASK_ALLOWS_DOCS=1 bash scripts/verify-full.sh` and a headless self-test run; record the counts.
- [ ] 5.5 Visual pass under Xvfb: quick-mode add/subtract, square/centre marquee + size readout, polygonal solid rubber band, hover move cursor, Move-tool pixel move and Alt duplicate.

## 6. Docs

- [ ] 6.1 Update `docs/dev/STATE.md` with the new behavior, test counts, and the content-move ceilings.
- [ ] 6.2 Commit the change (no `docs/` edits beyond `STATE.md`, which carries `TASK-ALLOWS-DOCS`).
