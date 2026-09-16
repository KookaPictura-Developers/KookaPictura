## Context

M20 built the Layers panel as a flat table over the top-level layer list
(`crates/pictura-app/cpp/panels/layers_panel.cpp`): the four bottom buttons are
Add Adjustment / Delete Layer / Move Up / Move Down. M36 added Fill/lock/color
editing. The bridge (`crates/pictura-app/src/cxxqt_object.rs`) exposes no
creation method, and `crates/pictura-render/src/document_ops/` has only
canvas/crop/orient/resize transform ops. `Document::layers` is bottom-first
(`crates/pictura-core/src/lib.rs:73`); a pixel layer carries channels `0,1,2`
and a transparency channel `-1`, and the compositor treats source alpha `0` as
inert (`blend_into` returns early, `crates/pictura-render/src/lib.rs:424`).
`new_document` seeds one `"Layer 0"`.

`docs/05-layers/layer-management-ui.md` (`LAY-002`) is the contract. The CS6
facts the design depends on: New Layer / New Group appear in the bottom strip
before Delete; `Ctrl/Cmd+J` duplicates; duplicated layers are named
`"<name> copy"` (Panel Options "Add copy to Copied Layers and Groups"); groups
carry child layers and can be ungrouped back into the parent.

Constraints: the CPU compositor is the frozen oracle and PSD I/O is untouched
(a transparent layer contributes nothing, so no compositor requirement
changes), no new dependency, and `cxx-qt-lib` carries only scalar bridge types
(so indices/lengths cross as `i32`).

## Goals / Non-Goals

**Goals:**

- Five pure `document_ops::layer_ops` functions over `&mut Document`:
  `add_layer`, `add_group`, `duplicate_layer`, `group_layer`, `ungroup_layer`,
  plus a deterministic `next_layer_name`.
- Expose them through the bridge, each recompositing then recording exactly one
  labelled undo state.
- Add New Group / New Layer to the panel strip (before Delete) and wire the
  five `Layer` menu leaves to handlers acting on the current layer.
- Ship runnable checks: unit tests per op (including the inert transparent
  layer) and an `m37_create` app self-test step.

**Non-Goals:**

- A New Layer / New Group **dialog** (neutral-color fill, "use previous layer as
  clipping mask", blend/opacity choice). The session self-test and shell use the
  defaults; the dialog is deferred with the rest of the panel chrome.
- **Multi-selection** grouping (`Group Layers` over several rows) and
  drag-reorder; M38's tree/selection work upgrades these.
- Merge, flatten, rasterize, layer-via-copy/cut, select-similar, convert
  background (M40), styles/effects (M41), smart objects / vector masks /
  artboards / comps (M42).
- A first-class Background layer kind and the forced type/shape locks.
- Any PSD `write_psd` change: the new layer is an in-memory document node; if
  the document is saved, the existing writer serializes it unchanged.

## Decisions

### 1. Ops are pure functions over `&mut Document` (frozen)

```rust
pub fn add_layer(doc: &mut Document, above: i32, name: &str) -> i32;   // new index, -1 on failure
pub fn add_group(doc: &mut Document, above: i32, name: &str) -> i32;   // new index
pub fn duplicate_layer(doc: &mut Document, index: i32) -> i32;         // new index, -1 OOR
pub fn group_layer(doc: &mut Document, index: i32) -> i32;             // group index, -1 OOR
pub fn ungroup_layer(doc: &mut Document, index: i32) -> bool;          // false for non-group
pub fn next_layer_name(doc: &Document, prefix: &str) -> String;        // "Prefix N"
```

They live in `crates/pictura-render/src/document_ops/layer_ops.rs` and are
re-exported from `document_ops::{...}` and the crate root, matching the existing
`translate_layer`/`rotate_document` convention.

### 2. Insertion-order semantics (frozen)

`Document::layers` is bottom-first, so inserting "directly above `above`" means
index `above + 1`. The index is clamped to the stack: a negative sentinel (no
selection) or an out-of-range index places the new node at the **top**
(`len`). `above == len - 1` also yields the top.

```
insertion_index(len, above) = if above < 0 { len } else { (above as usize + 1).min(len) }
```

- The panel passes the selected row's bottom-first index, or `-1` when nothing
  is selected; a New Layer with no selection therefore lands on top, matching
  CS6.
- `group_layer` and `ungroup_layer` do not insert relative to `above`: they
  operate at the target's own slot so the stack order is preserved. `group_layer`
  removes the layer and re-inserts the wrapping group at the same index;
  `ungroup_layer` removes the group and splices its children starting at that
  index, preserving child order.

### 3. Empty-layer representation (frozen, with a ceiling)

```rust
// add_layer: document-sized, fully transparent
Layer {
    name, rect: { 0, 0, h, w },
    blend: Normal, opacity: 255, fill: 255,
    lock: default, color: None, clipping: false, visible: true,
    mask: None, adjustment: None,
    channels: [0, 1, 2, -1] each w*h zero bytes,
    children: [], is_group: false,
}
```

- Photoshop stores an empty layer until it is painted; this repo stores a
  document-sized zeroed layer as the lazy stand-in. A `// ponytail:` comment in
  the code names the empty-rect upgrade and its `w*h*4`-byte cost.
- A fully transparent layer is inert: `blend_into` returns before touching the
  canvas when `src_a <= 0`, so the composite is byte-identical with and without
  it. This is asserted by a unit test and by the app self-test.
- `add_group` inserts an empty `is_group` node: `Normal` blend (not
  `PassThrough`, so the group composites its children as a unit once populated),
  empty `{0,0,0,0}` rectangle, no children, no channels.
- A new layer is document-sized even for a grayscale document (channels
  `0/1/2/-1`), matching the task contract; the extra planes are inert and the
  compositor reads channel `0` for gray.

### 4. Naming (frozen)

`next_layer_name(doc, prefix)` scans the whole layer tree (children included)
for names of the form `"<prefix> <number>"` and returns `"<prefix> <max+1>"`,
or `"<prefix> 1"` when none exist. It is deterministic and ignores
`"<prefix> copy"` names. `duplicate_layer` names the copy `"<name> copy"`
(Photoshop's Panel Options default). `group_layer` names its group via
`next_layer_name(doc, "Group")`.

### 5. Undo and the composite-then-record convention (frozen)

Every bridge method mutates the document, then calls `recomposite()` and then
`record(<label>)`, mirroring `set_layer_name`/`remove_layer`
(`crates/pictura-app/src/cxxqt_object.rs`). Labels: `New Layer`, `New Group`,
`Duplicate Layer`, `Group Layers`, `Ungroup Layers`. Because `record` snapshots
after the recomposite, an undo restores the pre-op document exactly and one
operation is exactly one history step.

### 6. Bridge API (frozen)

```rust
#[qinvokable] fn add_layer(self: Pin<&mut Self>, above: i32) -> i32;
#[qinvokable] fn add_group(self: Pin<&mut Self>, above: i32) -> i32;
#[qinvokable] fn duplicate_layer(self: Pin<&mut Self>, index: i32) -> i32;
#[qinvokable] fn group_layer(self: Pin<&mut Self>, index: i32) -> i32;
#[qinvokable] fn ungroup_layer(self: Pin<&mut Self>, index: i32) -> bool;
```

`add_layer`/`add_group` compute the name with `next_layer_name` and return the
new index; `-1` means no document. `duplicate_layer`/`group_layer` return `-1`
when the index is out of range; `ungroup_layer` returns `false` for a non-group.
A failed op records no history and emits no `changed`.

### 7. Menu wiring (frozen)

The five Layer leaves are frozen to explicit ids in `command_ids`
(`layer.new.layer`, `layer.new.group`, `layer.duplicate.layer`,
`layer.group.layers`, `layer.ungroup.layers` — identical to the previous
path-derived ids, so no behaviour or id changes elsewhere) and registered with
`implemented = true`. `frame.cpp` handles each against the active view's current
layer, read from the Layers panel's selection via
`LayersPanel::currentLayer()`. `// ponytail:` at the handler block names the
M38 multi-selection upgrade.

### 8. Panel (frozen)

Two `QPushButton`s — **New Group** then **New Layer** — are added before
**Delete Layer**, giving Add Adjustment / New Group / New Layer / Delete / Move
Up / Move Down (CS6's group-before-layer order). Clicking reads the selected
row's layer index (`-1` when none), calls the bridge, and on success refreshes
and selects the returned index via a new `selectLayer(int)`. `refresh()` reuses
`currentLayer()`/`selectLayer()` so the existing selection-restore path is
unchanged.

### 9. Tests and checks (frozen)

- `pictura-render` unit tests: insertion index/order for `add_layer`/`add_group`
  (including the no-selection top case); a new transparent layer's channel
  layout and metadata; `duplicate_layer` preserves children, mask, adjustment
  attributes, and is a deep copy; `group_layer` wraps in place; `ungroup_layer`
  splices children in order and refuses a non-group; `next_layer_name` uses the
  highest suffix; and a transparent layer leaves `composite_rgba(doc)`
  unchanged.
- App self-test (`main.cpp`) `m37_create`: count growth, an unchanged composite
  on layer creation, a `" copy"` duplicate name, wrap/unwrap, five history
  steps, and undo; print
  `pictura self-test: m37_create new=1 group=1 duplicate=1 ungroup=1 undo=1`
  and fail with exit codes 90–94.

## Risks / Trade-offs

- **Empty layers are document-sized.** A new layer costs `w*h*4` bytes even
  before it is painted; Photoshop stores nothing until a dab. The `// ponytail:`
  in `layer_ops.rs` names the empty-rect upgrade. No behavioural risk — the
  pixels are zero and the compositor skips them.
- **No multi-selection.** `Group Layers` / `Ungroup Layers` act on one layer.
  CS6 can group several; M38's selection work is the upgrade path, recorded in
  the handler comment and `tasks.md`.
- **No New Layer dialog.** The Create New Layer options (name, color, blend,
  opacity, clipping) are not surfaced; the defaults match a fresh CS6 layer's
  unedited state. Dialog is deferred.
- **Group blend is `Normal`, not `PassThrough`.** An empty group is inert
  either way; once children land the group isolates them, which is CS6's default
  for a newly created group only after it has content. If pass-through parity is
  wanted, `LAY-003` is the reference; the value round-trips through PSD's `lsct`.
- **`above` clamping.** The spec chooses "out of range means top", including the
  `-1` sentinel; a caller that wants "insert at the very bottom" must pass a
  valid index. Deterministic and documented.

## Migration Plan

Additive: five new functions, five new bridge methods, two buttons, and five
formerly-placeholder menu leaves. No model, codec, or compositor change, so no
document changes bytes and no on-disk migration is needed. Rollback: delete the
module, the bridge methods, the buttons, and revert the five command ids to
`leaf(...)` placeholders.

## Open Questions

- **Group default blend.** Whether a newly created empty CS6 group is `Pass
  Through` on disk before it has children is not settled here; the in-memory
  node uses `Normal` and the choice is invisible until the group has content.
- **Duplicate naming collision.** CS6 may append a number when `"<name> copy"`
  already exists (Panel Options). This change always produces `"<name> copy"`;
  collisions are accepted until a CS6 baseline is available.
