## 1. Bridge: preview / commit split

- [x] 1.1 Add `opacity_preview_changed` and `fill_preview_changed` bools to
      `PictureViewRust` (default `false`) in `cxxqt_object.rs`.
- [x] 1.2 Declare `#[qinvokable]` `preview_layers_opacity`,
      `commit_layers_opacity`, `preview_layers_fill`, `commit_layers_fill`.
- [x] 1.3 Implement them in `impl_layers.rs`: preview applies + recomposites +
      bumps `content_revision` without recording; commit applies + records
      `"Opacity"`/`"Fill Opacity"` once when the preview or the apply changed.
- [x] 1.4 `cargo fmt --all` and `cargo clippy -p pictura-app --all-targets -- -D warnings`.

## 2. PercentField: `%` inside and commit signal

- [x] 2.1 Make `suffix_` a child of `edit_`, reserve right text margin, reposition
      on `edit_` resize; keep the `%` a scrub handle.
- [x] 2.2 Add `valueCommitted(int)` and the `pending_`/`commitPending()` logic;
      emit commit on scrub release, `sliderReleased`, `editingFinished`, and
      popup `Hide`.

## 3. Row delegate: eye-only, lock badge

- [x] 3.1 Remove `Qt::CheckStateRole` from `LayersModel::data`/`setData` so no
      checkbox indicator is painted.
- [x] 3.2 Add `LayerRowDelegate::lockRect` and paint the `layers.lockAll` badge
      on the right when `LockRole != 0`.

## 4. Panel wiring and hooks

- [x] 4.1 Connect `valueChanged` → `preview_layers_*` and `valueCommitted` →
      `commit_layers_*` in `layers_panel.cpp`.
- [x] 4.2 Add test hooks (`lockBadgeLeftForTest`, `opacitySuffixInsideEditForTest`,
      `rowCheckStateForTest`) in `layers_panel_test.cpp` + declarations in
      `layers_panel.h`.

## 5. Self-tests

- [x] 5.1 `lpc_preview`: preview several Opacity values (history count unchanged),
      commit once (count +1).
- [x] 5.2 `lpr_percent`: the `%` suffix is a child of the value box and inside it.
- [x] 5.3 `lpc_lockbadge`: a locked row reports a lock badge left of the right
      edge; an unlocked row reports none.
- [x] 5.4 `lpr_eye`: the row reports no check state.
- [x] 5.5 Keep `selftest.cpp` under its size cap; put new checks in
      `selftest_layers_controls.{h,cpp}`.

## 6. Verification

- [x] 6.1 `cmake --build build --parallel` and
      `./build/pictura --headless --self-test` green.
- [x] 6.2 `TASK_ALLOWS_DOCS=1 bash scripts/verify-full.sh` green.
- [x] 6.3 `openspec validate layers-panel-control-polish --strict` green.
