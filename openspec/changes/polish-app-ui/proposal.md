## Why

The prior change (`fix-app-ui-interactions`) ended with a live-render
regression: a brush dab is not drawn while the mouse button is held and only
appears on release, and the drag stutters on large documents. A round of
hands-on use also confirmed 23 further defects across startup chrome, the Layers
panel, selection modifiers, the brush/pen keys, the Move tool, and workspace
zoom/scroll. Each is a confirmed root cause, not a hypothesis, so they can be
specified and fixed as one reviewable change.

## What Changes

- **Fix the brush live-render regression** (BREAKING for the present-cache
  contract): `ImageView::blitRegion` composes the scaled-cache patch as
  `translate(x, y)` then `scale(zoom, zoom)`, which lands the region at document
  pixels `(x, y)` in the scaled cache rather than `(x*zoom, y*zoom)`. At any
  zoom other than 1 — the fit zoom of a large image — the patch is clipped off
  the cache, so the canvas never shows the dab; release's full `recomposite`
  shows it. The fix composes the patch in the correct order (or translates by
  the scaled origin), keeps the cache patch as the per-dab fast path, and adds
  the missing zoom-≠-1 test. Painting is continuously visible during the drag;
  the stroke commits data and exactly one history state only on release.
- **Startup / chrome.** Hide the document tab pane (and therefore its
  ghost-canvas background) when no document is open, funnelled through
  `frame.cpp::refresh`; give document tab labels a medium weight and a few px of
  right padding.
- **Layers panel.** Make double-click rename work on the whole name/label band;
  return a valid drop flag for the invalid parent so Qt computes the
  above/below drop indicator and show a closed-hand cursor while dragging;
  convert the Background to `Layer N` through a name+color dialog; tint only the
  eye toggle's background with the color label (drop the post-name swatch);
  exclude the eye column from the selected-row highlight; draw a checkerboard
  behind regular-layer thumbnails; outline thumbnails 1 px and bracket the
  singular active layer; italicise `Background` and underline linked/placed
  layers via new row roles; raise the row-height floor; hide the panel's nesting
  lock button while keeping the engine `NESTING` rules; route visibility toggles
  through the region fast path so a large-image toggle is sub-second.
- **Selection tools.** Let Shift/Alt change the combine cursor and mode while
  hovering an existing selection (do not return the move-selection cursor
  first); capture the drag modifiers at press so releasing Shift/Alt mid-drag
  keeps the mode and geometry; keep Alt a geometry-only (from-centre) pivot and
  never a pre-toggle.
- **Invisible layers.** When the active layer is invisible, refuse paint and
  filter edits with a Block/Forbidden cursor while selection, copy, and Move
  (mouse drag and keyboard) still work; stop `compute_move_preview` from forcing
  `visible = true`; add a shared `active_layer_visible` helper.
- **Move tool.** Plain arrow keys nudge the active layer 1 px and `Shift+Arrow`
  10 px; holding Alt during a Move drag duplicates the layer (or the selected
  pixels), previews the duplicate, and makes it active on commit.
- **Brush / Pen.** Holding Alt transiently toggles the Eyedropper (sample + Alt
  cursor) and restores the previous tool on release; `[`/`]` bind by native scan
  code (evdev 34/35 plus Shift variants) so they work on EU/Scandinavian layouts.
- **Workspace zoom / scroll.** Shift doubles the wheel zoom step; Alt+wheel pans
  horizontally, Ctrl+Alt+wheel vertically, and a horizontal side-wheel pans
  horizontally; anchor Zoom-tool clicks and the Navigator slider at the
  click/cursor point so zoom targets the cursor everywhere.

**BREAKING**: the present-cache region patch semantics change; the previous
design's "patch is byte-identical to a direct draw" claim held only at zoom 1
(the self-test exercised `actualPixels`), and the earlier `document-canvas`
region-blit contract is corrected to be zoom-correct.

## Capabilities

### New Capabilities

None. Every item corrects or extends an existing capability.

### Modified Capabilities

- `application-shell`: hide the empty document tab pane so no ghost canvas is
  drawn with no document.
- `document-tabs`: document tab label weight and right padding.
- `layers-panel`: rename band, drop indicator flag and drag cursor, eye-only
  label tint, selection highlight exclusion, thumbnail checkerboard/outline and
  active brackets, row typography and height, and the hidden nesting button.
- `layer-management`: Background-to-normal conversion renames and prompts;
  edits on an invisible active layer.
- `layer-compositing`: visibility toggles use the region fast path with a
  sub-second large-image target.
- `shape-selection-tools`: combine cursor under Shift/Alt, drag-modifier
  capture, and Alt as a geometry-only pivot.
- `brush-tools`: invisible-active-layer paint refusal, transient eyedropper,
  and layout-independent `[`/`]` bindings.
- `canvas-tools`: keyboard nudge, Alt-drag duplicate, and wheel/zoom modifier
  precedence with cursor-anchored zoom.
- `selection-content-move`: Alt-drag duplicate previews and commits as the
  active layer.
- `tool-framework`: the transient-eyedropper cursor and the invisible
  active-layer cursor branch.

## Impact

- **C++ app** (`crates/pictura-app/cpp/`): `image_view.{h,cpp}` (blitRegion,
  wheel), `theme.cpp` (tab QSS), `frame.cpp`/`frame_build.cpp` (empty pane),
  `panels/layers_panel{,_internal.h,_actions,_menu}.cpp`,
  `panels/layer_new_dialog.{h,cpp}`, `tools.cpp`, `tools_marquee.cpp`,
  `tools_selection_move.cpp`.
- **Rust bridge** (`crates/pictura-app/src/cxxqt_object/`): `impl_layers.rs`
  (visibility region path), `impl_paint.rs`/`helpers*.rs` (visible helper),
  `impl_transform.rs` (`compute_move_preview`, `begin_move_duplicate`),
  `impl_selection.rs` (duplicate preview/commit).
- **Engine**: `pictura-render` visibility region path; `pictura-paint` only if
  the stroke needs the visible guard.
- **Tests**: new `crates/pictura-app/cpp/selftest_*.cpp` translation units
  (failure codes from **346**), Rust unit tests in the owning crates, and one
  C++ check that blits at a zoom other than 1. `selftest.cpp` does not grow.
- **No document-format change, no new dependency, no `docs/` edit.**
