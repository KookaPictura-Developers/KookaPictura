## 1. Engine op

- [x] 1.1 Add `crates/pictura-render/src/document_ops/layer_ops/transform.rs` with `LayerTransform { scale_x, scale_y, angle_radians, dx, dy }` and `pub fn transform_layer(doc: &mut Document, path: &str, transform: LayerTransform) -> bool`; re-export from `layer_ops`, `document_ops`, and `lib.rs`.
- [x] 1.2 Compute the integer result bbox from the four transformed corners (`floor`/`ceil`, quarter-turn `-1e-9` slack) and update `layer.rect`; refuse zero-area sources/results and non-finite or sub-epsilon scale factors with no mutation.
- [x] 1.3 Resample every channel plane with inverse-mapped bilinear sampling (edge-clamped taps, out-of-source → 0) into a fresh plane sized to the result rect; keep the planar layout and channel ids.
- [x] 1.4 Transform `layer.mask` by the same document-space matrix about its own rect; resample its data with out-of-source → 0 and update its rect.
- [x] 1.5 Refuse (no mutation) for a missing path, a group, an adjustment layer, a Background layer, and a position-locked layer; do not recomposite (the caller owns the refresh).
- [x] 1.6 Materialize a channel-less embedded smart object from its payload via `render_smart_source` before resampling, and on success drop `smart_object`, its `SoLd`/`plLd` preserved blocks, and its linked-source record; refuse an undecodable channel-less target.
- [x] 1.7 Mark the bilinear-only / no-perspective ceiling with a `// ponytail:` note naming the upgrade path.

## 2. Engine tests

- [x] 2.1 Unit tests in `transform.rs`: identity is bit-identical; `scale = 2` doubles rect and plane dimensions; a 90° turn on a square layer matches `pictura_ops::rotate90_cw`; a 45° turn yields the computed bbox with zeroed corners; out-of-source pixels are all-zero.
- [x] 2.2 Unit tests: a masked layer's mask follows the layer rect and resamples; `doc` is bit-identical after each refused case (group, adjustment, Background, position-locked, zero-area, zero scale, missing path); a 1×1 layer does not panic.
- [x] 2.3 Unit test: a channel-less embedded smart object materializes, transforms, and ends with `smart_object == None`; an undecodable channel-less target is refused untouched.

## 3. Transform session (Rust)

- [x] 3.1 Add `transform_session: Option<TransformSession>` to `PictureViewRust` (target path, original rect, `scale_x`/`scale_y`/`angle`/`dx`/`dy`, active handle).
- [x] 3.2 Implement `begin_free_transform(path) -> bool` (resolve, validate transformable, cache the base/layer preview for the target, set identity) and `cancel_transform()` (clear only the session).
- [x] 3.3 Implement `commit_transform() -> bool`: call `transform_layer` once, `recomposite`, `record("Free Transform")`, clear the session; record nothing on refusal or identity.
- [x] 3.4 Implement `layer_can_free_transform(path) -> bool` (raster, or a channel-less embedded object with a decodable payload; false for group/adjustment/Background/position-locked).
- [x] 3.5 Generalize the base/layer preview cache from `topmost_pixel_layer_index` to an explicit target index; leave the Move tool passing the topmost index.
- [x] 3.6 Implement the hit-test/gesture API (`transform_press`/`transform_move`/`transform_release`, taking document-space image coordinates, zoom, and modifier flags) with 6 screen-px handle tolerance, 20 screen-px rotate band, Shift aspect lock, Shift 15° snap, and 1-px minimum clamping.
- [x] 3.7 Expose `transform_quad()` encoded as `"x,y x,y x,y x,y"` (the `selection_contour` convention) and the session-active/getter probes used by the self-test.
- [x] 3.8 Session tests: begin→gesture→cancel leaves the document equal to the pre-begin clone; begin→gesture→commit adds exactly one `"Free Transform"` state; commit on identity records nothing.

## 4. Transform session (C++/Qt)

- [x] 4.1 Extend `ImageView` with a transform overlay: draw the `transform_quad()` with a bounding line, the 8 handles as 7×7 screen-px squares, and a rotate affordance; add `beginTransformPreview(base, layer, QTransform)` / `setTransformPreview(const QTransform&)` (or generalize the Move-preview members) so the paint path draws the layer image under the transform without a recomposite.
- [x] 4.2 Add Enter/Return and Escape handling to `ImageView::keyPressEvent`, emitted as signals; route canvas press/move/release to the session and set the size/move/cross cursor in `ToolController`, suspending normal tools while a session is active.
- [x] 4.3 On commit call `refresh()` and clear the overlay; on cancel clear the overlay only; cancel the session on tool switch, document switch, or document close.

## 5. Place integration

- [x] 5.1 In `frame_menus.cpp` `File > Place…`, after `place_smart_object`/`place_image` returns a non-empty path, call `view->begin_free_transform(path)` before `refresh()`.
- [x] 5.2 In `file_drop_router.cpp`, after a canvas drop fan-out with at least one successful place, enter the session on the last placed layer.
- [x] 5.3 Confirm cancel leaves the placed layer where it landed and keeps its `"Place"` state; do not roll back the placement.

## 6. Menu wiring

- [x] 6.1 Add `command_ids::EditFreeTransform = "edit.freeTransform"` to `commands.h` and replace the placeholder leaf at `command_tree.cpp:127` with a real `registry.add(..., QStringLiteral("Ctrl+T"), true)` entry.
- [x] 6.2 Register the handler (`view->begin_free_transform(layersPanel_->currentPath())`) and the enabled provider (document open, current path non-empty, `layer_can_free_transform`), evaluated on menu open.
- [x] 6.3 Leave the `Edit > Transform > …` leaves unregistered and disabled.

## 7. C++ self-test

- [x] 7.1 Extend `runFreeTransformChecks` in `crates/pictura-app/cpp/selftest_layers_smart_object.{cpp,h}` (called from `selftest_layers_controls.cpp`; no new test file) and register the call.
- [x] 7.2 Create a document, place a small raster image, and assert a session becomes active on the returned layer path.
- [x] 7.3 Drive a scale gesture through the session API, commit, and assert exactly one `"Free Transform"` history state and a changed layer rect.
- [x] 7.4 Start a second session, apply a rotation, cancel, and assert the document is byte-identical and the history count is unchanged.
- [x] 7.5 Assert a group, an adjustment layer, and a Background layer each refuse both `layer_can_free_transform` and a session begin.
- [x] 7.6 Use the next free exit code **292** (291 is the current max) with `ST_BEGIN`/`ST_PASS`/`ST_SKIP`/`ST_FAIL`/`ST_FINISH`, keep the file within its `scripts/file-size-allowlist.txt` ceiling, and clean up every document and temp file the check creates.

## 8. Regression and gates

- [x] 8.1 Confirm the Move-tool preview and its self-tests still pass with the generalized cache.
- [x] 8.2 `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 8.3 `cargo nextest run --workspace` and `cargo test --workspace --doc`.
- [x] 8.4 `bash scripts/verify-full.sh`; build with CMake and run `./build/pictura --headless --self-test`.
- [x] 8.5 `openspec validate free-transform-mode --strict` and `openspec validate --all --strict`.
