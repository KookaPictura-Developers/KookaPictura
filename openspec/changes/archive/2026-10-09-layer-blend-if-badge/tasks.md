# Tasks: layer-blend-if-badge

## 1. Bridge

- [x] 1.1 In `crates/pictura-app/src/cxxqt_object/layer_style.rs`, add the
  `layer_row_has_blend_if(view, i)` read to the existing bridge, backed by a
  `layer_is_blend_if_customised(layer)` helper: `Some(view)` -> `!view.is_default()`,
  `None` -> `!layer.blending_ranges.is_empty()`.
- [x] 1.2 Add the `layer_style_set_blend_if(view, path, source_black, source_white)`
  setter that writes `layer.blend_if` (recomposite + bump content revision,
  no history) and restores the default when the range is full.
- [x] 1.3 Rust unit test: default, all-full view, customised view, and a raw
  malformed `blending_ranges` block.

## 2. Row projection

- [x] 2.1 Add `bool hasBlendIf = false;` to `LayerRow` and `HasBlendIfRole` to
  the `LayerRole` enum, with its `data()` case, in `layers_panel_model.h`.
- [x] 2.2 Populate `row.hasBlendIf = layer_row_has_blend_if(*view_, i);` in
  `LayersPanel::refresh`.

## 3. Delegate chip

- [x] 3.1 Add a `Blend If` text chip (fixed bold font, width from its metrics)
  and `blendIfRect`; reserve its advance in `badgesRight` and `nameRect`.
- [x] 3.2 Paint the chip in the right-edge badge run left of fx/lock, leaving
  the fx badge behavior unchanged.

## 4. Tests and gates

- [x] 4.1 Add `rowHasBlendIfForTest` / `rowBlendIfRectForTest` accessors.
- [x] 4.2 Qt Test in `tst_layers_panel.cpp`: an un-customised row reports no
  badge and paints no chip; after `layer_style_set_blend_if(..., 0, 40000)` the
  row reports the badge and paints the chip.
- [x] 4.3 Run `cargo nextest run -p pictura-render -p pictura_app`,
  `cargo clippy -p pictura-render -p pictura_app --all-targets -- -D warnings`,
  `cargo fmt --all --check`, `cmake --build build --parallel`,
  `ctest --test-dir build -R '^tst_layers_panel$|^tst_command_tree$|^tst_layer_style$' --output-on-failure`,
  `bash scripts/verify-fast.sh`, `openspec validate --all --strict`.
- [x] 4.4 Archive the change with `openspec archive layer-blend-if-badge -y`.
