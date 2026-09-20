## Context

Follow-up to the archived `polish-app-ui`. The verification pass (see
`proposal.md`) separated already-shipped behaviour from genuine residuals. This
change fixes only the residuals; the deferred performance ceilings are recorded
in "Deferred".

## Goals / Non-Goals

**Goals**

- Make the empty-workspace rule hold on first show, not only on transitions.
- Make every selection tool resolve its combine mode the same way.
- Keep the press-locked combine cursor after a modifier release mid-drag.
- No stray layer or history state from a zero-offset Alt press.
- Live pixel preview for an Alt selection duplicate.
- Refresh the canvas cursor on a visibility change.
- Apply the outstanding Layers row gutter / thumbnail-aspect / drop-indicator /
  row-height cosmetics.

**Non-Goals**

- No compositor math, document-format change, GPU path, or new dependency.
- No change to the spec-required Alt-pivot geometry semantics.
- No attempt to chase the large-image paint smoothness or the full-canvas
  visibility latency; those remain documented ceilings.

## Decisions

### D1 — First-launch empty pane

Call `refresh()` once at the end of the `PicturaMainWindow` constructor, after
every pane is built and the session restored. `refresh()` stays the single writer
of `tabs_` visibility; the constructor simply routes through it rather than
duplicating `setVisible`. A hidden tab pane on a window with zero documents is
the intended empty workspace.

### D2 — Lasso reads the resolved mode

`LassoToolHandler::onPress` names its `mods` argument and calls
`ctx.setDragMode(ctx.resolveSelectionMode(mods, v->has_selection()))` before
`begin_lasso`, mirroring `PolygonalLassoToolHandler`, the Wand, and the marquee.
`begin_lasso` then receives `selectionModeString(ctx.dragMode())`. This keeps one
mode-resolution rule for all selection tools. The same press-captured mode feeds
the drag cursor.

### D3 — Cursor precedence: gesture beats hover

In `ToolController::refreshCursor`, evaluate the press-locked selection drag
branch (`isSelectionTool(active_) && dragging_ && !movingSelection_`) before the
hover `cursor.moveSelection` branch. Hovering with no press still shows the move
cursor; an active combine gesture keeps the add/subtract cursor for its whole
duration even after the modifier is released, because `dragCursorId()` derives
from the press-captured `dragMode_`.

### D4 — Zero-offset Alt duplicate

Prefer deferring the duplicate to the first non-zero move: no clone is created
(much less recorded) until the pointer actually moves, which matches the "drag to
clone" gesture and makes a bare Alt click a no-op. If deferral proves invasive to
the preview cache, the fallback is to keep the press-time clone but add a
`cancel_move_duplicate` bridge call that removes the inserted layer and restores
the previous active layer on a zero-offset release. Either way the contract is:
zero movement leaves no layer and no history state; the first move inserts the
clone and the preview shows it.

### D5 — Live selection-duplicate preview

Reuse the Move preview mechanism for the selection case: on an Alt press with a
selection, prepare the duplicated pixels as the preview layer and translate it
with the pointer; commit records one `"Move Selection"` state and leaves the copy
active, cancel restores bit-identically. This removes the `ponytail:` deferral in
`impl_selection.rs` that previewed only the outline. The engine copy helper
(`pictura_render::move_selection_content`) already exists and is reused.

### D6 — Cursor refresh on visibility

Route the active layer's visibility/identity change into
`ToolController::refreshCursor` (e.g. from the `changed`/`regionBlitted` funnel or
by making `bindCanvas` re-apply the tool policy when the canvas is unchanged), so
an invisible active layer shows the Block cursor without requiring a mouse move.
The paint filter itself already refuses edits on an invisible layer.

### D7 — Layers row gutter

- Centre the eye toggle inside its fixed gutter: `eyeRect` is the icon square
  centred in `[itemRect.left(), itemRect.left() + kEyeColumn)`, so left and right
  padding are equal (currently left 6 / right 0). The label tint and the eye icon
  both use this rect.
- Draw a 1 px separator in a slightly darker grey at the gutter's right edge,
  between the eye and the content.
- Reserve the chevron width only for expandable rows, so a non-group layer's
  thumbnail starts immediately after the gutter instead of leaving an empty
  16 px slot. `thumbRect`/`nameRect`/`paint`/`chevronRect` move together, and the
  existing hit-test tests are updated to the new geometry.

### D8 — Canvas-aspect thumbnail

The delegate computes the thumbnail's drawn rect inside the row's thumbnail box
using the document aspect ratio, letterboxing rather than stretching. The
document dimensions are supplied through a new row role (or the existing
thumbnail projection, if it can carry the aspect), and the outline, checkerboard,
brackets and hit-test rect all use the letterboxed rect.

### D9 — Styled drop indicator

Track the current drop row and mode while dragging; draw the CS6 indicator in the
view's paint pass — a thin blue line above/below a sibling and a thin blue
outline around a group for a drop-into — gated by the existing `dropValidator_`
so an invalid target shows nothing. Keep the closed-hand cursor and the
above/below resolution unchanged.

### D10 — Row height

Raise the named row-height floor and padding so a Medium thumbnail yields a
slightly taller row (target ≈ 36 px), from the single constant used by `sizeHint`
and the delegate. The existing "at least 28 px" requirement still holds.

## Deferred / verified-not-a-defect

- **Paint smoothness on large documents** — live render and per-dab region
  compositing are already fixed. Remaining ceilings (per-dab GPU buffer
  allocation, full-composted layer-effect stacks, the >64 MP present-cache
  cutoff, synchronous menu refresh) are profiled but not addressed here.
- **Full-canvas visibility toggle latency** — the region fast path helps bounded
  layers; a full-canvas region is a full composite plus a whole-document history
  snapshot. Sub-second for that case needs a structural snapshot change and is
  deferred.
- **Alt pre-press pivot** — current behaviour (from-centre geometry, no combine
  pre-toggle) is explicitly required by `shape-selection-tools`; not changed.
- Everything else in the report is already implemented and covered by passing
  self-tests.

## Risks / Trade-offs

- Changing row geometry touches hit-testing and several self-tests; the change
  updates them in lockstep and adds a gutter/separator assertion.
- The live selection-duplicate preview must not double-record history; the commit
  path stays the single `record("Move Selection")`.
- The drop-indicator paint pass must not interfere with selection/checkbox paint;
  it draws only while a valid drag target exists.

## Migration

None. No persisted state or document format changes.
