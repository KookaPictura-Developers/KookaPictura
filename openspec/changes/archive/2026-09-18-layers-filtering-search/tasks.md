## 1. Filter proxy

- [x] 1.1 New `crates/pictura-app/cpp/panels/layers_filter_proxy.{h,cpp}`: `struct LayerFilter` (D2) and `LayersFilterProxyModel : QSortFilterProxyModel` with `setFilter`/`filter`; `filterAcceptsRow` accepts a self-match or any matching descendant (ancestor promotion); re-filters on source `modelReset`/`dataChanged`.

## 2. Filter bar

- [x] 2.1 New `crates/pictura-app/cpp/panels/layers_filter_bar.{h,cpp}`: dimension `QComboBox` (Name/Kind/Effect/Mode/Attribute/Color, default Kind), `QStackedWidget` criteria (Name field, Kind toggles, Effect disabled, Mode/Attribute/Color menus), checkable on/off `QToolButton`; `filter()`/`setFilter()` and `filterChanged(LayerFilter)`.

## 3. Panel integration

- [x] 3.1 Insert the filter bar above the blend/opacity header; keep `model_` as the source and set `tree_->setModel(proxy_)`; route `currentPath`/`selectedPaths`/`selectPaths`/rename re-selection and the expanded/collapsed handlers through `proxyIndexForPath`/`pathForProxyIndex` (`layers_panel.{h,cpp}`).
- [x] 3.2 On `filterChanged`: apply the filter, auto-expand group ancestors of matches while active, and restore the session expansion set when off; reset the filter to Kind/off in `setView` (document switch) (`layers_panel.cpp`).
- [x] 3.3 Theme QSS for the filter bar; register the two new TUs and the new self-test TU in `CMakeLists.txt`.

## 4. Self-tests

- [x] 4.1 New `crates/pictura-app/cpp/selftest_layers_filter.{h,cpp}` (so `selftest.cpp` stays at its 6730 allowance): `lfs_name`, `lfs_kind`, `lfs_mode`, `lfs_color`, `lfs_none`, `lfs_ancestor`, `lfs_toggle` (no history), `lfs_live` (rename), `lfs_reset` (document switch) — next free exit codes after 200.

## 5. Verification

- [x] 5.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`.
- [x] 5.2 `cmake --build build --parallel`; `./build/pictura --headless --self-test` and the `two_layers.psd` run both exit 0 with no FAILs.
- [x] 5.3 `TASK_ALLOWS_DOCS=1 bash scripts/verify-full.sh` and `openspec validate --all --strict`.
