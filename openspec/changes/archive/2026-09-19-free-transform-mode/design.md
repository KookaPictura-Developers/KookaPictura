## Context

Place lands a layer at native size and the command tree has `Edit > Free
Transform` as a disabled leaf (`crates/pictura-app/cpp/command_tree.cpp:127`).
The app already owns a live-preview seam: the Move tool caches a base composite
(the target layer hidden) plus the target's layer image and draws them under the
pan/zoom transform (`ImageView::beginMovePreview`, `moveBase_`/`moveLayer_`),
driven by `PictureView` state (`move_base`, `move_layer`, `move_x/y`). That
overlay composites nothing during a drag, which is exactly what a transform
session needs.

The engine has document-scope orientation ops (`rotate_document`,
`flip_document`) and rect shifts (`translate_layer`/`translate_layer_rect`), but
no layer resample that combines scale and arbitrary rotation. `pictura_ops`
already provides exact remaps, `rotate_arbitrary` (bilinear inverse map,
bounding-box growth), and `resize` (bilinear/bicubic), so the transform op can
reuse the same inverse-map pattern without new resampling infrastructure.

Layer geometry is `Layer.rect: PsdRect` (top/left/bottom/right) over planar
`channels` (ids `0..N` plus `-1` alpha) and an optional
`LayerMask { rect, data }`. A placeholder Move-preview cache and a target-index
preview already exist; the transform session generalizes the same cache rather
than adding a second one.

## Goals / Non-Goals

**Goals:**

- One engine op that resamples a layer's channels and mask under a
  similarity transform and updates its rect, with a total refusal contract.
- A modal, per-`PictureView` session that previews via the Move overlay (no
  per-move recomposite), commits exactly one history state, and cancels
  bit-identically.
- Auto-enter the session after any successful Place (menu dialog or canvas drop),
  with cancel not rolling back the placement.
- Wire `Edit > Free Transform` (`Ctrl+T`) with a stable id and an active-layer
  enable predicate. No new dependency.

**Non-Goals:**

- Skew/distort/perspective/warp; numeric options bar; Transform Again;
  non-destructive Smart Object `Trnf`; multi-layer transforms; non-8-bit.
- Reworking the Move tool beyond sharing the preview overlay.

## Decisions

### D1. Engine op: `transform_layer(doc, path, LayerTransform) -> bool`

```rust
pub struct LayerTransform {
    pub scale_x: f64,
    pub scale_y: f64,
    pub angle_radians: f64,
    pub dx: f64,
    pub dy: f64,
}
pub fn transform_layer(doc: &mut Document, path: &str, transform: LayerTransform) -> bool;
```

The op lives in `document_ops/layer_ops/transform.rs` beside the other
`resolve_path`-based layer ops. `doc` is mutated only on success, and the op
**does not recomposite** — like `translate_layer_rect`, the app owns the
composite refresh. This keeps the op's success identical to the state the app
records, and makes the cancel path trivial (the document is never touched until
commit).

**Alternatives considered:** (a) put the math in the app and have Rust only
drive Qt — rejected, the engine has no Qt and the resample belongs with the
other document ops; (b) reuse `rotate_arbitrary` directly — rejected, it rotates
about the *canvas* center and cannot scale, and its centered-on-canvas bounding
box is the wrong frame; (c) a general affine/homography matrix — rejected as
YAGNI for v1 (no skew/perspective).

### D2. Transform math and bounding box

With source rect `r = (left, top, bottom, right)`, `w = right - left`,
`h = bottom - top`, and center `c = ((left+right)/2, (top+bottom)/2)` (document
space, y-down), the forward map of a layer-local point `p` (document coords) is

```
p' = c + R(θ) · (S · (p − c)) + (dx, dy)
```

where `S = diag(scale_x, scale_y)` and `R(θ)` uses the same positive-is-clockwise
screen convention as `pictura_ops::rotate_arbitrary`. The new rect is the
integer bounding box of the four transformed corners:
`left = floor(min x')`, `top = floor(min y')`, `right = ceil(max x')`,
`bottom = ceil(max y')`, with the same `−1e-9` quarter-turn slack
`rotate_arbitrary` uses. The op refuses when the source or result has zero area,
when any parameter is non-finite, or when `|scale_x|`/`|scale_y|` is below a
small epsilon.

Destination pixels are sampled by the inverse map: for a destination pixel
center `q`, `src = c + S⁻¹·R(−θ)·(q − c) − (dx, dy)`, then local
`(src.x − r.left, src.y − r.top)`. Each channel plane is resampled independently
with **bilinear** interpolation; a destination whose source point falls outside
`[0, w) × [0, h)` is written as **0** (transparent for alpha, black for colour),
and bilinear taps are edge-clamped exactly as `rotate_arbitrary` does. The mask,
when present, is transformed by the same document-space matrix about its own
rect, with out-of-source mask samples written as `0` (hidden).

**Ceiling:** bilinear only, no bicubic/nearest option and no perspective. The
`Interpolation` options-bar control and bicubic kernels are deferred; this is
recorded as a `// ponytail:` note in the op.

### D3. Channel-less embedded smart objects are materialized and rasterized

`place_smart_object` (native PSD Place) appends a layer with **no channels**;
`place_image` keeps a raster proxy. To make auto-enter work for both, the op
materializes a channel-less embedded object from its payload with the existing
`render_smart_source(so, rect, rect)` (the same call `rasterize_smart_object`
uses) before resampling, and on success **drops the smart object and its
preserved `SoLd`/`plLd` blocks and linked-source record** — i.e. a transform of
such a layer is destructive and yields a raster layer. This is honest about v1
having no `Trnf` writer and prevents a stale embedded payload surviving a
transform. Rendering is drawn from the proxy, so the commit is self-consistent.

**Alternative considered:** keep the smart object with a transformed proxy and a
stale payload — rejected (Export Contents would emit untransformed bytes).
Refusing channel-less targets was rejected because the task requires auto-enter
after `place_smart_object`.

### D4. Preview reuses the Move overlay with a `QTransform`

The session's live preview is the same cached base (document composite with the
target hidden) plus the target's layer image, painted under a `QTransform`
instead of a translation delta. `begin_free_transform(path)` resolves and caches
base/layer once; the per-move path only edits the session's scale/angle/offset
and repaints. For a channel-less target the overlay layer image is built from
`render_smart_source` **without mutating the document**, so cancel stays
bit-identical. The Move-preview base/layer cache is generalized from
`topmost_pixel_layer_index` to an explicit target index (the Move tool keeps
passing the topmost index), avoiding a second cache.

**Alternative considered:** rebuild `document_to_image` on every mouse move —
rejected (the existing preview exists precisely to avoid that).

### D5. Session state lives in Rust `PictureViewRust`; C++ is input + paint

`PictureViewRust` gains `transform_session: Option<TransformSession>` (target
path, original rect, scale/angle/dx/dy, active handle). Rust owns:
`begin_free_transform(path) -> bool`, `cancel_transform`, `commit_transform`,
`layer_can_free_transform(path)`, the hit-test/gesture API
(`transform_press/move/release` taking document-space image coordinates, a zoom,
and modifier flags), and a `transform_quad()` getter encoded as `"x,y x,y x,y
x,y"` (the `selection_contour` convention). Keeping the math in one place avoids
duplicating geometry between Rust and C++.

C++ owns the widget-facing parts: `ImageView` paints the quad, the 8 handles
(7×7 screen-px squares), and a rotate affordance, and maps gestures to document
coordinates; `ToolController` intercepts canvas press/move/release while a
session is active (normal tools are suspended until commit/cancel) and sets the
cursor. `ImageView` gains Enter/Return (commit) and Escape (cancel) handling in
`keyPressEvent`, emitted as signals and connected by `ToolController`.
`refresh()` runs after commit.

### D6. Handle geometry, hit-testing, and modifiers

The quad is `source_rect`'s four corners under the current transform. Handles:
4 corners + 4 edge midpoints; hit radius **6 screen px**. Dragging inside the
quad but away from a handle moves; a pointer band **20 screen px** outside a
corner rotates about the layer center. Cursors: diagonal/horizontal/vertical
size cursors on handles, `OpenHand`/`ClosedHand` for move, `CrossCursor` in the
rotate band (standard Qt cursors; the SVG cursor set is not required).

- **Shift** during corner scaling locks the aspect ratio (one uniform factor for
  both axes); edge handles stay single-axis.
- **Shift** during rotation snaps the angle to **15°** steps.
- Negative scale factors are allowed (they flip); the minimum is clamping so the
  transformed rect keeps at least 1 px in each axis (identity/zero-scale refused).
- The reference point is the layer center, fixed in v1.

### D7. Session state machine and commit/cancel

```
Idle ──begin_free_transform──▶ Active(identity)
Active ──interactive gesture──▶ Active
Active ──Enter/Return────────▶ commit_transform ─▶ Idle  (one "Free Transform" state)
Active ──Escape──────────────▶ cancel_transform ─▶ Idle  (document bit-identical)
Active ──tool/document change/close─▶ cancel_transform ─▶ Idle
```

A second `begin_free_transform` while active is a no-op; starting a new session
on a different layer first cancels the current one. `commit_transform` refuses
(no state, session cleared) when the engine op refuses, so a degenerate result
never records. Enter and Escape are the only documented exits; a commit on an
identity transform records nothing.

### D8. Place integration

- **Menu Place** (`frame_menus.cpp`): after `place_smart_object`/`place_image`
  returns a non-empty path, call `view->begin_free_transform(path)` then
  `refresh()`. The placement already recorded one `"Place"` state; the session
  adds nothing until commit.
- **Canvas drop** (`file_drop_router.cpp`): after the per-file fan-out, if at
  least one file placed, enter the session on the last successfully placed
  layer. A multi-file drop therefore ends with one active session on the topmost
  placed object; the other objects stay placed untransformed.
- **Cancel** clears only the session overlay. The placed layer and its `"Place"`
  history state remain; the placement is never rolled back.

**No modification to the `file-drop-routing` requirement:** the drop still
places each file and records the same `"Place"` states; the follow-on session is
owned by `free-transform` and is additive.

### D9. Menu wiring

`command_ids::EditFreeTransform = "edit.freeTransform"`; the placeholder leaf in
`command_tree.cpp:127` becomes a real `registry.add(..., Ctrl+T, true)` entry.
The enabled provider is true when the active view has a document, the Layers
panel has a current path, and `layer_can_free_transform(path)` holds. The
handler calls `begin_free_transform(currentPath())`. The `Edit > Transform > …`
leaves remain unregistered and disabled.

**Alternative considered:** put the menu requirement only in `free-transform` —
rejected; `command-registry` is the capability that owns the command table and
already carries a parallel "Layer management commands in the command table"
requirement.

### D10. Verification

- **Rust unit tests** in `transform.rs`: identity is bit-identical (0° / 1.0
  scale / 0 offset); a 2× scale doubles the rect and plane dimensions; a 90°
  turn matches `pictura_ops::rotate90_cw` on a square layer; a 45° turn produces
  the computed bounding box with zeroed corners; out-of-source pixels are zero;
  a mask resamples with the layer; refusals (group, adjustment, Background,
  position-locked, zero-area, zero scale, missing path) leave `doc` bit-identical;
  a channel-less embedded object materializes and rasterizes; a 1×1 layer does
  not panic.
- **Rust session tests** in `impl_transform`/`tests`: begin→gesture→cancel leaves
  the document equal to the pre-begin clone; begin→commit adds exactly one
  `"Free Transform"` state; commit on identity records nothing.
- **C++ self-test** in `selftest_layers_smart_object.{cpp,h}` (exit code **292**,
  the next free after 291): create a document, place a small raster image, assert
  a session became active, drive a scale gesture through the session API, commit,
  and assert exactly one `"Free Transform"` state and a changed layer rect; then
  start a second session, apply a rotation, cancel, and assert the document is
  byte-identical and the history count unchanged; assert a group/adjustment or
  Background target refuses a session. Clean up every document.
- **Gates**: `cargo fmt`, `cargo clippy -D warnings`, `cargo nextest`, the doc
  tests, `bash scripts/verify-full.sh`, and `openspec validate --all --strict`.

## Risks / Trade-offs

- **Preview and commit must agree.** The overlay applies a `QTransform` while
  the commit resamples in Rust; a mismatch would show a different result on
  Enter. → Both derive from the same `LayerTransform` fields, and the C++
  self-test compares the committed rect, not just the preview.
- **Bilinear edge coverage.** Out-of-source → 0 makes edge pixels fade rather
  than clamp; this is a deliberate, stated ceiling. → Documented in the op and
  covered by an out-of-source-zero unit test.
- **Destructive smart-object transform.** Materializing + rasterizing a
  channel-less embedded object surprises users who expect non-destructive Smart
  Object transforms. → Documented; cancel preserves the object, and `Trnf` is an
  explicit non-goal.
- **Preview cache staleness across target switches.** The cache is keyed by
  content revision + target index; entering a session on a different layer must
  invalidate it. → `begin_free_transform` recomputes the cache for the resolved
  target when the key differs.
- **A transform can grow the layer rect beyond the canvas.** → Pixels outside
  the canvas are simply clipped by the compositor, as Place already is; no
  canvas resize is attempted.
- **Exit-code discipline.** Codes are append-only and identify failures. → Take
  292 (291 is the current max), keep the check within the suite's
  `scripts/file-size-allowlist.txt` ceiling.

## Migration

None. The change is additive: no existing command, engine function, requirement,
or persisted state changes. `Edit > Free Transform` moves from disabled
placeholder to implemented; every other `Edit > Transform` leaf is untouched.
