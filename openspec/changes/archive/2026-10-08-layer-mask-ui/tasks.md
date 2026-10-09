# Tasks

## 1. Layers action strip

- [x] 1.1 Enable `layersStripMask` in `layers_panel.cpp`: a click adds
  `reveal-selection` when `has_selection()` else `reveal-all`; `Alt`-click adds
  `hide-all`; keep the CS6 tooltip. Include `layer_masks.cxxqt.h`.
- [x] 1.2 Drop `mask` from `stripDisabled` in `selftest.cpp` and update the
  comment; keep the link/fx assertions and the icon check.

## 2. Row context menu

- [x] 2.1 Mark `addLayerMask`, `deleteLayerMask`, `disableLayerMask` implemented
  in `kRowSpecs` and add an `enableLayerMask` row (no dynamic label: the bridge
  has no `enabled` read).
- [x] 2.2 Dispatch the four ids in `performRowAction` through the bridge.
- [x] 2.3 Update `tst_command_tree.cpp`'s expected pixel row menu and the
  enablement assertions; add a mask add/delete check.

## 3. Layer menu bar

- [x] 3.1 Add frozen `command_ids` for the eleven `Layer Mask` leaves.
- [x] 3.2 Register them implemented in `command_tree.cpp` (replace the `leaf`
  stubs).
- [x] 3.3 Add handlers and enablement providers in `frame_menus.cpp`.

## 4. Properties mask section

- [x] 4.1 Add a `maskSection_` to `properties_panel.h`/`.cpp`, shown when the
  active layer has a mask, with the name row and Enable/Disable, Link/Unlink,
  Delete, and Apply actions wired to the bridge.
- [x] 4.2 Add the disabled Density/Feather/Invert rows with the
  `— not implemented yet` tooltip.
- [x] 4.3 Add a Qt Test in `tst_properties_panel.cpp` covering the section's
  presence and its Delete action.

## 5. Row mask indicators

- [x] 5.1 Add the per-row mask `linked` and `disabled` model roles and populate
  them from the bridge: `layer_mask_disabled` in the engine plus
  `layer_row_mask_disabled` / `layer_row_mask_linked` per-row reads.
- [x] 5.2 Draw the link glyph between the layer and mask thumbnails, the red
  cross over a disabled mask, and wire the link-glyph click and mask-thumbnail
  `Shift`-click.

## 6. Verification

- [x] 6.1 `cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld && cmake --build build --parallel`
- [x] 6.2 `ctest --test-dir build -R '^tst_layers_panel$|^tst_command_tree$|^tst_properties_panel$' --output-on-failure`
- [x] 6.3 `bash scripts/verify-fast.sh`
- [x] 6.4 `openspec validate layer-mask-ui --strict`
