# M39 — Layers panel anatomy (CS6)

- **Status:** proposed (`openspec/changes/m39-panel-anatomy`); not implemented.
- **Type:** the third milestone of the layers-panel program (M36–M43), after the
  M38 icon/cursor interruption. See `docs/dev/layers-panel-program.md`.
- **Contract:** `docs/05-layers/layer-management-ui.md` (`LAY-002`) is the
  long-form behaviour spec; `docs/02-ui-ux/panels/layers-panel.md` (`PAN-001`) is
  the widget spec. This file freezes the interfaces those specs leave open.
- **Consumers:** `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}`,
  `crates/pictura-app/src/cxxqt_object.rs`,
  `crates/pictura-render/src/document_ops/layer_ops.rs`,
  `crates/pictura-app/cpp/main.cpp`, `crates/pictura-app/cpp/session.{h,cpp}`.
- **Non-goals:** filter/search (**M40**), merge/flatten/rasterize/select/link/
  convert-background/layer-via-copy-cut and the New Layer dialog (**M41**),
  styles/effects (**M42**), smart objects / vector masks / artboards / layer
  comps (**M43**), and any `docs/` change beyond this file.

## 1. CS6 behavior (cited from LAY-002)

The panel "lists all layers, layer groups, and layer effects in an image"
(`LAY-002` CS6 behavior). The anatomy relevant to this milestone:

- **Rows are a tree.** A group is a folder row with a disclosure triangle;
  expanding shows its children indented. A layer clipped to the layer below is
  indented too, and its base layer's name is underlined (`LAY-002` UI surface;
  `LAY-003` "Clipping into groups").
- **Row columns.** Left to right: eye (visibility), thumbnail, name,
  mask/vector-mask thumbnail with a link icon, clip indicator, style/fx badge
  (`LAY-002` UI surface, "Row anatomy and indicators").
- **Multi-selection.** Click selects; `Shift`-click extends contiguously;
  `Ctrl`/`Cmd`-click toggles non-contiguously. "Select one or more layers or
  groups and edit Opacity and Fill"; CS6 lets a multiple selection change
  **locking, blend mode, or color label** in one action (`LAY-002` "Selecting
  layers", "CS6 layer-management additions"; `PAN-001` "Multi-select behavior").
  A multi-select edit is **one command over several node ids** and one undo step
  (`LAY-002` "Data-model impact"; `PAN-001` "Algorithms & pipeline" step 4).
- **Bottom strip.** The CS6 order — link, layer styles (fx), add layer mask, new
  fill/adjustment, new group, new layer, delete (`LAY-002` UI surface; M38
  contract §7.2). Link, fx, and mask have no operation yet.
- **Solo visibility.** `Alt`/`Option`-click an eye shows only that layer/group
  and remembers prior visibility; a second `Alt`-click restores it exactly
  (`LAY-002` UI surface, "Visibility, lock, opacity/fill, blend mode",
  Algorithms step 6; parity criterion 9).
- **Inline rename.** Double-click the name to edit; in CS6 `Tab` goes to the next
  layer and `Shift+Tab` to the previous (`LAY-002` "CS6 layer-management
  additions" — Rename navigation; UI surface "Rename field").
- **Panel Options.** `Panel Options` in the panel menu offers thumbnail size
  (None/Small/Medium/Large) and thumbnail contents (Entire Document / Layer
  Bounds); `Expand New Effects` controls whether layer styles/smart filters open
  expanded (`LAY-002` "Panel display and options", Parameters).
- **Menus.** A panel menu (top-right triangle) and a row context menu carrying
  the type-specific commands plus the CS6 color label (`LAY-002` UI surface;
  `05-layers/layer-management-ui.md` "CS6 layer-management additions").
- **Tooltips.** Layer tooltips include the layer name (`LAY-002` "CS6
  layer-management additions" — Tooltips).
- **Refusals.** Background and locked layers refuse opacity/fill/blend/reorder;
  a group exposes Opacity only (no Fill); a fully locked layer refuses opacity/
  fill (`LAY-002` Edge cases, parity criteria 2–3; already frozen by M36 for the
  single-selection case).

Everything else in `LAY-002` (filtering/search, the Properties panel, merge
targets, `Rasterize Layer Style`, shape naming, link sets) belongs to another
milestone or is out of CS6 scope.

## 2. Current state (inventory)

The panel is a **flat table** over top-level layers only, and the bridge has no
tree access.

- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}`:
  - `LayersModel : QAbstractTableModel` (`layers_panel.cpp:168–305`) with five
    columns (0 visibility, 1 thumbnail, 2 name, 3 mode, 4 opacity) and 5
    columns reported by `columnCount`; `setData` handles visibility and rename.
  - `refresh()` (`:525–554`) walks `for i = count-1 .. 0` over **top-level**
    layers (`view_->layer_count()` / `layer_*(i)`), building a flat
    `QVector<LayerRow>` (index, name, kind, visible, blend, opacity, fill,
    lockBits, color, thumbnail).
  - `QTreeView` (`:334–344`) with `setRootIsDecorated(false)`,
    `setItemsExpandable(false)`, `SingleSelection`, 24 px icons.
  - Header controls (`:313–330`): blend combo, opacity spinbox, fill spinbox;
    lock strip (`:349–372`); bottom strip of seven icon buttons (`:374–435`)
    plus text **Move Up**/**Move Down** buttons (`:431–434`).
  - `syncControls()` (`:578–617`) enables the header from the current row only;
    `showColorMenu()` (`:619–645`) is the M36 row color-label menu.
  - Tooltip helper `layerTooltip` (`:156–162`): `"<name> (<kind>)"`, or `name`
    for a pixel layer.
- `crates/pictura-app/src/cxxqt_object.rs` `PictureView`:
  - `layer_count` (`:782`), `layer_name`/`layer_kind` (`:797`/`:803`),
    `layer_visible` (`:815`), `layer_blend` (`:841`), `layer_opacity` (`:874`),
    `layer_fill` (`:902`), `layer_lock` (`:930`), `layer_color` (`:962`),
    `layer_thumbnail` (`:1104`), and the `set_*` counterparts (`:819–:1010`),
    `move_layer` (`:1012`), `remove_layer` (`:1929`), and M37's
    `add_layer`/`add_group` (`:1032`/`:1050`) / `duplicate_layer` (`:1068`) /
    `group_layer` (`:1080`) / `ungroup_layer` (`:1092`).
  - **Every one of these indexes `doc.layers[i]` — top level only.** The private
    `layer(i)` helper (`:2034`) is `doc.layers.get(i as usize)`.
- `crates/pictura-core/src/lib.rs` `Layer` (`:343–358`) already carries
  `children: Vec<Layer>` (bottom-first), `is_group`, `mask`, `adjustment`,
  `clipping`, `fill`, `lock: LockFlags`, `color: ColorLabel`.
- `crates/pictura-render/src/document_ops/layer_ops.rs` has the M37 pure ops but
  no path/tree helpers.
- There is **no** expansion state, selection model, tree model, delegate,
  context menu, panel menu, Panel Options, solo visibility, or `Tab` rename
  navigation.

Self-test state at proposal time: `m20_panels` (exit 52, dock registry/toggle),
`m24_panels`/`m24_rail` (62/63), `m36_attrs` (86–89), `m37_create` (90–94),
`m38_panels`/`m38_tools` (99–101/95–98). All layer checks call the top-level
bridge methods listed above.

## 3. Frozen interfaces

### 3.1 Path grammar

A **path** is a `/`-separated sequence of non-negative decimal indices, each
segment an index into the **bottom-first** `children` vector of the node named
by the preceding segments, starting from the document's `layers` vector.

```
path    := segment *( "/" segment )
segment := "0" / ( %x31-39 *DIGIT )      ; no leading zeros
```

- There is no path for the document root: the empty string is invalid.
- `"0"` is `doc.layers[0]` (the bottom layer); `"2/1"` is
  `doc.layers[2].children[1]`; `"0/2/1"` is
  `doc.layers[0].children[2].children[1]`.
- Paths are **display-neutral**: they are bottom-first (the model's order),
  while the panel renders topmost-first by walking each container from its last
  index to `0`.
- A path is **positional, not a stable id**: any structural change (insert,
  delete, move, group, ungroup) invalidates it. Paths are only used within one
  panel refresh cycle; an out-of-range segment resolves to "not found" and a
  path-based operation returns failure / no-op — never a panic. A stable
  `lyid`-style identity is M43's concern (`layers-panel-program.md` §5).
- Resolution is `O(depth)`; a malformed path (empty segment, `+`, sign, leading
  zero, non-digit, or a trailing `/`) is "not found".

**Projection (frozen).** The flattened row list is depth-first, topmost-first:
for a container `C` with `n` nodes, emit indices `n-1 … 0`; emit the node's row,
then, if the node is a group, recurse into it the same way. `depth` is the
number of `/` characters in the path (top-level rows are depth 0). A group is
`expandable` iff it has at least one child. The bridge projects the **entire**
tree regardless of expansion; expansion is view state owned by the Qt model.

### 3.2 Bridge API (frozen)

The existing top-level `layer_*(i: i32)` / `set_layer_*(i, …)` / `add_layer(i)` /
… methods are **kept as compatibility wrappers** that resolve top-level path
`"<i>"`, so `frame.cpp`, the command handlers, and every existing self-test
(`m20`, `m24`, `m36`, `m37`, `m38`) keep working unchanged. They are not
migrated; there is exactly one tree implementation behind them.

**Tree rows (read).** `layer_row_count()` is the number of nodes in the whole
tree (all depths). Each of the following takes a flat row index
`i in 0..layer_row_count()`:

| Method | Returns |
|---|---|
| `layer_row_path(i) -> QString` | the path |
| `layer_row_depth(i) -> i32` | nesting depth (top-level = 0) |
| `layer_row_name(i) -> QString` | name |
| `layer_row_kind(i) -> QString` | `"pixel"` / `"group"` / `"adjustment"` / `"background"` |
| `layer_row_visible(i) -> bool` | visibility |
| `layer_row_blend(i) -> QString` | 4-byte PSD key |
| `layer_row_opacity(i) -> i32` | 0..255 |
| `layer_row_fill(i) -> i32` | 0..255 |
| `layer_row_lock(i) -> i32` | bitmask `0x01/0x02/0x04` |
| `layer_row_color(i) -> i32` | 0..7 |
| `layer_row_clipping(i) -> bool` | clipping flag |
| `layer_row_has_mask(i) -> bool` | a layer mask is present |
| `layer_row_has_adjustment(i) -> bool` | adjustment content present |
| `layer_row_expandable(i) -> bool` | group with ≥ 1 child |
| `layer_row_child_count(i) -> i32` | direct children |
| `layer_row_thumbnail(i, size, entire_document) -> QImage` | pixel layer thumbnail |
| `layer_row_mask_thumbnail(i, size) -> QImage` | mask thumbnail, null without a mask |

`entire_document` selects the thumbnail contents (see 3.5). A group returns a
null thumbnail (the delegate draws a folder glyph); an adjustment layer returns
null (its badge is the `layers.fx` icon).

**Mutation (path-based).** Every mutation recomposites, records exactly one
undo state, marks dirty, and returns a result; a refused/no-op call records
nothing:

| Method | Notes |
|---|---|
| `set_layer_name_path(path, name) -> bool` | inline rename |
| `move_layer_path(path, delta) -> bool` | swap with the neighbour `delta` places away **within the node's own container**; no cross-container move |

**Multi-selection (batch, one undo step).** Each takes a `QStringList` of paths
and returns the number of nodes actually changed (an `i32`); the whole batch is
one recomposite and **one** undo state iff the count is `> 0`. Ineligible paths
are skipped **per node** (a mixed selection still applies to the eligible
ones); see the refusal table in 3.3.

| Method | Notes |
|---|---|
| `set_layers_visible(paths, visible) -> i32` | eye; always allowed per node |
| `set_layers_blend(paths, key) -> i32` | |
| `set_layers_opacity(paths, value) -> i32` | |
| `set_layers_fill(paths, value) -> i32` | |
| `set_layers_lock(paths, flag, on) -> i32` | |
| `set_layers_color(paths, value) -> i32` | |
| `apply_visibility(paths, label) -> i32` | set exactly these visible, every other node invisible, one labeled state (solo enter/restore) |
| `delete_layers(paths) -> i32` | |
| `duplicate_layers(paths) -> i32` | deep-copy each path above itself |
| `group_layers(paths) -> QString` | one new group at the topmost selected position; returns its path or empty |
| `ungroup_layers(paths) -> i32` | splice each selected group in place |

Undo labels: `Layer Visibility`, `Blend Mode`, `Opacity`, `Fill Opacity`,
`Lock`, `Layer Color`, `Rename Layer`, `Reorder Layer`, `Delete Layer`,
`Duplicate Layer`, `Group Layers`, `Ungroup Layers`, `New Layer`, `New Group`,
and the caller-supplied `apply_visibility` label (`Solo Visibility` /
`Restore Visibility`).

**Creation (tree-aware, frozen).**

| Method | Notes |
|---|---|
| `add_layer_in(selection_path) -> QString` | if the path is a group, insert as its **top child**; else insert directly above the named layer; empty path → top of the stack. Returns the new path or empty. |
| `add_group_in(selection_path) -> QString` | same rule for a group |

M37's `add_layer(above: i32)` / `add_group(above: i32)` remain the top-level
compatibility wrappers for these.

**Path → row mapping.** The model stores each row's `path`; selection is a set
of paths (`QItemSelectionModel` mapped through the model's `PathRole`). After a
structural op the panel calls `refresh()` and re-selects by path where the path
still resolves.

### 3.3 Multi-selection and refusal table (frozen)

Controls that apply to **all selected rows in one undo step**: visibility,
lock, blend, opacity, fill, color, delete; grouping, ungrouping, and duplication
also operate over the selection (this is the upgrade of M37's single-layer
`group_layer` / `duplicate_layer`).

Legend: **apply** = the node changes; **skip** = the node is left unchanged and
does not fail the batch; **refuse** = the batch is rejected as a whole
(`group_layers` only, because a partial group is meaningless).

| Control | Background | Fully locked | Group | Non-group | Mixed selection |
|---|---|---|---|---|---|
| Visibility | apply | apply | apply | apply | apply each |
| Lock | skip | apply | apply | apply | apply each |
| Blend | skip | skip | apply (Pass Through + 27) | apply | apply each |
| Opacity | skip | skip | apply | apply | apply each |
| Fill | skip | skip | **skip** (no group Fill) | apply | apply each |
| Color | skip | apply | apply | apply | apply each |
| Delete | skip | skip | apply | apply | apply each |
| Duplicate | apply | apply | apply | apply | apply each |
| Group Layers | **refuse** if any | refuse if any is fully locked | apply | apply | refuse if the paths span more than one container |
| Ungroup Layers | n/a | skip | apply only to groups | skip | apply each |

A batch that changes nothing (`count == 0`) records no history and emits no
`changed`; a `group_layers` refusal records nothing and returns an empty path.
The per-node skip rule replaces the M36 single-selection "refuse the edit"
result; the eligible/ineeligible split is the batch form of the same rule.

### 3.4 Solo visibility (frozen)

- State lives in `LayersPanel`: `bool soloActive_`, `QString soloPath_`,
  `QHash<QString,bool> soloSnapshot_` (path → prior visibility of **every** row,
  captured on enter). It is transient view state: never serialized, never in the
  document.
- **Enter** (first `Alt`-click on a row's eye): snapshot every row's visibility;
  then call `apply_visibility(visiblePaths, "Solo Visibility")` where
  `visiblePaths` = the clicked path **plus its ancestors** (an ancestor group
  must be visible for the child to show) **plus**, when the clicked node is a
  group, the descendants whose snapshotted visibility was `true`. Every other
  node becomes invisible. The snapshot is taken **before** the call.
- **Restore** (any `Alt`-click while `soloActive_`): call
  `apply_visibility(snapshotTruePaths, "Restore Visibility")` and clear the solo
  state. This restores the exact prior per-row visibility, including a prior
  solo or a partial hidden state.
- Each direction is exactly one undo state. Undoing a solo restores all prior
  visibility too.
- **Invalidation:** the snapshot is cleared (and solo deactivated) when the
  active document changes or when a structural op (create/delete/group/ungroup/
  reorder) runs; the panel clears it before dispatching such an op. A path that
  no longer resolves is dropped on restore; if the tree's path set changed
  underneath, solo is cleared first rather than restoring stale paths.

### 3.5 Panel Options (frozen)

A `Panel Options…` dialog from the panel menu, with three controls and these
defaults:

| Control | Type | Default | Options |
|---|---|---|---|
| Thumbnail size | enum | **Medium** (24 px) | None (0) / Small (16) / Medium (24) / Large (32) |
| Thumbnail contents | enum | **Entire Document** | Entire Document / Layer Bounds |
| Expand New Effects | bool | **on** | on / off |

- The thumbnail-size default follows the `LAY-002` "Parameters & ranges" table
  (`medium`), which also preserves the current 24 px thumbnail; `LAY-002`'s Open
  questions lists it as unstated, so this is a recorded decision.
- Thumbnail **contents** is implemented at thumbnail-build time, not by scaling
  a prebuilt image:
  - **Layer Bounds:** the square is the layer's own `rect` content scaled to fit.
  - **Entire Document:** the square represents the whole document; the layer's
    pixels are drawn at their document position (`rect.left/top`, scaled by
    `size / max(doc.width, doc.height)`), so a small layer appears small in its
    document context. A group draws the folder glyph either way.
- `Expand New Effects` is persisted and surfaced now; it has **no visible
  effect until M42** adds effect child rows. It is not omitted just because its
  consumer is later.
- Persistence: the XDG session store (`crates/pictura-app/cpp/session.{h,cpp}`)
  gains `layersThumbSize` (0..3), `layersThumbContents` (0/1),
  `layersExpandNewEffects` (bool) at **schema v3**; a missing field or an older
  schema loads the defaults. (M26 set schema v2 for `gpuCompute`.)
- The CS6 dialog's other two options (`Add "copy" to Copied Layers and Groups`,
  `Use Default Masks on Fill Layers`) are deferred: the first is already the
  hard-coded M37 duplicate-naming behaviour, and the second needs fill layers
  (M43). They are not shown in M39.

### 3.6 Model, delegate, and tree (frozen)

- `LayersModel` is promoted from `QAbstractTableModel` to `QAbstractItemModel`
  with **one visible column** (the CS6 row anatomy). Blend/opacity/fill/lock/
  color stay in the header controls; they are exposed as roles, not columns.
  Rows carry these roles (`LayersModel::Roles`):
  `PathRole`, `DepthRole`, `KindRole`, `VisibleRole`, `BlendRole`,
  `OpacityRole`, `FillRole`, `LockRole`, `ColorRole`, `ClippingRole`,
  `HasMaskRole`, `HasAdjustmentRole`, `ExpandableRole`, `ChildCountRole`,
  `ThumbnailRole`, `MaskThumbnailRole`; plus `DisplayRole`/`EditRole`
  (name), `ToolTipRole`, and `CheckStateRole` (the eye).
- `LayerRowDelegate : QStyledItemDelegate` paints the row and handles the eye
  hit-test: eye checkbox (left), thumbnail or folder glyph, name, color-label
  swatch, clip indent + base underline, mask thumbnail, and an `fx` badge when
  `HasAdjustmentRole` is set. `sizeHint` follows the Panel Options thumbnail
  size. The eye click toggles visibility (`set_layers_visible` with the row's
  selection); `Alt`-click runs solo.
- **Badge fallback:** the disclosure triangle is the `QTreeView` style's own
  branch indicator. The `fx` badge uses `layers.fx`; the clip and mask
  indicators use the delegate's own painting. If an expected asset resolves to a
  null `QIcon`, the delegate omits the badge and the row remains legible; a null
  asset is a build error already guarded by `m38_panels`, so this is a
  belt-and-braces fallback, not a supported state.
- `QTreeView` configuration: `setRootIsDecorated(true)`,
  `setItemsExpandable(true)`, `setExpandsOnDoubleClick(true)`,
  `setUniformRowHeights(true)`, `ExtendedSelection`, `SelectRows`,
  `setDragEnabled(false)` (drag-reorder deferred). Expansion state is a
  `QSet<QString>` of expanded group paths in the panel; default is collapsed,
  a newly created group is expanded, and the set is session-only (not
  serialized).
- Header controls read the **current** row for their displayed value and are
  enabled iff at least one selected row is eligible for that control. Applying
  the control calls the matching `set_layers_*` over the whole selection.

### 3.7 Menus (frozen)

A panel-menu button (`objectName` `layersPanelMenu`, `InstantPopup`) sits at the
right of the panel header. Wired in M39:

- Panel menu: `Panel Options…`; New Layer; New Group; Duplicate Layer(s);
  Delete Layer(s); Group Layers; Ungroup Layers; Move Layer Up; Move Layer Down.
- Row context menu (right-click): Rename; New Layer; New Group; Duplicate
  Layer(s); Delete Layer(s); Group Layers; Ungroup Layers; Move Layer Up; Move
  Layer Down; and the M36 color-label submenu (`Color Label ▸` with the eight
  `None/Red/…/Gray` entries).
- Eye right-click: `Show/Hide This Layer Only` (solo) and `Show/Hide All Layers`.

Omitted (not shown disabled): Merge, Flatten, Blending Options, Layer/Vector
Mask, Clipping Mask, Link, Rasterize, and Select. The full CS6 `Layer` menu
already exposes those as disabled placeholders via `command_tree.cpp`; M39 keeps
the panel menus to what it actually wires, and they land in M41–M43. **No
`Move Up`/`Move Down` text buttons remain in the bottom strip** — reordering is
in the panel and row menus, leaving the strip exactly the CS6 seven.

### 3.8 Tooltips and the bottom strip (frozen)

- Every row's `ToolTipRole` is `"<name> (<kind>)"` with the four kinds above.
- The bottom strip stays the CS6 seven (`layers.link`, `layers.fx`,
  `layers.mask`, `layers.fillAdjustment`, `layers.group`, `layers.newLayer`,
  `layers.delete`); `link`/`fx`/`mask` remain disabled placeholders; `duplicate`
  is reachable from the panel and row menus (there is no CS6 duplicate strip
  button). M38's objectNames (`layersStripLink`…`layersStripDelete`) are
  preserved so `m38_panels` keeps passing.

### 3.9 Drag-reorder — **deferred to M41** (frozen)

M39 ships reordering only through the menu Move Up/Down (`move_layer_path`,
adjacent swap within the container). Native drag-reorder is deferred to M41
because its legality (locked/Background positions, groups accepting children,
no drop into a descendant) is structural management and needs a cross-container
move op that M39 does not add. When it lands, the drop rules are: a locked layer
or the Background cannot be displaced; a group accepts children; a node cannot
be dropped into its own descendant; an illegal target is not highlighted. M39
sets `setDragEnabled(false)` and implements no `dropMimeData`.

## 4. Staged tasks

See `openspec/changes/m39-panel-anatomy/tasks.md`. In outline: (1) brief and
frozen interfaces; (2) the path/tree helpers in `layer_ops` and the bridge tree
API; (3) the tree model, delegate, and badges; (4) multi-selection and the
refusal table; (5) solo, `Tab` rename, and tooltips; (6) Panel Options and the
menus; (7) drag-reorder (explicitly deferred); (8) tests and self-test; (9)
close-out.

## 5. Non-goals

- **Filter/search** (six dimensions, proxy, filter bar) — M40.
- **Remaining management operations** (merge/flatten, rasterize variants, Layer
  Via Copy/Cut, Convert Background, Select All/Similar/Linked, link sets, Delete
  Hidden Layers, the New Layer/Group dialogs) — M41.
- **Layer styles/effects** (`fx` menu, effect rows, the Layer Style dialog,
  rendering, the Blend-If badge) — M42. `Expand New Effects` is persisted but
  inert until then.
- **Smart objects, vector masks, artboards (non-goal), layer comps** — M43.
- A stable layer identity (`lyid`); cross-container drag-reorder; per-document
  expansion persistence; group content thumbnails (groups draw a folder glyph).
- Any `docs/` change beyond this file, `docs/dev/STATE.md`, and the OpenSpec
  change.

## 6. Open items this milestone had to decide

`LAY-002` / `PAN-001` leave these open; this contract chooses:

- **Panel Options thumbnail-size default** — Medium (table) despite the Open
  questions entry saying unstated.
- **Group thumbnail contents** (`LAY-003` Open questions) — a folder glyph in
  M39; a real group composite thumbnail is deferred.
- **Solo undoability** — one undo state per direction.
- **Expansion persistence** — session-only, keyed by path, default collapsed.
- **Drag-reorder** — deferred to M41.
- **Menu placeholder policy** — panel menus show only wired commands; the full
  CS6 set stays in the disabled `Layer` menu.
- **Mixed-selection refusal** — skip ineligible nodes per node; only
  `group_layers` refuses as a whole.
