# Tasks

## 1. Engine layer-mask operations

- [x] 1.1 Add `crates/pictura-render/src/document_ops/layer_ops/layer_masks.rs`
  with `LayerMaskKind`, a documented `MASK_FLAG_LINKED` bit constant, and
  `add_layer_mask`, `delete_layer_mask`, `apply_layer_mask`,
  `set_layer_mask_enabled`, `set_layer_mask_linked`, `has_layer_mask`,
  `layer_mask_linked`; wire the module and its re-exports through
  `layer_ops/mod.rs`, `document_ops/mod.rs`, and the crate root. Verify with
  `cargo build -p pictura-render`.
- [x] 1.2 Add in-module `#[cfg(test)]` unit tests, one per operation, asserting
  the observable result (mask geometry/coverage, composite change, mask cleared,
  flag round-trip, smart-object refusal). Verify with
  `cargo nextest run -p pictura-render layer_mask`.

## 2. Qt bridge

- [x] 2.1 Add `crates/pictura-app/src/cxxqt_object/layer_masks.rs` with a
  `#[cxx_qt::bridge]` module exposing `layer_mask_add/delete/apply/
  set_enabled/set_linked` and the `layer_mask_present/linked` reads over the
  active layer, mirroring `clipping.rs`; register the module in
  `cxxqt_object.rs` and add the file to the cxx-qt bridge list in `build.rs`.
  Verify with `cargo check -p pictura-app`.

## 3. Integration verification

- [x] 3.1 Run `cargo nextest run -p pictura-render -p pictura-app`,
  `cargo clippy -p pictura-render -p pictura-app --all-targets -- -D warnings`,
  `cargo fmt --all --check`, and
  `openspec validate layer-mask-model-and-ops --strict`; all green.
