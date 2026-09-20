## Why

Verifying the archived `polish-app-ui` round against the tree at `15b310e`
confirms most of its 24 fixes landed with passing self-tests, but a residual set
is incomplete. Every remaining item below is a confirmed root cause, not a
hypothesis:

- On a normal launch with no document, `PicturaMainWindow::refresh()` — the only
  writer of the tab pane's visibility — is never called, so the empty
  `documentTabs` pane (and its ghost-canvas background) is shown at startup. The
  archived fix only covered the create/open/close transitions.
- The freehand Lasso handler never resolves its combine mode: it reads the stale
  `dragMode_` instead of `resolveSelectionMode(mods, has_selection)`, so Shift
  and Alt and the options-bar combine buttons are ignored (and the drag cursor is
  wrong).
- `ToolController::refreshCursor` returns the move-selection cursor before the
  press-locked drag cursor, so releasing Shift/Alt mid-drag drops the
  add/subtract cursor even though the mode is retained.
- An Alt press with no movement inserts a duplicate layer via
  `begin_move_duplicate` but `onRelease` only commits on a non-zero offset, and
  `end_move_preview` does not roll the clone back: a stray, unrecorded layer is
  left behind.
- An Alt move of a **selection** previews only the selection outline; the copied
  pixels appear on release. The whole-layer branch already previews the clone.
- The canvas cursor is not refreshed when the active layer's visibility changes,
  because a visibility toggle at the origin emits `changed`, `refresh()` →
  `bindCanvas` early-returns on an unchanged canvas, so `refreshCursor` is
  skipped until the next mouse move.
- The Layers row gutter cosmetics were only partly applied: the eye is
  left-anchored (`left + 6`) with no right padding, there is no separator between
  the eye gutter and the content, and the always-reserved chevron slot leaves a
  16 px gap left of a non-group thumbnail. Thumbnails are drawn into a hardcoded
  square box, so they stretch rather than matching the canvas aspect ratio. The
  row's drop indicator is the stock Fusion primitive, not the CS6 thin blue line
  / group outline, and the row-height floor was not raised.

Out of scope (verified, not defects or not safely groundable): the workspace
wheel/zoom, tab weight, Background rename dialog, eye-only tint, selection
highlight clip, checkerboard, thumbnail outline/brackets, row fonts, hidden
nesting lock, nudge, transient Alt eyedropper, and layout-independent `[`/`]`
bindings are all already implemented and tested. The Alt pre-press behaviour is
spec-required (`shape-selection-tools` "Alt SHALL NOT pre-toggle"), so the
report's item 3 is not a bug. Paint live-render and per-dab region compositing
are fixed; the remaining large-image smoothness and full-canvas visibility
latency are performance ceilings recorded but not fixed here.

## What Changes

- **Startup:** call `refresh()` once at the end of the `PicturaMainWindow`
  constructor so the single-writer empty-pane rule also covers the first show.
- **Lasso combine mode:** resolve the mode at press from the live modifiers and
  the existing selection (mirroring Polygonal Lasso / Wand) before `begin_lasso`,
  so Shift/Alt and the options-bar mode apply.
- **Drag cursor retention:** in `refreshCursor`, check the press-locked
  selection drag cursor before the hover move-selection cursor, so a combine
  gesture keeps its add/subtract cursor after a modifier release while merely
  hovering still shows the move cursor.
- **Alt zero-drag:** an Alt press with no movement SHALL leave no layer and no
  history state — defer the duplicate to the first move or roll it back on a
  zero-offset release.
- **Alt selection duplicate preview:** the selected pixels SHALL follow the
  cursor during the drag, not only the outline, reusing the Move preview
  mechanism, and the copy SHALL be active on commit.
- **Cursor refresh on visibility change:** refresh the canvas cursor when the
  active layer's visibility or identity changes.
- **Layers row gutter:** centre the eye toggle in its gutter with equal padding,
  draw a slightly darker 1 px separator between the gutter and the content, and
  drop the reserved chevron width for non-expandable rows so the thumbnail sits
  closer to the gutter.
- **Thumbnail aspect:** size the thumbnail to the document's aspect ratio within
  the row box instead of stretching it into a square.
- **Drop indicator:** draw the CS6 indicator — a thin blue line above/below for
  a sibling reorder and a thin blue outline around a group for a drop-into —
  gated by the existing drop validator.
- **Row height:** slightly raise the named row-height floor.

## Capabilities

### New Capabilities

None. Every item corrects or extends an existing capability.

### Modified Capabilities

- `application-shell`: the empty-pane rule also applies on first show.
- `layers-panel`: row gutter/separator spacing, canvas-aspect thumbnails, the
  styled drop indicator, and the raised row height.
- `shape-selection-tools`: Lasso resolves its combine mode; the captured combine
  cursor survives a modifier release.
- `selection-content-move`: the Alt selection duplicate previews its pixels.
- `canvas-tools`: an Alt press with no movement leaves no duplicate.
- `tool-framework`: the canvas cursor refreshes on a visibility change.

## Impact

- **C++ app** (`crates/pictura-app/cpp/`): `frame.cpp` (constructor refresh,
  cursor refresh on visibility), `tool_selection.cpp` (Lasso), `tools_marquee.cpp`
  (cursor precedence), `tool_move.cpp` (zero-offset Alt),
  `tools_selection_move.cpp` (selection-duplicate preview),
  `panels/layers_panel_internal.h` (gutter, thumbnail, drop indicator, row
  height) and, if a role is needed, `panels/layers_panel.cpp`.
- **Rust bridge** (`crates/pictura-app/src/cxxqt_object/`): `impl_transform` /
  `impl_selection` for the deferred/cancelled duplicate and the live
  selection-copy preview.
- **Engine**: none expected; no document-format change.
- **Tests**: extend the existing C++ self-test TUs and Rust unit tests; new
  failure codes are taken from the next free number after the current maximum,
  and no existing code is reused. `selftest.cpp` does not grow.
- **No new dependency and no `docs/` edit.**
