## Context

The previous change (`fix-app-ui-interactions`) shipped 16 UI fixes in five
batches. Its Batch 5 rewrote the brush present path:

- `crates/pictura-paint/src/stroke.rs`: `Stroke::dirty()` became `take_dirty()`,
  returning only the dabs since the previous call (per-dab dirty rect);
- `crates/pictura-render/src/composite.rs` + `gpu/mod.rs`: a true CPU region
  compositor (`composite_rgba_region`);
- `crates/pictura-app/cpp/image_view.cpp::blitRegion`: instead of invalidating
  the scaled present cache, it patches the region of the scaled image in place.

The last item is the regression. The live path is
`impl_paint.rs::paint_dab` → `impl_core.rs::refresh_region` → the
`region_blitted(QImage, x, y)` signal → `frame.cpp`'s connection →
`ImageView::blitRegion` → `update()` → `paintEvent`. On release
`end_paint` → `recomposite()` → `changed` → `frame.cpp::refresh` →
`replaceImage(view->image())`, which is why the stroke only appears then.

The other 23 items are confirmed root causes from the round-3 recon; this design
records the shared pieces and the test strategy, not the per-item investigation.

Constraints that shape the design:

- `crates/pictura-app/cpp/selftest.cpp` is 6729 LOC at its 6730 allowlist
  ceiling and MUST NOT grow; new C++ checks live in new `selftest_*.cpp` TUs
  registered in `CMakeLists.txt` and invoked from `runSelfTest()` (directly or
  from a batch runner such as `runLayersControlsChecks`).
- Self-test failure codes are append-only; the highest in use is **345** (the
  `pp_present_cache` check), so new checks start at **346**.
- `frame.cpp::refresh()` is the single funnel that pushes `view->image()` into
  the canvas and rebinds tools; the empty-pane rule belongs there.
- `docs/` is the long-form contract and is not edited by this change.
- No new dependency; std/Qt6 only.

## Goals / Non-Goals

**Goals:**

- Make a brush stroke continuously visible from the first dab to release, and
  commit data plus exactly one history state only on release.
- Remove the large-image stutter on the live path without regressing the
  present-cache byte-identity contract.
- Encode every round-3 item as a testable requirement with an independent
  commit batch.
- Build the shared pieces once: the zoom-correct cache patch, one
  `active_layer_visible` helper, one per-row delegate geometry helper, one
  wheel-modifier precedence table, and one captured drag-modifier state.
- Keep the self-test exit-code identity append-only and every `selftest*.cpp`
  inside `scripts/file-size-allowlist.txt`.

**Non-Goals:**

- No resident GPU layer sources, tiles, display-time LoD, or a GPU-resident
  zero-copy present.
- No compositor math, ±1 LSB parity, or PSD/PSB format change.
- No `docs/` edit, no new dependency, no Layer Style dialog.
- No re-investigation of the confirmed root causes; only the regression is
  analysed here.

## Decisions

### D1. Root cause of the live-render regression: the present-cache patch uses the wrong painter transform order

`ImageView::blitRegion` (`crates/pictura-app/cpp/image_view.cpp:150-163`) does:

```cpp
QPainter patch(&presentCache_.scaled);
patch.translate(x, y);       // T(x, y)
patch.scale(zoom_, zoom_);   // then S(zoom)
patch.drawImage(QPointF(0, 0), region);
```

Qt composes transforms as `world = world * new`, so the mapping is `T·S`:
an image point `p` becomes `zoom * p + (x, y)`. The region's top-left
(document `(x, y)`) therefore lands at scaled-cache `(x, y)` instead of
`(x * zoom_, y * zoom_)`. This was verified directly: a 100 px document at
zoom 0.5, a region at document `(40, 40)`, drew its first white pixel at cache
`(40, 40)` under the current order and at `(20, 20)` under the corrected order.

Consequences:

- For a large image the initial view fits, so `zoom_ < 1` and the cache is
  `document * zoom_` wide. Document coordinates `(x, y)` routinely exceed the
  cache dimensions, so `drawImage` is clipped and the dab is simply not drawn.
  Where it is not clipped the dab is drawn at the wrong place. In every case the
  cursor's own pixel never updates, so the stroke "only appears after release".
- Because the patch still sets `presentCache_.key = image_.cacheKey()` and
  `presentCache_.valid` stays true, `paintEvent` keeps reusing the (wrong)
  cache. The visible repaint that makes the stroke appear is release's
  `recomposite()` + `replaceImage()`, which rebuilds the full document and its
  scaled cache — the O(document) work perceived as "laggy on big images".
- The prior self-test `pp_present_cache` missed it because it ran at
  `actualPixels()` (`zoom_ == 1`), where `T·S` and `S·T` coincide.

**Fix.** Compose the patch as scale-then-translate so the region maps to its
scaled destination, and keep the cache patch as the fast path:

```cpp
patch.scale(zoom_, zoom_);   // S(zoom)
patch.translate(x, y);       // then T(x, y)  => S·T: p -> (p + (x, y)) * zoom
patch.drawImage(QPointF(0, 0), region);
```

`scale(zoom)` then `translate(x, y)` yields `S·T`, so the region top-left maps
to `(x * zoom, y * zoom)` and the region itself scales by `zoom`. The
alternative `translate(x * zoom, y * zoom); scale(zoom)` is equivalent; the
scale-first form keeps the integer document coordinates as the single source
(the same values already passed to the full-resolution blit and the signal).
The `else` branch (cache disabled, invalid, or a different zoom) keeps
`presentCache_.valid = false`, so no consumer ever sees a half-patched cache.
The full-resolution `image_` write is unchanged and stays authoritative.

Rejected alternatives: invalidating the whole present cache per dab (the
pre-Batch-5 behaviour — correct but O(canvas) per dab, the stutter we are
fixing); rebuilding the scaled cache from a patched full image per frame (same
cost); a separate scaled-coordinate cache keyed on the patch rect (more state
than the fix needs).

### D2. The commit contract stays one history state on release

`paint_dab` (`impl_paint.rs`) keeps calling `refresh_region` for the stroke's
working document; mid-stroke `refresh_region` deliberately does not patch the
app document's `composite` and does not emit `changed` (M34). `end_paint`
finishes the stroke, stores the resulting document, `recomposite`s once, and
calls `record(&label)` once. The regression fix must not move any write or
`record` into the live path. The regression test asserts exactly one new history
state after a multi-dab stroke and that the companion `undo` restores the
pre-stroke image.

### D3. Per-layer-row delegate geometry (one source for paint and hit-test)

`LayerRowDelegate` (`panels/layers_panel_internal.h`) already exposes
`eyeRect`, `lockRect`, `chevronRect`, `thumbRect`, and `nameRect`. The round-3
items 3 and 6-11 all depend on paint and hit-test agreeing, so the shared rule
is: **`paint()` lays out from these same rects; no second geometry**.

- `nameRect` currently collapses to zero width when `thumb == 0` and omits the
  `+4` gap paint applies after the thumbnail, and it uses `right - 4` rather
  than paint's `right` cap. It is corrected to mirror paint exactly and is
  floored to a minimum width so a click never falls into a zero-width band.
- A content-band click that is not an eye/chevron/thumbnail/lock/fx/mask hit is
  treated as a name click (rename for a normal layer, style no-op otherwise), so
  double-click rename works across the whole label band.
- Row height gets one named constant with a floor of ~28 px, used by both
  `sizeHint` and the delegate's centring math.
- New roles: `LayerRowLinkedRole` (the path is in the frame's `link_sets`) and
  `LayerRowPlacedRole` (the layer's `smart_object` kind is External/Alias).
  Typography derives from them and from the `background` flag: `Background`
  italic/cursive, all other names normal, linked/placed underlined.
- Thumbnails: regular layers draw a cached 2-tone checkerboard behind the
  thumbnail; groups keep their glyph; every thumbnail gets a 1 px black
  outline; the singular active layer gets white 1 px corner brackets drawn one
  pixel outside the outline.
- The color label tints only `eyeRect`'s background (base colour elsewhere) and
  the post-name `labelSwatch` paint is removed while the menu item keeps using
  `labelSwatch`.
- The selected-row highlight is drawn by the style, so the delegate clips that
  paint to the row minus the eye column and repaints the eye column with the
  base colour before drawing the tint, keeping the eye legible.

### D4. `active_layer_visible` and the invisible-layer policy

One helper in `crates/pictura-app/src/cxxqt_object/` resolves whether the
exactly-one active layer (the same resolver the tool edits use) is visible. The
policy is asymmetric on purpose:

- paint and filter edits are refused with a user-visible refusal and the
  Block/Forbidden cursor (an invisible layer has nothing to show, and silently
  editing it is the confusion the item names);
- selection, copy, and Move remain allowed, including the keyboard nudge, so
  the user can still operate on the layer's content;
- `compute_move_preview` (`impl_transform.rs`) currently sets the layer
  `visible = true` to composite the base and must save and restore the prior
  value, so a move does not make an invisible layer visible as a side effect.

The cursor branch lives next to the existing lock-cursor branch in
`ToolController`, so the blank-brush, locked-layer, invisible-layer, and
transient-eyedropper cursors are resolved in one place with one precedence
order (transient eyedropper > session > blank brush > invisible > locked >
tool).

### D5. Wheel-modifier precedence

`ImageView::wheelEvent` today zooms unconditionally on `angleDelta().y()`.
The shared precedence, applied before the zoom math, is:

1. `angleDelta().x() != 0` → pan horizontally by the side-wheel delta;
2. `Ctrl+Alt` + vertical wheel → pan vertically;
3. `Alt` (without Ctrl) + vertical wheel → pan horizontally;
4. otherwise zoom at the cursor, with `Shift` doubling the zoom step.

The zoom anchor stays the wheel position (`zoomAt` already anchors there). The
same cursor anchor is applied to the Zoom-tool click (`tools.cpp`: the click
currently calls centre `zoomIn`/`zoomOut`) and to the Navigator slider, which
both route through `setZoom(zoom, anchor)`.

### D6. Drag-modifier capture for selection geometry

`shape-selection-tools` already locks the combine mode at gesture start, but
the geometry is read live from `QGuiApplication::queryKeyboardModifiers()` in
`updateMarqueeOverlay` and at release. The shared fix is one `dragMods_`
captured at press and used for the preview geometry, the release raster, and
the cursor while dragging, so releasing Shift or Alt mid-drag changes neither
the mode nor the geometry. The combine cursor is gated so the move-selection
cursor is returned only when neither Shift nor Alt is held; Shift/Alt show the
combine cursor. Alt remains geometry-only (from-centre) and never pre-toggles a
mode before the drag starts.

### D7. Live-render fix is the regression gate; new checks start at 346

`selftest.cpp` is frozen. The regression check lives in a new
`selftest_paint_region.cpp` (or an existing paint-perf TU) and must run the
blit at a zoom **other than 1** and assert the canvas shows the dab before
release — the missing coverage that let the bug ship. Additional round-3
checks land in new `selftest_*.cpp` TUs (for example `selftest_layers_round3.cpp`
invoked from `runLayersControlsChecks`) registered in `CMakeLists.txt`; codes
are allocated from **346** upward and never reused.

## Test Strategy

- **Regression (highest priority).** A C++ check drives `begin_paint` + several
  `paint_dab`s at a zoom `< 1` and `> 1`, grabs the canvas before release, and
  asserts the dab pixels are present and in the right place; then asserts
  `end_paint` adds exactly one history state and `undo` restores the pre-stroke
  image. This test fails on the current `translate`-then-`scale` order.
- **Rust unit tests.** The `active_layer_visible` helper (present/absent/hidden
  layer); the `compute_move_preview` save/restore; `begin_move_duplicate`
  (duplicate + recomposite + no `record`, one `commit_move` state); the
  wheel/native-scan-code `[`/`]` mapping helper; the visibility region path
  (`layer_visibility_region` → `refresh_region`, fallback only when a region is
  `None`).
- **C++ self-test checks** (new TUs, codes 346+): empty-pane visibility;
  tab-label styling is asserted through the theme string only where a stable
  token exists (do not touch the line `selftest.cpp` string-matches); rename band
  hit-test; drop-indicator flag and closed-hand cursor; Background conversion
  rename/dialog; eye-only tint and highlight clip; thumbnail checkerboard,
  outline, and active brackets; row typography roles; row-height floor; nesting
  button hidden; wheel modifier precedence; Zoom-click and Navigator cursor
  anchoring; transient eyedropper; invisible-layer paint refusal with the
  Block cursor and a successful Move.
- **Latency.** A `#[ignore]`d Rust profile for the visibility region path on a
  4000² document (print-only, not a flaky wall-clock assertion); the C++ check
  asserts the region path was taken (region blit fired, no full `changed`) and
  the toggle completed under the sub-1s target on the reference run.
- **Gates.** `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo nextest run --workspace`, `cargo test --workspace --doc`,
  `scripts/verify-full.sh`, `./build/pictura --headless --self-test`,
  `openspec validate polish-app-ui --strict`, `openspec validate --all
  --strict`. Every new `selftest*.cpp` stays under its file-size ceiling and
  `selftest.cpp` stays at 6729 LOC.

## Risks / Trade-offs

- **The transformed patch could differ from the full rebuild by a pixel.** →
  The patch uses the same `S(zoom)` factor as `cachedScaled`'s builder; a test
  compares the patched cache against a fresh rebuild at the same zoom, not only
  at 1.
- **A correct per-dab patch still costs a full-resolution `QPainter` write.**
  → Keep the region-sized write; if a profile shows the transform overhead, the
  named upgrade is a scaled-rect blit in the cache's own coordinate space.
- **Renaming the Background could surprise a user who expects the name to
  persist.** → The requirement makes `Layer N` the Photoshop-correct default and
  offers the name in the dialog; the dialog's name is authoritative.
- **Hiding the nesting lock button could read as removing the feature.** → Only
  the button is hidden; the engine `NESTING` refusal rules and the PSD flag stay
  and are covered by the existing tests.
- **The invisible-layer policy is asymmetric (refuse edits, allow move).** →
  That asymmetry is the requirement, not an accident: move has a preview the
  user can see; a paint edit would be invisible.
- **Wheel precedence could change existing Zoom-tool expectations.** → The
  unmodified vertical wheel still zooms; only combinations that were previously
  ignored (side wheel, Alt/Ctrl+Alt) gain pan behaviour.
- **Self-test code collisions with concurrent work.** → Re-verify the highest
  code (345) immediately before allocating; new checks use a dedicated
  `selftest_*.cpp` so `selftest.cpp` cannot grow.

## Migration Plan

Additive and internal except the corrected cache patch. No document format, no
session schema, no dependency change. Batches are independently committable and
reversible. Rollback of the regression fix alone restores the M35 behaviour
(invalidate the cache per dab) at the cost of the stutter; there is no data
migration.

## Open Questions

- Whether the `Background` conversion dialog should be the full New Layer dialog
  or a name+color-only factory — the requirement allows either; the design
  leans to a name+color factory plus `set_layer_name_path` + `set_layers_color`
  after the unlock, which is the smaller path and keeps the unlock's one-state
  contract.
- Whether the row-height floor is expressed in the delegate's `sizeHint` only or
  also in the model, so a themed font change cannot drop it below the floor.
  The design uses the delegate constant and asserts it in a check.
- The exact `Layer N` numbering source: `next_layer_name(doc, "Layer")` is the
  established helper; the open question is only whether a converted Background
  keeps its position when a same-numbered layer already exists (the helper's
  highest-suffix rule answers this; a test pins it).
