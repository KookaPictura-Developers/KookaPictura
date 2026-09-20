## 1. Batch 1 — Correctness bugs

- [x] 1.1 Ghost canvas: call `refresh()` once at the end of the
  `PicturaMainWindow` constructor (`crates/pictura-app/cpp/frame.cpp`) so the
  empty-pane rule also applies before the first document exists; keep `refresh()`
  the single writer of `tabs_` visibility. Check `lss_fresh_pane` (380).
- [x] 1.2 Lasso combine mode: in `LassoToolHandler::onPress`
  (`crates/pictura-app/cpp/tool_selection.cpp`) name the `mods` argument and call
  `ctx.setDragMode(ctx.resolveSelectionMode(mods, v->has_selection()))` before
  `begin_lasso`. Check `lst_lasso_modes` (381).
- [x] 1.3 Drag cursor retention: in `ToolController::refreshCursor`
  (`crates/pictura-app/cpp/tools_marquee.cpp`) evaluate the press-locked
  selection drag branch before the hover `cursor.moveSelection` branch. Check
  `lst_drag_cursor_retention` (382).
- [x] 1.4 Zero-offset Alt duplicate: an Alt press with no movement now arms the
  duplicate and only creates it on the first non-zero move, so no layer and no
  history state is left behind (`crates/pictura-app/cpp/tool_move.cpp`). Check
  `tc_alt_clone_noop` (384).
- [x] 1.5 Alt selection duplicate live preview: the duplicated pixels follow the
  pointer during the drag, reusing the Move preview; commit records one
  `"Move Selection"` state and leaves the copy active; cancel is bit-identical
  (`tools_selection_move.cpp`, `impl_selection.rs`,
  `impl_transform/session.rs`). Checks `tc_content_duplicate_preview` (385) and
  `tc_content_duplicate_cancel` (386).
- [x] 1.6 Cursor refresh on visibility: `bindCanvas` re-applies `refreshCursor()`
  on the unchanged-canvas path and the `regionBlitted` route refreshes it too
  (`tools.cpp`, `frame.cpp`). Check `vis_cursor_refresh` (383).
- [x] 1.7 Batch 1 regression: Rust tests extended; C++ checks 380-386 added,
  append-only and unique.

## 2. Batch 2 — Layers row cosmetics

- [x] 2.1 Eye gutter and separator: the eye is centred in its gutter with equal
  padding and a 1 px darker-grey separator is drawn at the gutter's right edge
  (`layers_panel_internal.h`). Check `lpr_eye_gutter` (387).
- [x] 2.2 Thumbnail left spacing: the chevron slot is reserved only for
  expandable rows via one shared content-x helper; a regular thumbnail sits
  closer to the gutter. Check `lpr_thumb_gap` (388).
- [x] 2.3 Canvas-aspect thumbnail: the thumbnail is letterboxed to the document
  aspect ratio via new `DocumentWidthRole`/`DocumentHeightRole`; checkerboard,
  outline, brackets, and hit rect follow (`layers_panel_internal.h`,
  `layers_panel.cpp`). Check `lpr_thumb_aspect` (389).
- [x] 2.4 Styled drop indicator: a self-drawn thin blue sibling line / group
  outline replaces the stock primitive, gated by the drop validator. Check
  `lpr_drop_indicator` (390).
- [x] 2.5 Row height: `kRowHeightFloor` raised to 32 and `rowHeight()` to
  `max(32, thumb+12)` (Medium → 36). Check `lpr_row_height_raised` (391).
- [x] 2.6 Batch 2 regression: all existing `lpr_*`/`lpc_*` still pass with the
  moved geometry; new codes only.

## 3. Verification and gates

- [x] 3.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings` — clean.
- [x] 3.2 `cargo nextest run --workspace` and `cargo test --workspace --doc` —
  1261 passed, 9 skipped; doctests 1 skipped.
- [x] 3.3 `cmake --build build --parallel` — OK.
- [x] 3.4 `./build/pictura --headless --self-test` — **326 passed, 0 failed,
  0 skipped**; codes 380-391, none reused; `selftest.cpp` unchanged.
- [x] 3.5 `bash scripts/verify-full.sh` — **TOTAL 1625 passed, 10 skipped,
  0 failed**; `verify-full: OK`.
- [x] 3.6 `openspec validate polish-app-ui-followups --strict` and
  `openspec validate --all --strict` — valid.
- [x] 3.7 No `docs/` change.

## 4. Explicitly not done / ceilings

- [x] 4.1 No large-image paint-smoothness work beyond the already-shipped live
  render and region compositor; the remaining ceilings stay documented.
- [x] 4.2 No structural history-snapshot change for the full-canvas visibility
  toggle latency.
- [x] 4.3 No change to the spec-required Alt pre-press pivot semantics.
- [x] 4.4 No new dependency, no document-format change, no `docs/` edit.
