# Tasks

## 1. Bridge

- [x] 1.1 Add `layer_row_has_style(view, i)` and `layer_style_effect_names()` to `crates/pictura-app/src/cxxqt_object/layer_style.rs`, next to the existing `layer_style_has`
- [x] 1.2 Re-export/verify the new symbols in the generated layer-style header used by the panel

## 2. Row projections

- [x] 2.1 Add `HasLayerStyleRole` and `StyleEffectsRole` to `LayerRole`, and `hasStyle` / `styleEffects` to `LayerRow` in `layers_panel_model.h`
- [x] 2.2 Return the two new roles from `LayersModel::data`
- [x] 2.3 Populate them in `LayersPanel::refresh` from the bridge

## 3. fx badge and delegate geometry

- [x] 3.1 Add a shared `showsFx(index)` predicate and an `fxRect(itemRect, index)` hit-target to `LayerRowDelegate`
- [x] 3.2 Paint the fx badge for styled layers and keep `badgesRight` / `nameRect` in sync

## 4. Panel behaviour

- [x] 4.1 Implement `openLayerStyle` with `LayerStyleDialog` + `runDialog`, gated on `layer_style_can_edit`
- [x] 4.2 Enable the `layers.fx` strip button: click opens Blending Options, Alt-click toggles all effects
- [x] 4.3 Alt-click the row fx region toggles all effects

## 5. Row menu

- [x] 5.1 Extend `RowSpec` with an enable policy and flip the four style rows to implemented
- [x] 5.2 Evaluate the policy per row in `populateRowMenu`
- [x] 5.3 Dispatch `blendingOptions` / `copyLayerStyle` / `pasteLayerStyle` / `clearLayerStyle` in `performRowAction`

## 6. Effect filter dimension

- [x] 6.1 Add `effect` to `LayerFilter` and match `StyleEffectsRole` in the proxy
- [x] 6.2 Populate the filter bar's Effect combo from `layer_style_effect_names()` and stop disabling it

## 7. Tests

- [x] 7.1 Qt Test in `tst_layers_panel.cpp`: style row menu presence/enablement, fx badge, FX Alt-click toggle, Effect filter dimension
- [x] 7.2 Refresh the `tst_command_tree.cpp` panel-menu expectations for the now-implemented rows
- [x] 7.3 Add the `layers_panel_test.cpp` hooks the tests need

## 8. Deferred (documented in design.md)

- [ ] 8.1 Blend-If badge; blocked on an advanced-blending flag in the layer model
- [ ] 8.2 Style rows for shape / smart-object kinds (kept at the pixel/type mask here)

## 9. Verification

- [x] 9.1 `cargo nextest run -p pictura-render -p pictura_app`
- [x] 9.2 `cargo clippy -p pictura-render -p pictura_app --all-targets -- -D warnings`
- [x] 9.3 `cargo fmt --all --check`
- [x] 9.4 `cmake --build build --parallel`
- [x] 9.5 `ctest --test-dir build -R '^tst_layers_panel$|^tst_command_tree$|^tst_layer_style$|^tst_filter_menu$' --output-on-failure`
- [x] 9.6 `bash scripts/verify-fast.sh`
- [x] 9.7 `openspec validate --all --strict`
