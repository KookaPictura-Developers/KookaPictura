## Why

The Layers panel is a flat table over top-level layers only. The document model
is already a tree (`Layer.children`, `is_group`, `clipping`), but the bridge
(`crates/pictura-app/src/cxxqt_object.rs`) exposes every `layer_*(i: i32)`
method as `doc.layers[i]`, so a group's children are invisible and unaddressable.
The panel therefore lacks CS6's row anatomy (groups, clipped-layer indentation,
mask thumbnails, fx/clip badges), multi-selection, solo visibility, inline
`Tab` rename, Panel Options, and panel/row menus — all of which `LAY-002` and
`PAN-001` require but the M20 panel never had. This is the next milestone of the
layers-panel program (`docs/dev/layers-panel-program.md`); filtering/search,
the remaining management operations, styles, and smart objects follow as
M40–M43.

## What Changes

- **Frozen layer-path grammar.** A path is a `/`-separated list of bottom-first
  child indices (`"0"`, `"2/1"`, `"0/2/1"`); positional, invalidated by any
  structural change, refused rather than panicked on out-of-range segments.
- **Bridge tree API.** The bridge projects the whole tree as a depth-first,
  topmost-first row list (`layer_row_count` + `layer_row_*` getters for path,
  depth, name, kind, visibility, blend, opacity, fill, lock, color, clipping,
  has-mask, has-adjustment, expandable, child count, thumbnail, mask
  thumbnail). The existing top-level `layer_*(i)` methods are **kept as
  wrappers** over path `"<i>"`, so no existing caller or self-test changes.
- **Path/batch mutation.** Single `set_layer_name_path` / `move_layer_path`,
  and `QStringList`-batch `set_layers_visible/blend/opacity/fill/lock/color`,
  `apply_visibility`, `delete_layers`, `duplicate_layers`, `group_layers`,
  `ungroup_layers`, each a single undo step over the selection. Tree-aware
  creation `add_layer_in` / `add_group_in`.
- **Tree model and delegate.** `LayersModel` becomes a `QAbstractItemModel`
  with one visible column and a role per row value; a `LayerRowDelegate` paints
  the eye, thumbnail/folder glyph, name, color swatch, clip indent + underline,
  mask thumbnail, and `fx` badge. Groups expand/collapse; extended selection.
- **Multi-selection refusals.** A frozen per-node skip table (Background, fully
  locked, group-fill) and whole-op refusal for `group_layers`; this lifts M37's
  single-layer Group/Duplicate limit.
- **Solo visibility.** `Alt`-click an eye snapshots every row's visibility and
  applies "only this layer/group"; a second `Alt`-click restores the snapshot
  exactly. One undo state per direction; state lives in the panel.
- **Inline rename with `Tab`.** Double-click to edit; `Tab`/`Shift`+`Tab`
  commits and moves to the next/previous visible row.
- **Panel Options.** A dialog with thumbnail size (None/Small/Medium/Large),
  thumbnail contents (Entire Document / Layer Bounds), and `Expand New Effects`,
  persisted in the XDG session store (schema v3).
- **Menus and tooltips.** A panel menu (`Panel Options…`, New/Duplicate/Delete,
  Group/Ungroup, Move Up/Down) and a row context menu (plus the M36 color-label
  submenu); Move Up/Down replace the two text buttons so the strip is exactly the
  CS6 seven; row tooltips are `"<name> (<kind>)"`.
- **Drag-reorder is deferred to M41**; reordering is menu-only in M39.

## Capabilities

### New Capabilities

None. This is panel view/controller work over the existing document model; no
file-format, compositor, or persistence capability is added (Panel Options is
session UI state, not a document capability).

### Modified Capabilities

- `layers-panel`: rows become the full expandable layer tree (groups, clipped
  indentation, mask/fx badges, child counts); property edits and the M37
  operations become selection-based, path-addressed, and one undo step over N
  rows; new requirements add the path grammar, multi-selection refusals, solo
  visibility, `Tab` rename, Panel Options, menus, tooltips, and badges.
- `application-shell` and `tool-framework` are **not** touched: the panel menu
  lives inside the panel, the `Window > Panels` toggle is unchanged, and no tool
  or command-registry behaviour changes.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops.rs` (+ `mod.rs`, `lib.rs`) —
  path resolve/flatten helpers and the batch ops, with unit tests.
- `crates/pictura-app/src/cxxqt_object.rs` — the tree-row getters, path/batch
  mutators, `add_layer_in`/`add_group_in`, and the top-level compatibility
  wrappers.
- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}` — tree model, delegate,
  selection model, solo/rename handling, Panel Options dialog, panel and row
  menus, header enablement.
- `crates/pictura-app/cpp/session.{h,cpp}` — `layersThumbSize`,
  `layersThumbContents`, `layersExpandNewEffects` at schema v3.
- `crates/pictura-app/cpp/frame.cpp` — the Layers `Layer`-menu handlers move to
  the selection-aware bridge batch (removing the M37 single-layer
  `// ponytail:` note).
- `crates/pictura-app/cpp/main.cpp` — the `m39_*` self-test steps with fresh
  exit codes.
- `docs/dev/m39-panel-anatomy.md`, `docs/dev/STATE.md` — the brief and the
  program status.
- No new dependency, no PSD format change, no compositor change. Every mutation
  keeps the M34 composite-then-record convention.
