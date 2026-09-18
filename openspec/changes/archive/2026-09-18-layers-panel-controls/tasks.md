## 1. Engine — nesting lock

- [x] 1.1 `LockFlags::NESTING = 0x08`, `all()` = `0x0F`, `is_all()` tests `0x0F`, and widen the `lock_flags_bits_contains_with_and_all` unit test (`crates/pictura-core/src/lib.rs`).
- [x] 1.2 `lock_from_bits` reads `& 0x0F` and sets the nesting flag; the `lspf` reader mask widens to `& 0x0F` (`crates/pictura-codec/src/read.rs`); the writer is unchanged (`layer.lock.bits()`).
- [x] 1.3 `lock_bit("nesting")` maps to `LockFlags::NESTING` (`crates/pictura-app/src/cxxqt_object/helpers.rs`).
- [x] 1.4 `group_paths` and `ungroup_paths` refuse when the selection contains a nesting-locked node; within-container `move_path` is unaffected (`crates/pictura-render/src/document_ops/layer_ops/properties.rs`).
- [x] 1.5 Widen the bridge/panel full-lock mask from `0x07` to `0x0F` (`impl_layers.rs`, `layers_panel.cpp`).
- [x] 1.6 Unit tests: nesting bit round-trips through the codec; Group/Ungroup refused; Move Up/Down allowed.

## 2. UI — percent controls

- [x] 2.1 New `crates/pictura-app/cpp/panels/percent_field.{h,cpp}`: `QLineEdit` + popup `QSlider` + horizontal-drag scrub; `value()`/`setValue(int pct)`/`valueChanged(int pct)`; emits only for user input.
- [x] 2.2 Replace the Opacity and Fill `QSpinBox`es with `PercentField`; convert `pct↔byte` at the panel boundary (`layers_panel.cpp`).
- [x] 2.3 Theme QSS for `PercentField` (`theme.cpp`); register the new TU (`CMakeLists.txt`).

## 3. UI — five-lock strip and clip indicator

- [x] 3.1 Replace the four text lock toggles with five icon toggles (alpha, paint, position, nesting, all); wire `nesting` to `set_layers_lock`; update `syncControls` to the five flags and the `0x0F` full-lock mask (`layers_panel.{h,cpp}`).
- [x] 3.2 Add independent-creation SVGs `layers.lock{Alpha,Paint,Position,Nesting,All}.svg` and `layers.clipMask.svg`; regenerate `assets/pictura.qrc`.
- [x] 3.3 Draw the clipping-mask glyph left of the thumbnail when `ClippingRole` is set (`layers_panel_internal.h`).

## 4. Self-tests

- [x] 4.1 `lpc_percent` (exit code 199): 50 %↔128 both ways, slider + label drag apply, no feedback on sync.
- [x] 4.2 `lpc_nesting` (exit code 200): nesting bit set/read, `all` four bits, Group/Ungroup refused, Move Up/Down allowed.

## 5. Verification

- [x] 5.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`.
- [x] 5.2 `cmake --build build --parallel`; `./build/pictura --headless --self-test` and the `two_layers.psd` run both exit 0 with no FAILs.
- [x] 5.3 `TASK_ALLOWS_DOCS=1 bash scripts/verify-fast.sh` and `openspec validate --all --strict`.
