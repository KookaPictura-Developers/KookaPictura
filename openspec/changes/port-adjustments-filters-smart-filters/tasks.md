# Tasks

## 1. Engine-ready menu wiring

- [ ] 1.1 Add the fifteen `Filter ▸ Artistic` leaves in `crates/pictura-app/cpp/command_tree.cpp` (CS6 order, before `Blur`) and the matching `def(...)` rows in `crates/pictura-app/cpp/filter_commands.cpp` with slot orders/defaults copied from `filter_map.rs`. Verify `ctest --test-dir build -R '^tst_filter_menu$'` passes and every row's kind resolves.
- [ ] 1.2 Extend `adjustment_layer` in `crates/pictura-app/src/cxxqt_object/helpers.rs` to accept `levels`, `curves`, `exposure`, `vibrance`, and `black-white` via `pictura_render::default_adjustment_block`, and add the five `adjustment:*` rows to the Adjustments panel menu in `crates/pictura-app/cpp/panels/panel_group_menu.cpp`. Verify with a Rust test that all sixteen kinds return a layer and an unknown kind returns `None`.
- [ ] 1.3 Keep `cxxqt_object.rs` within its allowlist ceiling while adding any `mod` line, and run `bash scripts/check-file-size.sh`. Verify it reports OK.

## 2. Render and Stylize filter kernels

- [ ] 2.1 Port Lighting Effects into `crates/pictura-filters/src/render.rs` (or a submodule) following the crate's planar-buffer/alpha contract; add the `Filter` variant, apply dispatch, and lib re-export. Verify with a Rust test that it relights colour, preserves alpha, distinguishes light types, and rejects bad parameters untouched.
- [ ] 2.2 Port Diffuse and Glowing Edges into `crates/pictura-filters/src/stylize.rs` with the `DiffuseMode` enum and the Glowing Edges constants; add the `Filter` variants, dispatch, and a clamp-to-edge dilate helper. Verify with Rust tests for determinism, alpha preservation, and out-of-range refusal.
- [ ] 2.3 Add `FILTER_ARITIES` rows and `filter_from_kind_params` arms in `filter_map.rs` and the `filter_commands.cpp` rows; keep `filter_map.rs` under the file-size cap by moving its inline tests to `filter_map_tests.rs`. Update `tst_filter_menu.cpp` so Glowing Edges is no longer a disabled stub and the three new dialogs collect their slots. Verify `cargo nextest run -p pictura-filters -p pictura_app` and `ctest -R '^tst_filter_menu$'` pass.

## 3. Adjustments: Shadows/Highlights and Color Lookup

- [ ] 3.1 Add `ShadowsHighlightsParams` and `Adjustment::ShadowsHighlights` to `pictura-adjust` (8-bit `tonal` kernel plus a native arm), with the luminance-delta table, neutrality, alpha preservation, and parameter validation. Verify with a Rust test (`crates/pictura-adjust/tests/shadows_highlights.rs`) and the native-depth parity list.
- [ ] 3.2 Add the `shdH` decode/encode, `default_adjustment_block` arm, `ADJUSTMENT_DIALOG_KINDS` entry, and the two-slider `layout` arm; add the `kDialogs` row in `frame_menus_adjust.cpp`. Verify the dialog opens on `0/0` and `tst_image_adjustments` passes.
- [ ] 3.3 Add the seven Color Lookup presets and `preset_cube` (`crates/pictura-render/src/color_lookup_presets.rs`), `set_color_lookup_preset`, the dialog default/`layout` Choice, and the `"preset"` special case in `set_adjustment_param`; add the `kDialogs` row. Verify `None` is the identity, each preset builds a valid 16-grid cube, and the Qt preset-rebuild case passes.

## 4. HDR Toning

- [ ] 4.1 Add `HdrToningParams` and the Local Adaptation kernel in `crates/pictura-filters/src/hdr_toning.rs` with parameter validation, alpha preservation, and determinism; add the `Filter` variant and dispatch. Verify with a Rust test.
- [ ] 4.2 Add the `image_hdr_toning` bridge (preview/apply) reusing the filter preview/commit core, and the preview-apron arm in `pictura-render`. Verify wrong-length/invalid parameters are refused and preview does not record history.
- [ ] 4.3 Add `hdr_toning_dialog.{h,cpp}` with the seventeen presets plus `Custom`, register it in `CMakeLists.txt`, and wire the `Image ▸ Adjustments ▸ HDR Toning` handler in `frame_menus_adjust.cpp`. Verify the presets populate the nine controls and `tst_image_adjustments` passes.

## 5. Replace Color

- [ ] 5.1 Add `ReplaceColorSample`, `ReplaceColorParams`, and the engine (`sigma_sq`, `match_weight`, `replace_color`, `replace_color_mask`) in `crates/pictura-adjust/src/replace_color.rs`; add the `Adjustment` variant, dispatch, and re-exports. Verify with Rust tests for fuzziness 0 exact-match, hue shift, empty-list identity, alpha preservation, and invalid-parameter refusal.
- [ ] 5.2 Add the `image_replace_color` bridge (mask/preview/apply) parsing the `"x,y,r,g,b;…"` sample string and reusing the `ActiveOp` preview/commit path. Verify preview does not record history and cancel is bit-identical.
- [ ] 5.3 Add `replace_color_dialog.{h,cpp}` as a non-modal dialog reusing the `ColorRangeDialog` canvas sampler, register it in `CMakeLists.txt`, and wire the `Image ▸ Adjustments ▸ Replace Color` handler plus `frame.h` state. Verify `tst_replace_color` passes (menu enabled, sample, apply commits one state, cancel restores).

## 6. Smart Filters: model, codec, render

- [ ] 6.1 Add `SmartObject::smart_filters_enabled` and the `filter_mask_*` fields in `pictura-core`, auditing every struct literal. Verify `cargo nextest run -p pictura-core`.
- [ ] 6.2 Parse the `filterFXStyle` group flags in `smart_object.rs` and author them in `author_filter_fx`/`smart_writer.rs`; add the generic `attach_smart_filter` and make `attach_pictura_raw_filter` a wrapper. Verify a `write_psd`/`read_psd` round-trip preserves the flags and the filter fields.
- [ ] 6.3 Add `pictura-render/src/smart_filter.rs` (`decode_smart_filter`, `apply_smart_filter_chain`, group/per-filter gating) and the `composite_layer_inner` branch that applies the chain to the embedded source only when every enabled filter decodes. Verify with render tests that a disabled filter is a no-op, the chain is not double-applied, and an unknown enabled filter falls back to the proxy.

## 7. Smart Filters: tree and Convert

- [ ] 7.1 Add the `layers_smart_filters` file-level bridge (row getters, group/filter visibility setters, `convert_for_smart_filters`) following the `image_hdr_toning` pattern, registered in `build.rs`; keep `cxxqt_object.rs` net-zero. Verify the Rust row-API round-trip tests pass.
- [ ] 7.2 Add the synthetic `Smart Filters` parent and per-filter child rows to the Layers panel with the `synthetic` guards (no rename, no drag, no layer ops) and the eye-toggle branch, putting new logic in a new translation unit registered in `CMakeLists.txt`. Verify `tst_layers_smart_filters` and `tst_layers_panel` pass.
- [ ] 7.3 Add `convert_for_smart_filters` to `pictura-render` and wire the `Filter ▸ Convert for Smart Filters` command id, handler, and enablement in `command_tree.cpp`/`frame_menus.cpp`. Verify converting a raster layer preserves its composite and records one state.

## 8. Integration verification

- [ ] 8.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, and `cargo test --workspace --doc`; verify all green.
- [ ] 8.2 Build with CMake and run `ctest --test-dir build -R '^tst_' --output-on-failure` offscreen, plus `./build/pictura --headless --self-test`; verify the suites and self-test pass with no new self-test checks.
- [ ] 8.3 Run `openspec validate --all --strict` and `bash scripts/verify-fast.sh`; verify both pass.
- [ ] 8.4 Update `docs/dev/STATE.md` with the resume note (commit it with `TASK-ALLOWS-DOCS`); verify `bash scripts/guard.sh` stays green.
