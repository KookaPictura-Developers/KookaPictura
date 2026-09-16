## Why

The Layers panel cannot create a layer. CS6's most basic panel action — the
**New Layer** button — and its companions **New Group** and **Duplicate Layer**
do not exist anywhere: `pictura-render` has no layer-creation function, the
bridge exposes no creation method, and the `Layer` menu leaves (`Layer > New >
Layer`, `Layer > New > Group…`, `Layer > Duplicate Layer…`, `Layer > Group
Layers`, `Layer > Ungroup Layers`) are unimplemented placeholders whose only
identity is the path-derived id from `command_tree.cpp`'s `idFor`. A user can
edit an existing layer but can never add one, so the panel is usable only for
documents that already carry a stack.

This is the next milestone of the layers-panel program
(`docs/dev/layers-panel-program.md`). Creation and grouping are the foundation
the later panel work (tree rows, multi-selection, filtering, management
operations) builds on, so they land first as **M37 — layer creation and
grouping**, and the program's remaining stages shift by one (panel anatomy
M38, filtering/search M39, remaining management ops M40, styles/effects M41,
smart objects / vector masks / artboards / comps M42).

## What Changes

- **Operations (`pictura-render::document_ops::layer_ops`).** Five pure
  functions over `&mut Document`, keeping the bottom-first stack convention:
  `add_layer(doc, above, name) -> i32`, `add_group(doc, above, name) -> i32`,
  `duplicate_layer(doc, index) -> i32`, `group_layer(doc, index) -> i32`, and
  `ungroup_layer(doc, index) -> bool`, plus `next_layer_name(doc, prefix)` for
  deterministic `Layer N` / `Group N` naming.
- **Empty-layer representation.** A new layer is a document-sized raster layer
  with channels `0/1/2/-1` of `w*h` zero bytes, `Normal` blend, opacity 255,
  fill 255, visible. A fully transparent layer MUST NOT change the composite.
  A new group is an empty (`is_group`) node with a `Normal` blend and an empty
  rectangle.
- **Insertion order.** `above` is a bottom-first index; the new node goes at
  `above + 1`. A negative sentinel (no selection) or an out-of-range value puts
  the node at the top of the stack.
- **Bridge (`PictureView`).** `add_layer(above) -> i32`, `add_group(above) ->
  i32`, `duplicate_layer(index) -> i32`, `group_layer(index) -> i32`, and
  `ungroup_layer(index) -> bool`, each `#[qinvokable]`, recompositing and then
  recording one labelled undo state (`New Layer`, `New Group`, `Duplicate
  Layer`, `Group Layers`, `Ungroup Layers`) per the M34 composite-then-record
  convention.
- **Panel.** Two new buttons — **New Group** then **New Layer** — before
  **Delete Layer**; each passes the selected row's layer index and selects the
  returned one. `LayersPanel::currentLayer()` exposes the selection.
- **Menu.** The five `Layer` leaves are frozen to explicit `command_ids`
  (`layer.new.layer`, `layer.new.group`, `layer.duplicate.layer`,
  `layer.group.layers`, `layer.ungroup.layers` — the same strings the
  path-derived ids produced) and handled in `frame.cpp` against the active
  view's current layer.
- **Self-test.** A new `m37_create` step that proves count growth, an inert
  transparent layer, deep duplication naming, wrap/unwrap ordering, one history
  step per op, and undo, with fresh exit codes 90–94.

## Capabilities

### New Capabilities

None. Creation and grouping are panel behaviours over the existing document
model; no file-format or compositor capability changes.

### Modified Capabilities

- `layers-panel`: the panel SHALL offer New Layer, New Group, and Duplicate
  Layer in addition to Add Adjustment / Delete / Move, each applied through the
  bridge as one undoable step, with the rows updating afterwards. A new layer
  SHALL be an empty transparent layer at document size that does not change the
  composite; a duplicate SHALL be a deep copy named `"<name> copy"` placed
  directly above its source; and Group Layers / Ungroup Layers SHALL wrap and
  unwrap the selected layer in place.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops.rs` — the five ops and the
  naming helper, with unit tests.
- `crates/pictura-render/src/document_ops/mod.rs` and `src/lib.rs` — export the
  new functions.
- `crates/pictura-app/src/cxxqt_object.rs` — five `#[qinvokable]` methods and
  their qobject declarations.
- `crates/pictura-app/cpp/commands.h` — five frozen layer command ids.
- `crates/pictura-app/cpp/command_tree.cpp` — the five Layer leaves become
  implemented commands; no other id is renamed.
- `crates/pictura-app/cpp/frame.cpp` — the five Layer handlers.
- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}` — the two buttons and
  `currentLayer()`.
- `crates/pictura-app/cpp/main.cpp` — the `m37_create` self-test step.
- `docs/dev/layers-panel-program.md`, `docs/dev/STATE.md` — renumber the program
  to M36–M42 and record M37.
- No new dependency. PSD I/O and compositing are unchanged: a transparent layer
  already contributes nothing, and no new document is serialized differently.
