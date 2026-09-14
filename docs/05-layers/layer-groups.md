# Layer Groups

- **Spec ID:** `LAY-003`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — group creation, nesting, Pass Through, masks, and composition are CS5-era. CS6 adds the panel's filter/search over group rows, right-click **color labels** on groups, Properties-panel group editing, `Group Properties` handling, and `Ctrl/Cmd+J` duplicating a selected group. Group **Fill** remains unavailable (Opacity only).
- **Depends on:** `LAY-001`, `LAY-002`, `LAY-004`, `LAY-005`, `01-architecture/document-model.md` (`ARCH-008`), `05-layers/blend-modes.md`, `05-layers/layer-styles.md`.

> Module and type names are design proposals. Behavior from the CS6 Help
> reference (cited); inferred items are marked.

## CS6 behavior

A **layer group** is a folder of layers. Groups "help you organize and manage
layers," arrange them in a logical order, and reduce panel clutter. Groups can be
**nested within other groups**, and can "apply attributes and masks to multiple
layers simultaneously." Group attributes are name, color label, blend mode,
opacity, and masks.

### Creating, nesting, adding, ungrouping

- **Group a selection:** select multiple layers, then `Layer > Group Layers`, or
  Alt/Option-drag layers onto the folder icon. `Layer > Ungroup Layers` dissolves
  the group.
- **New group:** New Group button, `Layer > New > Group`, panel menu, or
  `Layer > New > Group From Layers` (groups the selected layers; used to confine
  an adjustment to specific layers — see below). `Shift`-click New Group adds the
  current selection to a new group; `Alt`-click New Group opens the New Group
  dialog.
- **Add to a group:** select the group and click Create a New Layer; drag a layer
  onto the group folder; drag a group folder into another group folder (the
  whole group moves); drag an existing group onto the New Group button.
- **View inside a group:** click the triangle left of the folder; right-click /
  Control-click the triangle for `Open This Group`; Alt/Option-click the triangle
  opens/closes the group and all nested groups.
- **Select a layer in a group:** click the group, open it, then click the layer.
- **Move a layer into a group by dragging:** if the group is closed, the layer is
  placed at the bottom of the group.

### Group blend mode: Pass Through vs Normal (isolation)

- By default a group's blend mode is **Pass Through**: "the group has no blending
  properties of its own." Children keep blending against the backdrop outside
  the group.
- Choosing any mode **other than Pass Through** effectively changes the order of
  assembly: "All of the layers in the group are put together first. The composite
  group is then treated as a single image and blended with the rest of the image
  using the selected blending mode."
- Consequently, with a non-Pass-Through group, "none of the adjustment layers or
  layer blending modes inside the group will apply to layers outside the group."
  This is the documented **isolation** behavior.
- There is no Clear blending mode for layers; groups expose Pass Through in
  addition to the 27 layer modes.

### Group opacity and fill

- Select one or more layers or groups and edit **Opacity** and **Fill**. **"If
  you selected a group, only Opacity is available."** A group has no Fill
  control.
- Overall opacity "determines to what degree it obscures or reveals the layer
  beneath it."

### Group masks

- A mask can be added to "the layer or group" — both the layer-mask path
  (`Layer > Layer Mask > Reveal All / Hide All / Reveal Selection / Hide
  Selection / From Transparency`) and the vector-mask path apply to groups. The
  mask then affects the group's composited result. See `LAY-004` / `LAY-005`.
- By default a layer or group is **linked** to its layer/vector mask; unlink via
  the link icon to move them independently.
- To confine an adjustment to specific layers, group those layers, "change the
  Mode from Pass Through to any other blending mode," then place the adjustment
  layer at the top of the group. The non-Pass-Through group isolates the
  adjustment so it no longer affects content outside the group.

### Groups and knockouts

Knockout options make a layer "punch through" to reveal content from other
layers. For group-related setups the Help prescribes:

- **Reveal a layer above the background:** place the layers to punch through in a
  **group**; the top layer in the group then punches through the grouped layers
  to the next layer **below the group**.
- **Reveal the base of a clipping mask:** place the layers in a clipping mask and
  ensure **Blend Clipped Layers As Group** is selected for the base layer.
- **Shallow** knocks out to "the first possible stopping point, such as the first
  layer after the layer group or the base layer of the clipping mask"; **Deep**
  knocks out to the background (or transparency if there is no background).
- With no group or clipping mask, either option reveals the background layer (or
  transparency if the bottom layer is not a background).
- The knockout is shown by lowering fill opacity or changing the blend mode.

### Clipping into groups

- Clipping masks use the content of a **base layer** to mask the layers above.
  Clipped layers must be **successive**; the base layer's name is underlined and
  overlying thumbnails are indented with a clipping-mask icon.
- New layers created between clipped layers, or unclipped layers dragged in,
  **become part of the clipping mask**.
- Layers in a clipping mask are assigned the **opacity and mode attributes of
  the base layer** by default; **Blend Clipped Layers As Group** (advanced
  blending on the base) decides whether the base's blend mode applies to all
  clipped layers or only to the base.
- `Layer > Create Clipping Mask` / `Release Clipping Mask`, or Alt/Option-click
  the dividing line. `Layer > Merge Clipping Mask` (base must be a raster layer)
  merges a clipping mask. Full clipping semantics are in `LAY-005`.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Layers panel group row | Dock | `F7` | Folder icon, expand triangle, group name, blend/opacity, group thumbnail, mask thumbnails, link icon. |
| `Layer > Group Layers` | Menu | `Ctrl+G` | Groups selected layers. |
| `Layer > Ungroup Layers` | Menu | `Ctrl+Shift+G` | Dissolves group, keeping children. |
| `Layer > New > Group` | Menu | n/a | New empty group (Alt-click for dialog). |
| `Layer > New > Group From Layers` | Menu | n/a | Groups selection; used with mode change to confine adjustments. |
| `Layer > Layer Mask > …` | Menu | n/a | Applies masks to the selected group. |
| `Layer > Vector Mask > …` | Menu | n/a | Applies a vector mask to the selected group. |
| Group Properties dialog | Dialog | double-click group | Name, color, blend mode, opacity. |
| Row context menu | Context menu | right-click | (CS6) color label; Open This Group; properties. |
| New Group button | Panel button | n/a | `Shift`-click adds selection; `Alt`-click opens dialog; `Ctrl`-click creates below. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Group name | string | "Group N" | — | Rename inline / Group Properties. |
| Blend mode | enum | **Pass Through** | Pass Through + 27 layer modes | Non-Pass-Through isolates children. |
| Opacity | percent | 100 | 0–100 | Group opacity. |
| Fill | — | n/a | **unavailable for groups** | Documented: only Opacity. |
| Color label | enum | none | palette | CS6 right-click. |
| Layer mask | ref | none | Reveal/Hide All / Reveal/Hide Selection / From Transparency | Affects group result. |
| Vector mask | ref | none | path | Density/Feather apply. |
| Linked to mask | bool | on | on / off | Unlink to move mask independently. |
| Clipping (as base/clipped) | enum | none | none / base / clipped | Clipped group participates in clipping. |
| Knockout | enum | None | None / Shallow / Deep | On a child to punch through the group. |

## Algorithms & pipeline

1. **Container walk.** The compositor recurses into groups. Display order is
   top-first; compositing of children runs bottom-to-top into the group result.
2. **Pass Through.** With `pass_through == true`, no intermediate buffer is
   allocated: each child blends directly against the running parent backdrop, so
   child blend modes and adjustments see content outside the group.
3. **Isolated group.** With any other mode, children composite into an **isolated
   group buffer** (transparent initial backdrop), then the buffer is composited
   into the parent using the group's blend mode and opacity. The Help's statement
   that internal adjustments/blends no longer affect outside content follows from
   this buffering *(mechanism inferred; observable behavior sourced)*.
4. **Clipping at group level.** If a group is a clipping base, the clipped stack
   above is constrained to the group's opaque result. If the group is a clipped
   layer, it is constrained to its base unless **Blend Clipped Layers As Group**
   alters scope.
5. **Masks.** The group's layer mask and/or vector mask multiply into the group
   result (and, per advanced blending, effects can be restricted to masked
   areas).
6. **Knockout.** A child with Knockout Shallow/Deep composites against the
   resolved stopping point (next layer below the group, base of clip, background,
   or transparency) rather than the immediate backdrop.
7. **Opacity.** Group opacity scales the whole group result; Fill is not
   applicable.

## Rust module mapping

- `pictura_core::node::group` — `Group { pass_through: bool }`; group is a
  `NodeKind` with a child id list, per `ARCH-008`.
- `pictura_core::composite::group` — `composite_group(group, backdrop, ctx) -> Surface`, implementing the Pass-Through fast path (no buffer) vs isolated buffering.
- `pictura_core::composite::knockout` — `resolve_knockout_target(node, ctx) -> Backdrop` (shallow/deep stopping point).
- `pictura_core::document::layer_ops` — `GroupLayers`, `UngroupLayers`, `NewGroup`, `MoveIntoGroup`, `SetGroupBlend`, `SetGroupOpacity`.
- `pictura_core::mask` — group mask resolution shared with `LAY-004`/`LAY-005`.
- Crossing types: `NodeId`, `BlendMode` (incl. `PassThrough`), `Surface`, `ClipScope`, `Knockout`.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `GroupNodeDelegate` | `QStyledItemDelegate` | Folder icon, expand triangle, group blend/opacity, mask/link thumbnails, indentation of nested groups. |
| `GroupPropertiesDialog` | `QDialog` | Name, color label, blend mode (incl. Pass Through), opacity (no fill). |
| `LayersModel` | `QAbstractItemModel` | Reused from `LAY-002`; group rows are `NodeKind::Group` with child rows. |
| `BlendModeComboBox` | `QComboBox` | Offers Pass Through only for group contexts. |
| `PassThroughBadge` | delegate decoration | Distinguishes Pass Through from isolated groups. |

## Data-model impact

- A group is a `NodeKind::Group` node with a child list; nesting is child lists
  recursively.
- Persisted field `pass_through: bool` maps to the PSD group blend key (`pass`)
  when true and to the group's chosen 4-char mode key otherwise (per
  `01-architecture/document-model.md`).
- Group masks are the same `mask`/vector-mask fields as any node; group opacity
  uses the standard opacity field; Fill is absent for groups (the field exists in
  PSD but is unused for groups).
- Knockout lives in the advanced-blending record already attached to a node's
  styles/blending data.
- Undo: group/ungroup, move into/out of group, blend/opacity edits, mask edits,
  and nesting changes are commands (`ARCH-009`).

## Edge cases

- **Empty group.** Must composite to transparent and not force an allocated
  buffer; Pass Through empty group is a no-op.
- **Nested groups.** Pass Through nesting: a child group under a Pass-Through
  parent passes through transitively; an isolated child group creates its own
  buffer boundary.
- **Group selected in a multiple selection with layers.** Only Opacity is exposed
  via the shared control; Fill is disabled for the group portion.
- **Group as clipping base.** The Help requires the base to be a raster layer for
  Merge Clipping Mask; a group base may not merge the same way.
- **Locked group.** Child layers show a dimmed lock; group-level lock/all locks
  all children.
- **32-bit / Lab / CMYK.** Group blend mode list is filtered exactly as for
  layers (15 modes at 32-bit; Lab exclusions).
- **Hidden group.** Children are not composited; visibility state is preserved so
  toggling restores it.
- **Undo of ungroup.** Must restore the original child order and all attributes.
- **GPU unavailable.** Isolated-group buffering still works on the CPU reference.

## Parity acceptance criteria

1. Given a group set to Pass Through, a child's blend mode interacts with the
   parent backdrop; switching the group to Normal first composites children into
   an isolated buffer, and the two composites differ as the blend formulas
   specify (within `11-cross-cutting/testing-strategy.md` tolerance).
2. Given a non-Pass-Through group containing an adjustment layer, content outside
   the group is unaffected by that adjustment; switching the group to Pass
   Through restores the outside effect.
3. Given a group selected, the Fill control is unavailable and Opacity is
   editable.
4. Given nested groups, collapsing/expanding the outer group hides/shows inner
   groups without changing visibility semantics.
5. Given a group with a layer mask, the mask scales the group's composited result;
   unlinking lets the mask move independently of the group's layers.
6. Given a group containing a top layer with Knockout Shallow, the layer punches
   through the grouped layers to the next layer below the group.
7. Given a clipped group, the base layer's opacity/mode are applied to the clipped
   stack when Blend Clipped Layers As Group is selected, and each layer keeps its
   own mode when deselected.
8. Given `Layer > New > Group From Layers` on selected layers, the layers move
   into a new group preserving order, and the group can be set non-Pass-Through
   to confine an adjustment.
9. Given `Ctrl/Cmd+G` then `Ctrl/Cmd+Shift+G`, the selection is grouped and then
   ungrouped with child order and attributes preserved across undo/redo.
10. Given a 32-bit document, the group blend list exposes only the supported
    modes plus Pass Through.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  Photoshop CS6 Help (fetched with `curl`, extracted with `pdftotext -layout`).
  Sections used: "About layers / Organizing layers" (p. 156); "Layers panel
  overview / Filter layers (CS6)" (pp. 156–157); "Selecting, grouping, and
  linking layers" (pp. 163–164); "Create layers and groups" (p. 185); "Layer
  opacity and blending" — group Pass Through and group opacity/fill
  (pp. 192–193); "Group blend effects" / "Knockout to reveal content from other
  layers" (pp. 180, 193–194); "Revealing layers with clipping masks" (p. 174);
  "About layer and vector masks" (pp. 176–179); "Confine adjustment and fill
  layers to specific areas" (pp. 288–289).

## Open questions

- **Group Fill serialization.** Groups expose no Fill in the UI, but PSD stores a
  fill-opacity field for every layer record. Whether CS6 writes a meaningful
  group fill or a constant is not sourced. *Resolves with:* a CS6 group PSD
  inspection.
- **Pass-Through + clipping + adjustment interaction.** The exact composite when
  a Pass-Through group is also a clipping base with an internal adjustment is
  only partly documented. *Resolves with:* CS6 reference renders.
- **Knockout with nested groups.** The stopping point for Shallow across nested
  groups is described as "the first layer after the layer group" but the
  multi-level case is ambiguous. *Resolves with:* controlled CS6 tests.
- **Group thumbnail contents.** Whether the group thumbnail follows the
  document-wide Panel Options (Entire Document vs Layer Bounds) is not stated.
  *Resolves with:* a CS6 UI capture.
