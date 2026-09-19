## Why

Phase 1 (`image-import`) and Phase 2 (`file-drop-routing`) made Place accept
raster images and OS drags, but a placed object lands at native size with no way
to size or rotate it. Photoshop's Place is inseparable from an immediate Free
Transform session, and `Edit > Free Transform` (`Ctrl+T`) is still an
unregistered placeholder in the command tree. This phase closes the place/drop
program with an interactive similarity transform on the active layer.

## What Changes

- **New capability `free-transform`**: an engine layer similarity transform and
  an interactive, per-document transform session.
  - **Engine op** `pictura_render::transform_layer(doc, path, LayerTransform)
    -> bool` resamples the layer's planar channels (and mask, if any) with
    bilinear sampling into the transformed bounding rect and updates `rect`.
    `LayerTransform` carries `scale_x`, `scale_y`, `angle_radians`, and
    `(dx, dy)` about the layer's center. It refuses (no mutation) for missing
    paths, groups, adjustment layers, Background layers, position-locked
    layers, zero-area sources, non-finite/zero scale, and an empty result. A
    channel-less embedded smart object is materialized from its payload and
    rasterized on success.
  - **Session**: a modal session holding the target path, the original rect, and
    the live scale/rotation/translation. The canvas draws the transformed
    bounding box with 8 scale handles and a rotate affordance; mouse
    press/move/release manipulate it. The live preview reuses the Move-preview
    overlay (cached base + cached layer image drawn under a `QTransform`), so no
    document recomposite happens per mouse move. Enter/Return commits exactly one
    history state through the engine op; Esc cancels with the document
    bit-identical to before.
  - **Place integration**: after a successful `place_image` / `place_smart_object`
    (menu dialog or canvas drop) the app selects the new layer and enters the
    session. Cancelling leaves the placed layer where it landed, untransformed;
    the prior `"Place"` state is never rolled back.
  - **Menu**: `Edit > Free Transform` (`Ctrl+T`) is registered with the stable id
    `edit.freeTransform` and enabled only when the active layer is transformable.
    The `Edit > Transform > …` leaves stay placeholders.
- **ADDED `command-registry`**: a requirement declaring the `edit.freeTransform`
  command, its `Ctrl+T` shortcut, and its active-layer enablement.
- **MODIFIED `smart-object-layer-actions`**: the `File > Place…` command
  requirement gains the auto-enter clause (select the new layer, start a
  transform session; cancel does not roll back the Place).
- **No new dependencies.**

## Capabilities

### New Capabilities

- `free-transform`: the engine layer similarity transform (math, bilinear
  resampling, coverage and refusal rules, mask handling, channel-less
  smart-object materialization) and the interactive session (state machine,
  bounding-box geometry and hit-testing, modifier behaviour, preview reuse,
  commit/cancel semantics, place integration).

### Modified Capabilities

- `smart-object-layer-actions`: the `File > Place…` command requirement changes
  so a successful place also selects the new layer and starts a Free Transform
  session; the one-undo-state and refusal contracts are unchanged.
- `command-registry`: adds a `Free Transform command in the command table`
  requirement (declaration, shortcut, enablement). This is an ADDED requirement
  to an existing capability, not a change to an existing requirement.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/transform.rs` (new module)
  plus re-exports from `layer_ops`/`document_ops`/`lib.rs`; optional
  `psd`/`rotate` helper reuse from `pictura-ops`.
- `crates/pictura-app/src/cxxqt_object/impl_transform.rs` (session state and
  bridge methods), `state.rs` (session fields), and the base/layer preview cache
  generalized from topmost to a target index.
- `crates/pictura-app/cpp/image_view.{h,cpp}` (transform overlay and preview
  transform, Enter/Esc key handling) and `crates/pictura-app/cpp/tools.{h,cpp}`
  (`ToolController` routes canvas input to the active session).
- `crates/pictura-app/cpp/command_tree.cpp` and `commands.h` (`edit.freeTransform`),
  `crates/pictura-app/cpp/frame_menus.cpp` (handler + enable provider + place
  auto-enter), and `crates/pictura-app/cpp/file_drop_router.cpp` (drop
  auto-enter).
- `crates/pictura-app/cpp/selftest_layers_smart_object.{cpp,h}`: one C++ self-test
  (exit code **292**).
- No Rust dependencies, no `docs/` change, no new smart-object engine
  infrastructure.

## Non-Goals

Deferred deliberately; each is a separate follow-up:

- **Skew, Distort, Perspective, Warp, Puppet Warp, Content-Aware Scale** and the
  non-destructive Smart Object transform (`Trnf`) record.
- The Free Transform **numeric options bar** (reference-point locator, X/Y, W/H,
  angle, Interpolation) and the `Edit > Transform > …` leaves.
- **Transform Again** (`Shift+Ctrl+T`) and repeating the last transform.
- Multi-layer / multi-selection transforms; transforming selection borders.
- Non-RGB / non-8-bit depth resampling and ICC-aware interpolation.
