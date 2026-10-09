# Tasks: paint-on-layer-mask

## 1. Engine

- [x] 1.1 `pictura-render` `document_ops/layer_ops/mask_edit.rs`: `MaskDocument`,
  `mask_document`, `write_mask_back` (region-limited, materialising), exported;
  unit tests (round-trip, step coverage, refusals).
- [x] 1.2 No new dependency.

## 2. Bridge

- [x] 2.1 `PictureViewRust` mask edit target + mask stroke context; bridge
  set/clear/read in `impl_layers/layer_masks.rs`.
- [x] 2.2 `begin_paint` / `paint_dab` / `end_paint` / `cancel_paint` route to the
  mask document when the target is active (no GPU/preview).
- [x] 2.3 `edit_fill` routes to the mask with the selection cropped to the mask.
- [x] 2.4 `apply_op_active_region` routes filter preview/commit/cancel to the
  mask coverage.

## 3. C++

- [x] 3.1 Layers panel: plain mask-thumbnail click sets the target, layer
  thumbnail clears it; delegate draws the active-mask border; a row role carries
  the target.
- [x] 3.2 `CMakeLists.txt` unchanged (no new files).

## 4. Verification

- [x] 4.1 Qt Test: activation click + brush/fill write coverage and history;
  layer click clears.
- [x] 4.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cmake --build build --parallel`; `ctest -R '^tst_'`;
  `bash scripts/verify-fast.sh`; `openspec validate --all --strict`.
