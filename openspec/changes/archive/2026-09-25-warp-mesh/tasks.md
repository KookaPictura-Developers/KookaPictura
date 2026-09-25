# Tasks: warp-mesh

## 1. Engine

- [x] 1.1 Add `crates/pictura-render/src/document_ops/layer_ops/warp.rs`: `WarpMesh { rows, cols, points }`, `WarpParams { distort_h, distort_v }`, `identity_mesh(cols, rows, w, h)`, `apply_distortion`, the cubic Bézier surface evaluation, and `transform_layer_warp`.
- [x] 1.2 Reuse the shared refusal/materialize/rect/smart-object skeleton from `transform.rs` (do not duplicate the rules; extract a helper or call the same internal path).
- [x] 1.3 Rasterize by subdividing the parameter domain into cells, inverse-mapping each destination pixel barycentrically per cell, and sampling bilinearly with the existing edge-clamp/out-of-source→0 rule; guard the result with `MAX_RESULT_PIXELS`.
- [x] 1.4 Export `transform_layer_warp`, `WarpMesh`, `WarpParams`, `identity_mesh` from `layer_ops/mod.rs`, `document_ops/mod.rs`, and `lib.rs`.

## 2. Tests (`warp.rs` `#[cfg(test)]` or the existing transform tests)

- [x] 2.1 Identity mesh is byte-identical; a translated uniform mesh matches `transform_layer` translation.
- [x] 2.2 Moving one interior control point changes interior pixels and the bounding rect while the four corner pixels stay put.
- [x] 2.3 Non-finite point / `rows<2` / wrong point count refuse with `doc` bit-identical; out-of-source destination pixels are all `0`.
- [x] 2.4 A group / adjustment / Background / position-locked target is refused.

## 3. Verification

- [x] 3.1 `cargo nextest run -p pictura-render`; `cargo nextest run --workspace`.
- [x] 3.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/check-file-size.sh`, `openspec validate warp-mesh --strict`.
