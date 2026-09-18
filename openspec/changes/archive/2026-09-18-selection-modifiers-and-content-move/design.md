## Context

The selection tools already share one `ToolController` switch and a
`Selection`-based bridge (`select_rect`/`select_ellipse`/`begin_lasso`/
`end_lasso`/`magic_wand`/`quick_select`) in
`crates/pictura-app/src/cxxqt_object/impl_selection.rs`. Combine modes are
already a `mode_` field (New/Add/Subtract/Intersect) set from the options bar and
applied at commit. The Move tool renders a cached base + moving layer
(`impl_transform.rs`, `image_view.cpp`) and only shifts the topmost layer's
`rect`. `pictura-render::document_ops::layer_ops::via` already implements
selection-masked copy and cut (`layer_via_copy`/`layer_via_cut`), and
`merge::merge_scope` can merge one layer down.

`ToolController` has no pointer-modifier stream: `ImageView::mouseMoved` carries
only the image position, so drag-time modifiers are read with
`QGuiApplication::queryKeyboardModifiers()` (already the pattern in
`refreshCursor`). `ImageView` never enables mouse tracking, so it only receives
moves with a button held — the reason the move-selection cursor currently
appears only after a press.

## Goals / Non-Goals

**Goals**

- Honour Shift/Alt/Shift+Alt as Add/Subtract/Intersect for the modal selection
  tools, decided per gesture and locked until commit.
- Constrain marquee geometry: Shift = square/circle, Alt = draw from the press
  point as centre; show a live `W × H` readout next to the cursor.
- Render the Polygonal Lasso's in-progress path as a solid rubber band that
  tracks the cursor.
- Update the move-selection hover cursor on plain pointer movement.
- Move selected pixels with the Move tool (Alt duplicates to a new layer) and
  with a selection tool while Ctrl is held (Ctrl+Alt duplicates), one undo
  state per gesture.
- Leave runnable checks: Rust tests for the engine move, C++ self-tests for the
  modifier mapping, geometry, preview flags, and move state.

**Non-Goals**

- A live *pixel* preview during a content move. The gesture previews the moving
  selection outline; the pixels commit on release. (A masked move preview needs
  a new base-composite path in `impl_transform`.)
- Moving a selection from a layer other than the topmost pixel layer; the Move
  tool has always targeted the topmost pixel layer and this change keeps that.
- Alt-drag duplicating a whole layer when no selection is active.
- Transform Selection, Free Transform, or resampling.

## Decisions

### D1. Modifier → combine mode, decided per gesture

A pure helper on `ToolController`:

```cpp
static SelectionMode selectionModeForModifiers(SelectionMode base,
                                               Qt::KeyboardModifiers mods,
                                               bool hasExisting);
```

- No modifier, or no existing selection → `base` (the options-bar mode). This
  keeps Add/Subtract/Intersect from erasing a selection when the base is empty
  (subtracting from nothing would yield nothing).
- Shift only → Add; Alt only → Subtract; Shift+Alt → Intersect.

`handlePressed` computes the effective mode once and stores it in `dragMode_`,
used by every subsequent commit of that gesture. `dragMode_` is re-derived at
the next press, so a gesture that began with Shift stays Add even if Shift is
released before mouse-up. For the Polygonal Lasso the lock spans from the first
vertex to the close; for Quick Selection and the marquee/lasso it spans the
drag.

### D2. Marquee geometry

`marqueeDragRect` gains the live modifiers:

- Alt: the press point is the centre — the rectangle is `press ± |delta|` on
  each axis (a pivot, mirrored on all sides).
- Shift: square — the longer axis sets both extents, sign-preserving.
- Both together: a square centred on the press point.
- Fixed Ratio / Fixed Size keep their existing behaviour; Shift/Alt refine the
  Normal drag only.

The size readout is `QString("%1 x %2").arg(rect.width()).arg(rect.height())`
pushed through a new `ImageView::setDragSizeHint(text, imagePos)` and painted as
a small rounded tooltip offset from the mapped cursor, clamped to the widget.

### D3. Polygonal Lasso preview

The preview polygon is `polygonPoints_ + live cursor`, drawn open, so the
rubber band runs first-vertex → clicked vertices → cursor. `setSelectionPreview`
gains a `solid` flag: a solid light line under a solid dark line, no dash
offset, so the in-progress path reads as a plain line rather than marching ants.
The committed polygon still uses only the clicked vertices (`pending_lasso`), so
the cursor segment is never part of the selection.

### D4. Hover cursor

`ImageView` enables `setMouseTracking(true)`, so `mouseMoved` fires on hover and
`updateSelectionHover` re-evaluates `cursorOverSelection_` continuously.
`refreshCursor` also treats a held Ctrl over the canvas as a pending content
move, and the frame's key handler calls `refreshCursor()` on Ctrl press/release
so the cursor flips without moving the pointer.

### D5. Content move = cut/copy + translate + merge

Engine (`crates/pictura-render/src/document_ops/layer_ops/move_content.rs`):

```rust
pub fn move_selection_content(doc: &mut Document, source_path: &str,
                              mask: &LayerMask, dx: i32, dy: i32,
                              duplicate: bool) -> bool
```

- `duplicate` → `layer_via_copy`, leaving the source intact.
- otherwise → `layer_via_cut`, clearing the source region.
- translate the new layer's `rect` (and mask) by `(dx, dy)`.
- non-duplicate → `merge_scope(MergeScope::Down(&new_path))` so the pixels land
  back on the source layer and the layer count is unchanged.

Bridge (`impl_selection.rs`): `move_selection_content(dx, dy, duplicate)`
resolves the topmost pixel-layer path, uses `selection_move_origin` (or the
current selection) as the mask, clears the origin, recomposites, and records one
`"Move Selection"` state. `dx == dy == 0` and a missing document/selection
refuse without recording.

The `ToolController` content-move drag reuses the existing selection-move
outline preview (`begin_selection_move` / `preview_selection_move`). On release
it calls the bridge move instead of `commit_selection_move`, so the pill and the
pixels move together under one state.

`cxxqt_object.rs` is at the 1000-LOC cap; the new method's declaration trims
equivalent duplicated doc comments to stay under it.

### D6. Ctrl with a selection tool

`maybeBeginSelectionMove` currently intercepts any press inside a selection. It
now splits:

- Ctrl held inside a selection → content move (`duplicate = Alt`).
- No Shift/Alt inside a selection → outline move (unchanged).
- Shift and/or Alt → fall through to a normal combine marquee.
- Move tool with a selection → content move regardless of press point.

## Risks / Trade-offs

- **No pixel preview during content move** — the selection outline moves, then
  the pixels snap on release. Marked `ponytail:` with the masked-base upgrade.
- **Topmost pixel layer only** — matches the existing Move tool; a per-layer
  target needs the panel's `currentPath` plumbed into `ToolController`.
- **`merge_scope(Down)` uses Normal blend** — a moved region merged into a
  non-Normal source composites by the source's own blend/opacity only insofar as
  merge already models it; acceptable for the Normal case the tools exercise.
