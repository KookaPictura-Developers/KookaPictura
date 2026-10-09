# Tasks

## 1. Codec list editing

- [x] 1.1 Add `delete_smart_filter`, `reorder_smart_filters`, and `clear_smart_filters` to `crates/pictura-codec/src/smart_filter.rs`, re-authoring the preserved `filterFX` and syncing the typed view
- [x] 1.2 Export the new functions from `crates/pictura-codec/src/lib.rs`
- [x] 1.3 Codec tests: reorder/delete/clear round-trip on the fixture and out-of-range refusal

## 2. Engine document operations

- [x] 2.1 Add `crates/pictura-render/src/document_ops/smart_filters.rs` with `add_smart_filter`, `delete_smart_filter`, `reorder_smart_filters`, `clear_smart_filters`
- [x] 2.2 Re-export the operations from `document_ops/mod.rs` and `crates/pictura-render/src/lib.rs`
- [x] 2.3 Engine tests: add/delete/reorder/clear track the typed list; non-smart refusal

## 3. App bridge

- [x] 3.1 Add `delete_layer_smart_filter`, `clear_layer_smart_filters`, and `reorder_layer_smart_filter` to `layers_smart_filters.rs`, each recompositing and recording one state
- [x] 3.2 Bridge tests: edits reach the preserved descriptor and survive a write/re-read

## 4. UI

- [x] 4.1 Register `layer.smartFilters.clear` and add the `Layer > Smart Filter > Clear Smart Filters` command
- [x] 4.2 Wire the handler and enabled provider in `frame_menus.cpp`
- [x] 4.3 Qt Test: Clear Smart Filters enabled with filters, empties the stack, disables after

## 5. Deferred (documented in design.md)

- [ ] 5.1 Decode filter-mask pixels from `FEid`/`FXid`/`FMsk` into `SmartObject.filter_mask`; blocked on a reference PSD and the unresolved block format
- [ ] 5.2 Route Filter-menu filters into the smart stack; blocked on a filter registry (only Camera Raw is renderable)
- [ ] 5.3 Panel drag-reorder, filter-mask thumbnail, and per-filter Blend Options dialog
- [ ] 5.4 Wire the Disable/Delete Filter Mask menu leaves once 5.1 lands

## 6. Verification

- [x] 6.1 `cargo nextest run -p pictura-render -p pictura_app -p pictura-codec`
- [x] 6.2 `cargo clippy -p pictura-render -p pictura_app -p pictura-codec --all-targets -- -D warnings`
- [x] 6.3 `cargo fmt --all --check`
- [x] 6.4 `cmake --build build --parallel`
- [x] 6.5 `ctest --test-dir build -R '^tst_layers_panel$|^tst_command_tree$|^tst_smart' --output-on-failure`
- [x] 6.6 `bash scripts/verify-fast.sh`
- [x] 6.7 `openspec validate --all --strict`
