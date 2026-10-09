# Tasks

## 1. Row-spec table and dispatcher

- [x] 1.1 Add a `RowSpec` table (`id`, `label`, `KindMask`, `implemented`) and a
  `KindMask kindMaskFor(const QString&)` helper in `layers_panel_menu.cpp`;
  verify it compiles.
- [x] 1.2 Add `LayersPanel::performRowAction(const QString& id, const QString&
  path)` dispatching to existing `view_` ops and panel helpers, mirroring
  `performPanelMenuAction`; verify each currently-wired row maps to it.
- [x] 1.3 Rewrite `populateRowMenu` to walk the table for the row's kind, keep
  Rename/Color Label/Export layout, and render `implemented == false` rows
  disabled with the `— not implemented yet` tooltip; verify the C++ builds.

## 2. Tests

- [x] 2.1 Extend `rowMenuTextsForTest` to accept a kind and assert the per-kind
  row sets (pixel vs group vs type vs adjustment) in
  `crates/pictura-app/cpp/panels/layers_panel_test.cpp`; verify via the Qt Test
  suite.
- [x] 2.2 Add a self-contained check that an unimplemented applicable row is
  present and disabled with the tooltip; verify via the Qt Test suite.

## 3. Verification

- [x] 3.1 Run `ctest --test-dir build -R '^tst_layers_panel' --output-on-failure`
  and `bash scripts/verify-fast.sh`; both green.
