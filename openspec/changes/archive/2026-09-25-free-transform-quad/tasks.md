# Tasks: free-transform-quad

## 1. Engine: projective op

- [x] 1.1 Add a projective `Map` variant (or a second map type) carrying the inverse 3×3 homography, with `forward`/`inverse` using homogeneous coordinates.
- [x] 1.2 Add `pub fn transform_layer_quad(doc: &mut Document, path: &str, quad: [(f64, f64); 4]) -> bool` in `crates/pictura-render/src/document_ops/layer_ops/transform.rs`, solving the source-corners→`quad` homography by an 8×8 linear solve; reuse the existing refusal rules, channel-less smart-object materialization, bounding-box integerization, `MAX_RESULT_PIXELS` guard, and bilinear resample; refuse non-finite corners, a singular/near-singular map, or a zero-area result; do not recomposite.
- [x] 1.3 Factor the shared resolve/materialize/resample/write skeleton so `transform_layer` and `transform_layer_quad` cannot drift; keep the similarity path bit-identical.
- [x] 1.4 Export `transform_layer_quad` from `pictura-render`'s `lib.rs`.

## 2. Engine tests

- [x] 2.1 Identity quad leaves the layer unchanged; a translated quad equals `transform_layer` with the same translation.
- [x] 2.2 A non-affine perspective quad maps the four source corners onto the targets within tolerance.
- [x] 2.3 Degenerate quads (collinear, non-finite, zero-area) return false with `doc` bit-identical; out-of-source destination pixels are all-zero.
- [x] 2.4 A channel-less embedded smart-object target materializes and drops its payload; a group/adjustment/Background/position-locked target is refused.

## 3. App: session modes

- [x] 3.1 Add `TransformMode` (`Free | Skew | Distort | Perspective`) and `quad: Option<[(f64, f64); 4]>` to `TransformSession` in `state.rs`; `Free` sessions set `quad: None`.
- [x] 3.2 `begin_transform_mode(path, mode)` shares `begin_free_transform`'s target resolution and refusal; the three commands use it.
- [x] 3.3 `transform_quad_points` (`geometry.rs`) returns the live `quad` when present; add `gesture_distort`, `gesture_perspective`, `gesture_skew` and make hit-testing mode-aware (handles only in projective modes).
- [x] 3.4 `commit_transform` routes to `transform_layer_quad` for projective modes and `transform_layer` for `Free`; identity detection compares the quad to the source corners; Escape still cancels exactly.

## 4. App: commands

- [x] 4.1 Add `EditTransformSkew`/`EditTransformDistort`/`EditTransformPerspective` ids in `commands.h`; mark the three `command_tree.cpp` leaves implemented and give them the ids.
- [x] 4.2 Add handlers + enabled providers in `frame_menus.cpp` (mirroring `EditFreeTransform`); route through `ToolController`/`frame_menus` to `begin_transform_mode`.

## 5. Verification

- [x] 5.1 App self-test: a Distort, Perspective, and Skew session each commit one `"Free Transform"` state and change the layer rect; Escape in a projective mode restores the document and adds no state.
- [x] 5.2 `cargo nextest run --workspace`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `bash scripts/check-file-size.sh`; `openspec validate free-transform-quad --strict`; CMake build + `./build/pictura --headless --self-test`.
