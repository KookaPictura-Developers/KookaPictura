## Context

M20 built the Layers panel as a flat `QAbstractTableModel` over
`Document::layers` (`crates/pictura-app/cpp/panels/layers_panel.cpp:168–305`);
`refresh()` walks top-level layers only. The bridge
(`crates/pictura-app/src/cxxqt_object.rs`) mirrors that with
`doc.layers[i]`-indexed getters/setters and the M37 creation ops. The document
model is already a tree: `Layer.children: Vec<Layer>` (bottom-first),
`is_group`, `mask`, `adjustment`, `clipping`, `fill`, `lock: LockFlags`,
`color: ColorLabel` (`crates/pictura-core/src/lib.rs:343`). The missing piece is
a projection from that tree to the panel and a way to address and edit nested
nodes.

`docs/05-layers/layer-management-ui.md` (`LAY-002`) and
`docs/02-ui-ux/panels/layers-panel.md` (`PAN-001`) are the contracts; the frozen
interfaces live in `docs/dev/m39-panel-anatomy.md`. Constraints: docs/proposal
only in this change; no new dependency; `cxx-qt-lib` carries `QString`,
`QStringList`, and `QImage` across the bridge; the CPU compositor and PSD I/O
are untouched; every mutation keeps the M34 composite-then-record convention
(`record` after `recomposite`, so a snapshot carries the rendered composite).

## Goals / Non-Goals

**Goals:**

- Freeze the layer-path grammar and project the whole tree across the bridge as
  a depth-first topmost-first row list.
- Keep every existing top-level `layer_*(i)` method working as a wrapper, so no
  existing handler or self-test is disturbed.
- Make property edits and the M37 operations selection-based, path-addressed,
  and one undo step over N rows.
- Promote `LayersModel` to a tree with a delegate for the CS6 row anatomy, and
  add multi-selection, solo visibility, `Tab` rename, Panel Options, menus, and
  tooltips.
- Ship runnable checks: Rust unit tests for the path/batch core and `m39_*`
  C++ self-test steps.

**Non-Goals:**

- Filter/search (M40), the remaining management operations and the New Layer
  dialog (M41), styles/effects and `Expand New Effects`' consumer (M42), smart
  objects / vector masks / artboards / layer comps (M43).
- Native drag-reorder (deferred to M41); a stable layer id; cross-container
  moves; group content thumbnails; per-document expansion persistence.

## Decisions

### 1. Paths are bottom-first child-index sequences (frozen)

```
path    := segment *( "/" segment )
segment := "0" / ( %x31-39 *DIGIT )
```

`"0"` is `doc.layers[0]` (bottom), `"2/1"` is `doc.layers[2].children[1]`, and
the empty string is invalid. Paths are positional, not ids: structural changes
invalidate them, and a path-based op on an out-of-range segment fails/no-ops
rather than panicking. Display order is topmost-first, so the projection walks
each container from its last index to `0`.

*Alternative considered:* cxx-qt can return `QStringList`/`QImage` but not a
`QVector<RowStruct>` without an extra bridge. Row getters keyed by a flat index
plus path-based setters match the existing `layer_name(i)` style and keep the
C++ `refresh()` loop shape. Chosen.

*Alternative considered:* return a JSON blob of rows. Rejected: parsing on the
C++ side duplicates the schema and is slower for no gain.

### 2. Top-level methods are wrappers, not migrated (frozen)

`layer_*(i)`, `set_layer_*(i, …)`, `add_layer(above)`, `add_group(above)`,
`duplicate_layer`, `group_layer`, `ungroup_layer`, and `remove_layer` all
resolve top-level path `"<i>"` through the new core. This is deliberately the
smallest diff: `frame.cpp`, the command handlers, and `m20`/`m24`/`m36`/`m37`/
`m38` keep their call sites and assertions. One implementation (the tree core)
sits behind both surfaces. The panel itself moves to the path API.

### 3. Bridge API (frozen)

Read: `layer_row_count()`, then for a flat row index
`layer_row_{path,depth,name,kind,visible,blend,opacity,fill,lock,color,clipping,has_mask,has_adjustment,expandable,child_count}(i)`,
plus `layer_row_thumbnail(i, size, entire_document)` and
`layer_row_mask_thumbnail(i, size)`.

Single mutation: `set_layer_name_path(path, name)`, `move_layer_path(path,
delta)`.

Batch (`QStringList`, one undo step):
`set_layers_{visible,blend,opacity,fill,lock,color}(paths, …)`,
`apply_visibility(paths, label)`, `delete_layers(paths)`,
`duplicate_layers(paths)`, `group_layers(paths) -> QString`,
`ungroup_layers(paths)`.

Creation: `add_layer_in(selection_path) -> QString`,
`add_group_in(selection_path) -> QString`.

Batch count is `i32` (nodes changed); a zero-change batch records nothing. Undo
labels are the M36/M37 strings plus `apply_visibility`'s caller label.

### 4. Multi-selection refusals are per-node skips (frozen)

`LAY-002`'s refusal rules are stated for one selection ("Background and locked
layers refuse…", "a group has no Fill"). For a mixed selection the design
applies the same rule **per node**: an ineligible node is skipped and the
eligible nodes change, all in one undo step; only `group_layers` refuses as a
whole (a partial group has no meaning, and a group cannot contain the
Background or span containers). The table:

| Control | Background | Fully locked | Group |
|---|---|---|---|
| Visibility | apply | apply | apply |
| Lock | skip | apply | apply |
| Blend | skip | skip | apply |
| Opacity | skip | skip | apply |
| Fill | skip | skip | skip |
| Color | skip | apply | apply |
| Delete | skip | skip | apply |
| Duplicate | apply | apply | apply |
| Group Layers | refuse any | refuse any locked | apply |
| Ungroup Layers | skip | skip | apply (groups only) |

*Alternative considered:* refuse the whole batch if any node is ineligible.
Rejected: CS6 applies a multi-selection edit to the layers that permit it; a
single Background in the selection must not block editing the rest.

### 5. Solo is panel state; `apply_visibility` is the one undo state (frozen)

`Alt`-click snapshots every row's visibility into a `QHash<QString,bool>` in
`LayersPanel`, then calls `apply_visibility(visiblePaths, "Solo Visibility")`
with the clicked path, its ancestors, and (for a group) its still-visible
descendants. A second `Alt`-click calls
`apply_visibility(snapshotTruePaths, "Restore Visibility")` and clears the
snapshot. The snapshot is transient (never serialized) and is cleared on a
document switch or before any structural op, so a stale path can never be
restored. Each direction is one undo state, satisfying "every edit is a command
with an undo record" (`LAY-002` Algorithms step 4).

*Alternative considered:* store the snapshot in the document. Rejected: it is
not document state, PSD has no place for it, and it would pollute undo
snapshots.

### 6. One visible tree column; header controls read the selection (frozen)

`LayersModel` becomes a `QAbstractItemModel` with one column. Blend, opacity,
fill, lock, and color are header controls, not columns; the model exposes them
as roles so the delegate (color swatch, lock icon) and the header can read a
row. The header shows the **current** row's value and is enabled iff at least
one selected row is eligible; applying it calls the matching `set_layers_*` over
the whole selection. This is the CS6 layout and removes the M20 Mode/Opacity
columns, which no test asserts.

### 7. Delegate paints the anatomy; assets have a fallback (frozen)

`LayerRowDelegate : QStyledItemDelegate` paints the eye, thumbnail (pixel) or
folder glyph (group), name, color swatch, clipped-layer indent and base
underline, mask thumbnail, and `fx` badge when `HasAdjustmentRole` is set, and
handles the eye hit-test (toggle, `Alt`-click = solo). The disclosure triangle
is the tree style's branch indicator. The `fx` badge uses `layers.fx`; a null
asset omits the badge (guarded by `m38_panels`, so this is not a supported
state).

### 8. Panel Options are session state with fixed defaults (frozen)

Thumbnail size None/Small/Medium/Large default **Medium** (24 px); thumbnail
contents Entire Document / Layer Bounds default **Entire Document**; Expand New
Effects default **on**. Thumbnail contents is resolved when building the
thumbnail: Layer Bounds fills the square from the layer's `rect`; Entire
Document draws the layer at its document position scaled into the square. The
three values persist in the XDG session store at schema v3; older/missing fields
load the defaults. `Expand New Effects` has no consumer until M42 and is
persisted now.

*Alternative considered:* skip the dialog and hard-code Layer Bounds. Rejected:
`LAY-002` names both contents modes and `Entire Document` as the default.

### 9. Menus are minimal and wired; strip is the CS6 seven (frozen)

The panel menu and row context menu show only commands M39 (and M37) actually
wire: `Panel Options…`, New Layer/Group, Duplicate, Delete, Group/Ungroup, Move
Up/Down, and the M36 color-label submenu. The full CS6 panel-menu set
(Merge/Flatten/Blending Options/masks/link/rasterize/select) stays in the
disabled `Layer` menu via `command_tree.cpp` and lands in M41–M43. Move Up/Down
leave the bottom strip, so it is exactly the CS6 seven and the M38 objectNames
survive.

### 10. Drag-reorder deferred (frozen)

Native drag-reorder needs a cross-container move with validation; that is M41
management work. M39 sets `setDragEnabled(false)`, implements no
`dropMimeData`, and reorders only through `move_layer_path` (adjacent swap in
the container) from the menus. The eventual drop rules are recorded in the
brief.

## Risks / Trade-offs

- **Bridge surface is wide (≈30 new methods).** → It is mechanical and mirrors
  the existing per-layer getters; the top-level wrappers keep the diff to
  existing callers at zero. Rows are read-only projections, so the risk is
  bounded to the new methods.
- **Paths are positional.** → A stale path fails safely; the panel rebuilds its
  rows on every `changed` and clears solo/selection paths that no longer
  resolve. A stable `lyid` is M43's.
- **`QAbstractItemModel` is more code than the table.** → The alternative
  (keep the table and fake indentation) does not give real expand/collapse or
  `QItemSelectionModel` extended selection, so it would be throwaway work.
- **A mixed-selection skip can surprise.** → The header shows the current row's
  value and the panel does not mutate ineligible nodes; the per-node rule is the
  batch form of the M36 single-selection refusal and is tested.
- **`Expand New Effects` is inert.** → Stated in the UI tooltip and the brief;
  it is persisted now so M42 does not need a migration.
- **`apply_visibility` can change many rows in one step.** → It is exactly the
  CS6 solo contract (one state) and undo restores all prior visibility.
- **Session schema v3.** → Older stores load the defaults, matching the M26 v2
  pattern; no document format changes.

## Migration Plan

Additive and app-local, except the panel rewrite. Rollback: restore the table
model and the M37 single-layer handlers, delete the new bridge methods, and drop
the three session fields. The document model, codec, and compositor are
untouched, so no on-disk migration is needed. Sequence: freeze the brief →
`layer_ops` path/batch helpers + tests → bridge → tree model/delegate →
selection/batch → solo/rename/tooltips → Panel Options/menus → self-test →
STATE.

## Open Questions

- **Thumbnail-size default.** `LAY-002`'s table says `medium` while its Open
  questions says the fresh-install default is unstated. This design follows the
  table (Medium = 24 px, also the current size). A CS6 preference dump would
  settle it.
- **Group content thumbnails.** `LAY-003` asks whether a group thumbnail follows
  Panel Options; M39 draws a folder glyph and defers the group composite.
- **Expansion persistence.** Whether CS6 persists group expansion is unsourced;
  M39 keeps it session-only.
- **`group_layers` ordering across containers.** M39 refuses a selection that
  spans containers; if CS6 instead flattens, M41 can relax it.
