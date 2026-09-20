## Why

An investigation of nineteen reported UI defects (`docs/dev/app-bugs-investigation.md`,
commit `7ee4805`) traced each to a confirmed root cause with `file:line` evidence.
They are small, independent, and mostly code-vs-spec mismatches or a single missing
hook, not missing architecture. Left unfixed they make the shell feel unfinished:
the workspace forgets its layout on a normal quit, every launch fabricates a
512×512 document, opened files impersonate untitled buffers, three lock flags do
nothing, and the canvas cannot be panned predictably. This change lands the
seventeen in-scope fixes as five independently committable, spec-backed batches
and explicitly defers the two items (arrow-key nudge, large-canvas paint latency)
that need decisions or engine work beyond an app-UI change.

## What Changes

- **Persistence (issue 1).** The quit path (`File > Exit`, `Ctrl+Q`, window close)
  always saves; a column's width is stored per column in the session and applied
  and measurable after restart, including a restore out of iconic mode. Schema
  v6 → v7 adds a per-column `width` (absent field loads the default).
- **Startup (issue 2).** A normal launch opens no document. `--self-test` and
  `--headless` keep the scratch document their checks depend on.
- **Import identity (issues 3, 4).** An opened raster carries a display name so
  its tab reads `base (Mode/Bits)` with the dirty marker; a fully opaque import
  becomes one locked opaque `Background` layer (D1), while any non-opaque input
  stays a regular alpha layer named from the file stem.
- **Layers panel (issues 5, 10, 11, 12).** Dragging a row activates the existing
  reorder pipeline; double-click renames only on the name (Layer Style elsewhere
  is a documented placeholder); the color label tints the eye gutter; `Ctrl+G`
  groups the selection through the selection-aware op.
- **Numeric input (issues 13, 14, 15, 16).** One shared `NumericField` (D3)
  generalizes the existing `PercentField`, giving scrubbing labels, a slider
  popup, and Left/Right + Home/End/PageUp/PageDown handling; the shared
  `JumpSlider` is hoisted for the navigator and color sliders so press-drag
  tracks instead of jumping.
- **Canvas view (issues 17, 18, 19).** A brush-size circle overlay in image space
  tracks size and zoom; at least one display inch (96 logical px) of the canvas
  stays visible via one `offsetRangeFor` clamp at every mutation point (D5); two
  workspace scrollbars project `offset_`/`zoom_` and hide when the document fits.
- **Locks (issues 6, 9).** A new `layer-locks` capability owns enforcement of
  `POSITION`, `PIXELS`, and `TRANSPARENCY` at the mutation entry points (D4);
  refusal surfaces to the user. Structural panel reordering is not blocked by
  position lock, and enforcement is not placed in `topmost_pixel_layer*` because
  that selector is shared with filters.
- **BREAKING:** none. The session schema advances v6 → v7 but an older store
  loads defaults (missing width), so an existing `state.json` keeps working.

## Decisions

These are settled for this change; the proposals below do not re-open them.

- **D1 (issue 4).** A fully opaque imported raster becomes one layer named
  `Background` with `background = true`, `LockFlags::all()`, and the redundant
  opaque alpha channel dropped. An image with any non-opaque pixel is a regular
  alpha layer named from the file stem. Rationale: matches Photoshop, and the
  compositor already treats a missing channel as opaque
  (`crates/pictura-render/src/composite.rs`).
- **D2 (issue 19).** The scrollbars are a pure projection of `offset_`/`zoom_`;
  `offset_` stays the single source of truth. Zoom and offset persistence is
  explicitly out of scope, so no `workspace-persistence` schema field is added
  for it.
- **D3 (issues 13/14/16).** Generalize the existing `PercentField`
  (`crates/pictura-app/cpp/panels/percent_field.{h,cpp}`) into one shared
  `NumericField`, re-express `PercentField` as a thin configuration, and migrate
  the other numeric controls to it. No two parallel implementations.
- **D4 (issues 6/9).** A new `layer-locks` capability owns enforcement of
  `POSITION`, `PIXELS`, and `TRANSPARENCY` locks at the mutation entry points;
  `layers-panel`/`layer-management` keep owning the lock-state UI. Enforcement
  does not go in `topmost_pixel_layer*` because that selector is shared with
  filters.
- **D5 (issues 18/19).** One shared range helper, `offsetRangeFor`, serves both
  the pan clamp and the scrollbar ranges; never two copies.

## Capabilities

### New Capabilities

- `numeric-fields`: one shared `NumericField` (scrubbing label, slider popup,
  Left/Right + Home/End/PageUp/PageDown keys, scrub sensitivity and Shift/Ctrl
  modifiers), with `PercentField` as a thin configuration.
- `canvas-scrollbars`: workspace scrollbars that project `offset_`/`zoom_` and
  hide when the document fits, following pan, zoom, fit, and the Navigator.
- `layer-locks`: enforcement of `POSITION` (move-tool drag, content move),
  `PIXELS` (paint, filters, fills), and `TRANSPARENCY` at the mutation entry
  points, with user-visible refusal.

### Modified Capabilities

- `workspace-persistence`: the quit path always persists; a per-column width
  round-trips and is applied and measurable after restart.
- `panel-column`: the session store carries a width per column and seeds the
  normal-mode width from the stored `railWidth`.
- `application-shell`: a normal launch opens no document; `--self-test`/
  `--headless` keep the scratch document.
- `document-lifecycle`: a document is created only by an explicit New/Open
  action, never implicitly at launch.
- `document-tabs`: the tab title is `base (Mode/Bits)` plus the dirty marker.
- `image-import`: an import keeps a display name and an opaque input becomes the
  locked `Background` (D1).
- `layers-panel`: drag-reorder is active, rename triggers only on the name, the
  color label tints the eye gutter, and `Ctrl+G` groups the selection.
- `tool-framework`: the options bar and toolbar numeric controls use the shared
  `NumericField`; the cursor reflects the locked state.
- `brush-tools`: a brush-size circle overlay in image space tracks size and zoom;
  the size/hardness/opacity/flow controls use the shared `NumericField`.
- `svg-cursors`: the SVG cursor is the fallback; the brush outline is a drawn
  overlay, not a cursor pixmap.
- `navigator-panel`: pressing and dragging the zoom slider tracks rather than
  jumping.
- `color-swatches-panel`: pressing and dragging a color slider tracks rather
  than jumping.
- `canvas-tools`: at least one display inch of the canvas stays visible; every
  `offset_` mutation goes through one range helper.
- `filter-application`: applying a filter to a pixel-locked layer is refused and
  leaves the layer unchanged.

## Impact

- **App C++ only, plus a renderer predicate and a core import post-process.**
  No new crate or dependency. `CMakeLists.txt` gains the new `selftest_*.cpp`
  translation units and any new shared headers explicitly (there is no globbing).
- `crates/pictura-app/cpp/`: `frame.cpp`, `frame_menus.cpp`, `frame_columns.cpp`,
  `frame_session.cpp`, `session.{h,cpp}`, `main.cpp`, `tools.cpp`, `image_view.{h,cpp}`,
  `layers_panel.cpp`, `layers_panel_internal.h`, `layers_panel_actions.cpp`,
  `command_tree.cpp`, `theme.cpp`, `panels/percent_field.{h,cpp}`,
  `panels/navigator_panel.cpp`, `panels/color_panel.cpp`, `options_bar.cpp`, and
  the dialogs carrying numeric controls; new `NumericField`, `JumpSlider`, and
  `offsetRangeFor` homes.
- `crates/pictura-app/src/cxxqt_object/impl_core.rs`: the opaque-import
  post-process (D1) and the display-name plumbing; `src/cxxqt_object.rs` for the
  mode/depth accessors used by the tab title.
- `crates/pictura-render/`: the `layer-locks` predicate used by the translate,
  content-move, paint, filter, and fill entry points.
- `crates/pictura-app/cpp/selftest*.cpp`: one new file per implementation batch
  starting at code **299**; `selftest.cpp` (6730 LOC, at its allowlist ceiling)
  does not grow and the pan assertions 64/65 and the `zoom/pan transform wrong`
  check at `selftest.cpp:6702` move to the clamp.
- `openspec/specs/`: one archived capability per new capability and MODIFIED
  deltas per listed capability. No `docs/` change.

## Out of scope

- **Issue 7 (arrow-key nudge), deferred.** It needs a multi-layer translate in
  the renderer plus a shortcut decision that conflicts with
  `docs/dev/canvas-view-spec.md:141` and `docs/02-ui-ux/keyboard-shortcuts.md:389-392`.
  A single-layer-only nudge would half-satisfy the requirement, so the whole
  item is deferred rather than half-landed.
- **Issue 8 (brush slow at 4000×4000), deferred.** This is a `paint-engine`
  region/architecture project (incremental dirty rects, a real region compositor)
  that needs benchmarking, not an app-UI change. It gets its own change.
- **Zoom/offset persistence (D2).** Deliberately not stored, so no schema field
  and no restore path is added for the scrollbar position.
