# Tasks

## 1. Engine: matting ops

- [x] 1.1 Add `crates/pictura-render/src/document_ops/matting.rs` with
  `MatteBackground` (`Black`/`White`), `MattingError` (`Locked`/`Empty`),
  `remove_matte(layer, background, mask)`, and `defringe(layer, width, mask)`;
  export from `document_ops/mod.rs` and the crate root. Verify:
  `cargo nextest run -p pictura-render matting`.
- [x] 1.2 Add engine unit tests: black/white un-matte maths (including clamp and
  alpha-zero), transparent pixels untouched, locked and empty refusals, defringe
  replaces a one-pixel edge band with the interior colour, larger width reaches
  deeper, mask coverage confines the write. Verify:
  `cargo nextest run -p pictura-render`.

## 2. Bridge

- [x] 2.1 Add `crates/pictura-app/src/cxxqt_object/impl_layers/matting.rs` with
  free bridge functions `layer_remove_black_matte`, `layer_remove_white_matte`,
  `layer_defringe(view, width)`, and `matting_target_ready(view)`; each mutation
  recomposites and records one state, honouring the selection mask. Register
  `mod matting;` in `impl_layers.rs` and the file in `build.rs`. Verify:
  `cargo nextest run -p pictura_app`.
- [x] 2.2 Add bridge tests over a document with a semi-transparent edge to prove
  one undo state per command and refusal on a locked layer.

## 3. C++ Layer menu wiring

- [x] 3.1 Add frozen ids to `cpp/commands.h`: `LayerMattingDefringe`,
  `LayerMattingRemoveBlack`, `LayerMattingRemoveWhite`.
- [x] 3.2 Replace the three greyed `Layer > Matting` leaves in
  `cpp/command_tree.cpp` with `registry.add(command_ids::…)` rows.
- [x] 3.3 Add `cpp/frame_menus_matting.cpp` with a `wireMattingActions()`
  method: handlers and enabled-providers for the three ids (Defringe opens
  `DefringeDialog`). Declare it in `cpp/frame.h`, call it from
  `cpp/frame_menus.cpp`, and add the file to `CMakeLists.txt`.
- [x] 3.4 Add `cpp/defringe_dialog.{h,cpp}` — a `Width` spin box (default 1) and
  OK / Cancel — and add both files to `CMakeLists.txt`. Verify:
  `cmake --build build --parallel`.

## 4. Qt Test + verification

- [x] 4.1 Add a Qt Test case to `crates/pictura-app/cpp/tests/tst_command_tree.cpp`:
  the three Matting leaves exist, enable over a pixel layer, and Defringe's
  dialog exposes a Width field defaulting to 1. Add a dialog test to
  `tst_layers_panel.cpp` or the same case. Verify:
  `ctest --test-dir build -R '^tst_command_tree$|^tst_layers_panel$' --output-on-failure`.
- [x] 4.2 Run the full gate set: `cargo nextest run -p pictura-render -p
  pictura_app`; `cargo clippy -p pictura-render -p pictura_app --all-targets --
  -D warnings`; `cargo fmt --all --check`; `cmake --build build --parallel`;
  `bash scripts/verify-fast.sh`; `openspec validate --all --strict`.
- [x] 4.3 Archive with `openspec archive layer-matting-commands -y` and
  re-validate all specs.
