# Design

## Context

`crates/pictura-render/src/document_ops/layer_ops/` already owns the layer
tree primitives the change needs: `properties::move_path` (within-container
reorder with Background/locked guards), `properties::move_path_to`
(reparent/place), `merge::merge_scope` / `MergeScope::Down`, and
`create::add_raster_layer_from_rgba`. The bridge (`impl_layers.rs` /
`impl_layers_merge.rs`) exposes them as `move_layer_path`, `merge_layers`,
`delete_layers`, etc., each wrapping a mutation in `recomposite()` +
`record(label)`. The C++ `CommandRegistry` drives the `Layer` menu from
`command_tree.cpp`, with handlers and enablement wired in `frame_menus.cpp`.

`doc.layers` is bottom-first: index `0` is the bottom, the last index is the
front/top. `flatten_rows` emits top-first. The panel group header menu already
lists an Arrange submenu of stubs; this change does not touch that menu (the
task scopes the work to the `Layer` menu), so no `ui/layers-panel` delta.

## Goals / Non-Goals

**Goals:**
- Add engine ops only where a primitive is missing: within-container arrange
  (including edge moves), contiguous-run reverse, and non-destructive stamp.
- Reuse the existing merge/stamp-composite machinery; no second compositor.
- Keep every command a single undo state via the existing bridge pattern.

**Non-Goals:**
- No change to the Layers panel header menu or its Arrange stubs.
- No change to Merge Layers / Merge Visible / Flatten behaviour.
- No new PSD serialization; stamps are ordinary raster layers.

## Decisions

- **Arrange reuses `move_path`.** `arrange_path(doc, path, op)` computes the
  signed delta for Front (`len-1-index`), Forward (`+1`), Backward (`-1`), and
  Back (`-index`) and delegates to `move_path`, so the Background/locked guards
  and boundary clamping stay in one place. A read-only `can_arrange_path`
  mirrors the target computation so the menu enables exactly when the position
  changes. Alternative: four bespoke functions — rejected as duplicated guards.
- **Reverse requires a contiguous run in one container.** The panel selection
  is resolved with `selected_paths`, then the selected siblings must share a
  parent and have consecutive indices; the slice is reversed in place. This
  matches the documented "selected contiguous run" and makes refusals
  unambiguous. Alternative: reorder non-contiguous selections among themselves
  — rejected, unsourced and surprising.
- **Stamp composites a scratch document, then adds + repositions.** `stamp_scope`
  builds the same input list Merge uses (visible nodes or the selected paths),
  composites it with `crate::composite_rgba`, packs the planar buffer to RGBA8888,
  and calls `add_raster_layer_from_rgba`, then `move_path_to(..., mode 0)` to put
  it directly above the active layer. Originals are never removed. This keeps
  one compositor and satisfies the "reuse primitives" constraint.
- **Merge Down gets its own frozen id and handler** calling
  `merge_scope(MergeScope::Down)`; `Ctrl+E` still dispatches Merge Layers.

## Risks / Trade-offs

- [Stamp position when the active layer is in a group] → `move_path_to` places
  the stamp as a sibling directly above the anchor; a group active layer is a
  valid anchor and is not descended into.
- [Fully transparent composite still adds a layer] → acceptable; Photoshop also
  adds the stamp layer, and it is undoable.
- [Arrange boundary commands] → disabled via `can_arrange_path` rather than
  shown enabled and refused.

## Open Questions

None.
