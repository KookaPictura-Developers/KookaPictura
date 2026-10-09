# Tasks

## 1. Engine vector-mask operations

- [x] 1.1 Add `crates/pictura-render/src/document_ops/layer_ops/vector_masks.rs`
  with `VectorMaskKind`, the `VECTOR_MASK_FLAG_*` constants (invert `0x01`,
  not-linked `0x02`, disabled `0x04`), and `add_vector_mask`,
  `delete_vector_mask`, `set_vector_mask_enabled`, `set_vector_mask_linked`,
  `rasterize_vector_mask`, `has_vector_mask`, `vector_mask_linked`,
  `vector_mask_disabled`; patch the flag word in place for enable/link; wire the
  module and its re-exports through `layer_ops/mod.rs`, `document_ops/mod.rs`,
  and the crate root. Verify with `cargo build -p pictura-render`.
- [x] 1.2 Add in-module `#[cfg(test)]` unit tests, one per operation, asserting
  the observable result: created block geometry, composite change, delete,
  flag round-trip preserving path bytes, rasterize merge/disabled, and the
  refusals (duplicate, absent work path, unknown path, no mask).

## 2. Qt bridge

- [x] 2.1 Add `vector_mask_thumbnail_image` to
  `cxxqt_object/helpers_composite.rs`, rasterizing `VectorMask::inside` plus
  `invert` into the same white-shows / black-hides square as the raster mask.
- [x] 2.2 Add `crates/pictura-app/src/cxxqt_object/impl_layers/vector_masks.rs`
  with a `#[cxx_qt::bridge]` module exposing
  `vector_mask_add/delete/set_enabled/set_linked/rasterize`, the
  `vector_mask_present/linked/disabled` reads, and the per-row
  `layer_row_has_vector_mask`, `layer_row_vector_mask_linked`,
  `layer_row_vector_mask_disabled`, `layer_row_vector_mask_thumbnail` reads over
  the active layer; register the module in `impl_layers.rs` and add the file to
  the cxx-qt bridge list in `build.rs`. Verify with `cargo check -p pictura-app`.

## 3. Layer menu bar

- [x] 3.1 Add frozen `command_ids` for the eight `Layer ▸ Vector Mask` leaves
  and `Layer ▸ Rasterize ▸ Vector Mask`.
- [x] 3.2 Register them implemented in `command_tree.cpp` (replace the `leaf`
  stubs).
- [x] 3.3 Add a `wireVectorMaskActions()` method (declared in `frame.h`, called
  from `frame_menus.cpp`) implemented in `frame_menus_layer_ops.cpp`: handlers
  and enablement providers for every leaf.

## 4. Layers panel row indicators

- [x] 4.1 Add `HasVectorMaskRole`, `VectorMaskThumbnailRole`,
  `VectorMaskLinkedRole`, `VectorMaskDisabledRole` to
  `layers_panel_model.h` and populate them from the bridge in `layers_panel.cpp`.
- [x] 4.2 Draw the vector-mask thumbnail with its link glyph and disabled red
  cross in `layers_panel_internal.h`, with `vectorMaskThumbRect` /
  `vectorLinkGlyphRect` geometry and an updated `nameRect` reservation.
- [x] 4.3 Wire the vector link-glyph click and vector-thumbnail `Shift`-click in
  `layers_panel.cpp`, and consume them in the double-click guard.
- [x] 4.4 Add the corresponding test seams to `layers_panel.h` /
  `layers_panel_test.cpp`.

## 5. Properties vector mask section

- [x] 5.1 Add a `vectorMaskSection_` to `properties_panel.{h,cpp}`, shown when
  the active layer has a vector mask, with the name row and Enable/Disable,
  Link/Unlink, Delete, and Rasterize actions wired to the bridge.
- [x] 5.2 Add the disabled Density/Feather rows with the
  `— not implemented yet` tooltip and a `vectorMaskSectionVisibleForTest` hook.

## 6. Tests

- [x] 6.1 Qt Test in `tst_command_tree.cpp`: the vector-mask menu leaves' initial
  enablement, Reveal All add, Delete, and Rasterize.
- [x] 6.2 Qt Test in `tst_layers_panel.cpp`: row vector-mask indicators — linked
  and disabled reads, link-glyph click, and thumbnail `Shift`-click.
- [x] 6.3 Qt Test in `tst_properties_panel.cpp`: the Vector Mask section's
  presence, Delete, and Rasterize.

## 7. Verification

- [x] 7.1 `cargo nextest run -p pictura-render -p pictura_app -p pictura-codec`
- [x] 7.2 `cargo clippy -p pictura-render -p pictura_app -p pictura-codec --all-targets -- -D warnings`
- [x] 7.3 `cargo fmt --all --check`
- [x] 7.4 `cmake --build build --parallel`
- [x] 7.5 `ctest --test-dir build -R '^tst_layers_panel$|^tst_command_tree$|^tst_properties_panel$' --output-on-failure`
- [x] 7.6 `bash scripts/verify-fast.sh`
- [x] 7.7 `openspec validate --all --strict`, then
  `openspec archive vector-mask-authoring -y`.
