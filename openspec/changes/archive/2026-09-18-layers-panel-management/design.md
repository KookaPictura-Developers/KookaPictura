## Context

The Layers panel program has landed its panel stages: attributes (M36), creation
and grouping (M37), panel anatomy (M39), filtering/search, controls, chrome, row
interactions, and control polish. The remaining stage is the structural
management set. The `Layer` menu already lists the commands as disabled leaves in
`command_tree.cpp` (`Merge Layers`, `Merge Visible`, `Flatten Image`, `Layer via
Copy/Cut`, `Layer from Background`, `Background From Layer`, `Delete Hidden
Layers`, `Hide Layers`, `Select Similar/Linked`, `Link/Unlink`, the `Rasterize`
submenu), and the document model has no engine for any of them.

Merge/flatten are the only destructive operations in the stage and the rest of
the stage recomputes the composite around them, so they land first. Two
foundations already exist: `pictura-render::composite::composite_rgba` is the
single compositing oracle, and `layer_ops::paths` provides the path grammar,
`resolve_path`, `flatten_rows`, and the batch `edit_paths` helper. The engine
stays UI-free; the bridge resolves the panel's selected paths and owns the
transient link-set and selection state.

## Goals / Non-Goals

**Goals:**

- `merge_scope(doc, MergeScope)` for Merge Down / Selected / Visible / Clipping
  Mask, plus `flatten(doc)`, all reusing `composite_rgba` into a scratch buffer.
- Exactly one undo state per command, produced by the existing
  composite-then-record bridge convention.
- A first-class `Layer` background flag with codec read/write and working
  `Layer from Background…` / `Background From Layer`.
- The New Layer/Group dialog with the mode-neutral fill lookup.
- Layer via Copy/Cut, Select Similar/Linked, link sets, Delete Hidden Layers,
  Hide Layers.
- Rasterize Fill Content and `Layer`/`All Layers`; the other variants stay
  visibly disabled with a stated reason.
- Finish the drag-reorder drop rules: validate before commit, reject invalid
  drops, and move a row out of a group via the empty viewport.

**Non-Goals:**

- Rasterizing Type, Shape, Vector Mask, Smart Object, Video, or 3D layers: those
  kinds do not exist in the model.
- Layer-style/effects rendering and `Rasterize Layer Style` (styles stage).
- Link-set or Background serialization beyond the PSD name/order convention
  (`LAY-002` open question: link-set persistence is unsourced). Link sets are
  transient session state.
- `Merge To HDR Pro`, `Merge Shape Components`, and Stamp (`Ctrl+Alt+E`).

## Decisions

### D1 — Engine API: selection travels in `MergeScope`

`crates/pictura-render/src/document_ops/layer_ops/merge.rs`:

```
pub enum MergeScope<'a> {
    Down(&'a str),
    Selected(&'a [String]),
    Visible(&'a str),
    ClippingMask(&'a str),
}
pub struct MergeOutcome { pub path: String, pub replaced: usize }
pub enum MergeError { NoDocument, NoSelection, NoLayerBelow, InvalidTarget, NotClippable }

pub fn merge_scope(doc: &mut Document, scope: MergeScope<'_>) -> Result<MergeOutcome, MergeError>;
pub fn flatten(doc: &mut Document) -> Result<MergeOutcome, MergeError>;
pub fn can_merge_target(layer: &Layer) -> bool;        // not adjustment/fill content
pub fn is_visible_in_panel(doc: &Document, path: &str) -> bool; // eye + ancestors
```

The variants carry the resolved paths because `pictura-render` cannot read the
app's selection. `merge_scope(doc, MergeScope)` is the literal signature the
contract asks for.

### D2 — Reuse `composite_rgba`; no second compositor

Inputs are cloned into a scratch `Document` whose `layers` are exactly the merge
inputs in stacking order; `composite_rgba(&scratch)` produces the full-document
planar RGBA buffer. The merged node is baked from the union of the inputs'
content rects, cropped out of that buffer. This keeps parity with normal
compositing by construction, exactly as `05-layers/merge-and-flatten.md`
requires. `composite_rgba` ignores the `clipping` flag (the compositor does not
render clipping anywhere yet), so `Merge Clipping Mask` first folds each clipped
sibling's coverage into the base's alpha (intersection) before the scratch
composite; this is a documented gap shared with the compositor, not a merge-only
divergence, and is marked `// ponytail:` naming the upgrade path (teach
`composite_layer` clipping, then delete the fold).

### D3 — Result node and attributes

- One `Pixel` node with channels `0/1/2/-1` sized to the union content rect,
  straight-alpha RGBA from the scratch buffer.
- **Merge Down** replaces the lower + upper pair at the lower index; the result
  inherits the **lower** layer's name, blend, and opacity *(inferred from the
  spec's open question; resolved in favor of the lower node)*.
- **Merge Layers** replaces the selected set at the **topmost** selected
  position and resets blend/opacity to `Normal`/`255` *(inferred)*.
- **Merge Visible** replaces the contiguous visible run at its topmost input
  position, leaving hidden layers (and their indices) in place.
- **Flatten** discards hidden layers, composites all visible layers over an
  opaque white full-document backdrop, and replaces the whole layer list with a
  single `Layer` whose `background = true`, `Normal` blend, opacity/fill 255,
  full-document rect, and no alpha below 255. No confirmation prompt.
- Parent/sibling bookkeeping uses the existing path/container helpers; a merge
  inside a group stays inside that group.

### D4 — Selection-dependent `Ctrl+E`

One bridge command resolves the panel's selected paths and calls
`merge_scope` with `Down(path)` when exactly one layer is selected and
`Selected(paths)` when more than one is. `Merge Down`/`Merge Layers` therefore
share one command handler and one id (`layer.merge.layers`), matching CS6.

### D5 — Background as a first-class flag

`pictura_core::Layer` gains `pub background: bool` (default `false`), updated at
every clone/literal site. `is_background(doc, path)` reads the flag instead of
the index-0 + `name == "Background"` heuristic. The codec derives the flag on
read for the bottom top-level non-group layer named `Background` and writes the
`Background` name for a flagged layer (PSD has no background bit; the convention
is the name plus position — marked inferred). `Layer from Background…` clears
the flag and unlocks; `Background From Layer` sets it, converts transparent
pixels to the background color, and moves the node to the bottom.

### D6 — New Layer/Group dialog

A modal `LayerNewDialog` collects Name, Color label, blend Mode, Opacity,
Fill-with-mode-neutral-color, and Use-Previous-Layer-to-Create-Clipping-Mask
(clipping hidden for groups). The neutral color is a lookup by mode from
`docs/05-layers/blend-modes.md`: white for Darken/Multiply/Color
Burn/Linear Burn/Darker Color/Divide, black for Lighten/Screen/Color
Dodge/Linear Dodge/Lighter Color/Difference/Exclusion/Subtract, 50 % gray for
Overlay/Soft Light/Hard Light/Vivid Light/Linear Light/Pin Light, transparent
for Normal/Dissolve/Hard Mix/Hue/Saturation/Color/Luminosity, and the
documented default for any unlisted mode is **transparent** *(inferred)*. The
engine's `add_layer_in`/`add_group_in` grow an attribute parameter
(`NewLayerSpec`); the fill is written as a solid pixel layer at the mode-neutral
RGBA, not as a fill-content layer.

### D7 — Bridge, commands, and enablement

New `#[qinvokable]` methods in `impl_layers.rs`, declared in `cxxqt_object.rs`,
each recompositing then recording exactly one undo state:

`merge_layers(paths)`, `merge_visible(path)`, `merge_clipping_mask(path)`,
`flatten_image()`, `new_layer_dialog(...)` / `new_group_dialog(...)`,
`layer_from_background(path)`, `background_from_layer(path)`,
`layer_via_copy()`, `layer_via_cut()`, `select_similar(path) -> QStringList`,
`select_linked(path) -> QStringList`, `link_layers(paths, on)`,
`hide_layers(paths)`, `delete_hidden_layers()`, `rasterize_fill_content(path)`.

New ids in `commands.h` (`layer.merge.layers`, `layer.merge.visible`,
`layer.merge.clippingMask`, `layer.flatten.image`, `layer.new.layerFromBackground`,
`layer.new.backgroundFromLayer`, `layer.new.layerViaCopy`,
`layer.new.layerViaCut`, `layer.new.groupFromLayers`,
`layer.delete.hiddenLayers`, `layer.select.similar`, `layer.select.linked`,
`layer.link.layers`, `layer.unlink.layers`, `layer.hide.layers`,
`layer.rasterize.fillContent`, `layer.rasterize.layer`,
`layer.rasterize.allLayers`). `command_tree.cpp` wires the existing disabled
leaves to these ids; `frame_menus.cpp` registers a handler and an enable
provider for each. Enablement: merge/visible require a document and an eligible
selection; Flatten requires a document; Copy/Cut require an active selection;
Background conversions require the applicable kind; the disabled Rasterize
variants get no handler and stay disabled.

### D8 — Rasterize subset

`Rasterize Fill Content` treats a fill-content layer as one whose opaque
adjustment key is a fill key (`SoCo` solid, `GdFl` gradient, `PtFl` pattern)
with a decodable payload: it renders the decoded content to a full-layer pixel
node and clears the adjustment data. `Rasterize Layer` rasterizes the active
layer only when it is such a fill-content layer, and refuses everything else
(including groups and plain adjustments — CS6 does not collapse a group through
Rasterize). `Rasterize All Layers` rasterizes every fill-content layer. A fill
block whose payload is not decodable leaves the command disabled (documented,
`ponytail:` the descriptor decoders are the ceiling). Type/Shape/Vector
Mask/Smart Object/Video/3D are disabled because the kinds are absent.

### D9 — Link sets are transient

The bridge owns a `HashMap<String, u32>` of path → link-set id. `link_layers`
assigns a fresh id to a path absent from any set and merges sets when members
already belong to one; `unlink_layers` removes members and drops an emptied set.
`select_linked(path)` returns the active layer's set members. The map is never
serialized and a structural operation clears it: this is the documented
limitation the proposal calls out (`LAY-002` open question).

### D10 — Drag-reorder drop rules

`LayersTreeView` validates each candidate drop during `dragMoveEvent` with the
engine's `move_path_to` dry-run/refusal predicate, highlights only valid
targets, and rejects the rest without calling the bridge. Dropping a row on the
empty viewport below the last row moves it to the document root (out of its
group). Every committed drop is one undoable `"Move Layer"` step, unchanged from
`row-interactions`.

### D11 — Verification

- Rust unit tests in `merge.rs`/`layer_ops/tests.rs` for the resulting node
  structure (count, order, name/blend/opacity, rect) and for pixel equality with
  `composite_rgba` of the inputs.
- A fill-content rasterize test and Background round-trip test; codec read/write
  covered in `pictura-codec`.
- At least one external-oracle composite test (psd-tools / ImageMagick) that
  self-skips when the tool is absent, comparing the merged pixels.
- App self-tests for each command: merge down/selected/visible/clipping, flatten
  white backdrop and hidden-layer discard, dialog, background convert, copy/cut,
  select similar/linked, delete hidden/hide, rasterize fill, and drop-out rule.

## Risks / Trade-offs

- **Background heuristic change** touches every `Layer` literal; the codec
  round-trip and the existing `is_background_uses_default_heuristic` test pin
  the behavior, and the golden baseline may need an explicit note.
- **Merge Down onto a Background** is allowed (normal layer above it); its
  result name/mode is inferred as the lower (Background) layer's, which is
  consistent with D3 but unverified against CS6.
- **Clipping coverage** is a merge-local fold because the compositor has no
  clipping pass; if the compositor later gains clipping, the fold must be
  deleted so the two agree.
- **Fill-content decode** is bounded by the descriptor decoders; solid color is
  the safe case and gradient/pattern degrade to disabled rather than wrong
  pixels.
- **Link-set path keys** are positional and the map is cleared by structural
  operations; no stable layer id exists (M39 note). Acceptable while
  serialization is out of scope.
- **Scratch full-document composite** allocates one document-sized buffer per
  merge; the existing `composite_rgba` ceiling (one full f32 buffer) already
  applies and is marked there.

## Residual inferred behavior and documented ceilings

Independent verification confirmed the requirements and recorded these
deliberate, documented deviations. None is a spec violation; each is either
inferred behavior or a known ceiling with an upgrade path.

- **rename ↔ background.** `rename_path` sets/clears `Layer.background` from the
  same name + bottom-top-level-non-group convention the codec uses on read, so
  an in-session rename stays consistent with a save/reopen. This is an
  in-session consistency rule for the codec's derived flag, not a new
  requirement.
- **Clipping coverage.** Only `Merge Clipping Mask` folds clipping coverage into
  the base alpha; the compositor has no clipping pass, so Merge Down / Layers /
  Visible share that gap instead of diverging. The spec wording reflects this.
- **Zero-layer Flatten.** `flatten` on an empty tree yields a white Background
  but reports `replaced = 0`, so the bridge records no history. Unreachable
  through the UI (no empty document is creatable); documented rather than
  churned.
- **`select_similar` excludes the active layer**, matching this design's stated
  match-key rule; CS6 keeps the active layer selected. The panel does not re-add
  it.
- **Fixed colors.** `background_from_layer` fills with opaque white and the
  panel's `Solid Color…` fill is opaque black because no Rust-facing toolbox
  background/foreground accessor exists; both carry `ponytail:` comments naming
  the upgrade.
- **Panel context menu enablement.** The Layers panel widget menu enables the
  merge rows when any layer is selected; the `Layer` menu uses the
  `can_merge_*` predicates. The panel menu has no per-row predicate hook yet.
- **`SoCo` decoding.** A 4-byte `SoCo` payload is our solid-fill encoding; a real
  Photoshop `'Clr '` descriptor is preserved verbatim on disk but not decoded
  (its fill is not rendered), and gradient/pattern fills remain unsupported.
