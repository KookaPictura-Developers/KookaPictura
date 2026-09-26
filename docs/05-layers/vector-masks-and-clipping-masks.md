# Vector Masks and Clipping Masks

- **Spec ID:** `LAY-005`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — vector-mask editing and its Density/Feathering controls move from the CS5 **Masks panel** to the CS6 **Properties panel**; the path itself, Reveal/Hide All, Current Path, rasterize-to-layer-mask, and clipping-mask behavior are CS5-era. Clipping masks are unchanged in CS6.
- **Depends on:** `LAY-001`, `LAY-003`, `LAY-004`, `01-architecture/document-model.md` (`ARCH-008`), `03-tools/pen-and-path-tools.md`, `03-tools/shape-tools.md`, `01-architecture/color-management.md`.

> Module and type names are design proposals. Behavior from the CS6 Help
> reference (cited); inferred items are marked.

## CS6 behavior

This spec covers two closely-related, resolution-independent constraints:

- **Vector mask** — a resolution-independent path that clips the layer's
  contents. Created with the pen or shape tools. The Help describes the
  vector-mask thumbnail as representing a path that clips out the layer content.
- **Clipping mask** — uses one layer's content to mask the layers above it.
  The **base layer**'s non-transparent content clips (reveals) the layers above
  it; all other content in the clipped layers is masked out.

### Vector masks

- **Add that reveals or hides the layer:** `Layer > Vector Mask > Reveal All`
  creates a mask revealing the entire layer; `Layer > Vector Mask > Hide All`
  creates one hiding it.
- **Add from a shape/path:** select the layer, then select a path or draw a work
  path (with a Shape tool choose the Paths icon; or the Pen tools), then click
  the **Vector Mask** button in the Properties panel (CS6) / Masks panel (CS5),
  or `Layer > Vector Mask > Current Path`.
- **Edit:** select the layer, click the Vector Mask button in the Properties
  panel (CS6) / Masks panel (CS5), or the thumbnail in the Paths panel; then
  edit the path with the shape, pen, or Direct Selection tools. (See
  `03-tools/pen-and-path-tools.md`.)
- **Opacity / feather:** click the Vector Mask button in the Properties panel
  (CS6) / Masks panel (CS5), then drag **Density** to adjust mask opacity and
  **Feathering** to feather mask edges (same semantic as layer-mask density and
  feather; see `LAY-004`). *(The Help groups these under "Change mask opacity or
  refine edges.")*
- **Disable/enable:** click Disable/Enable Mask in the Properties/Masks panel;
  Shift-click the vector-mask thumbnail; or `Layer > Vector Mask > Disable` /
  `Enable`. A **red X** appears over the thumbnail; content then renders
  unmasked.
- **Delete:** with the vector mask active, click **Delete Mask** in the
  Properties panel (CS6) / Masks panel (CS5).
- **Convert to a layer mask:** `Layer > Rasterize > Vector Mask`. After
  rasterizing, the mask cannot be converted back into a vector object.
- Vector masks are **nondestructive** and can be re-edited without losing the
  pixels they hide.
- A vector mask is distinct from the document's named **Paths**; layer
  vector-mask paths are stored on the layer, while the Paths panel holds named
  paths and the working path (`ARCH-008`).

### Clipping masks

- **Mechanism:** arrange the base layer below the layers to mask; the base's
  non-transparent content reveals the layers above, and all other content in the
  clipped layers is masked out.
- **Multiple clipped layers:** a clipping mask can hold several layers, but they
  must be **successive**. The base layer's name is **underlined**
  and overlying thumbnails are **indented**; each overlying layer displays a
  clipping-mask icon.
- **Create:** Alt/Option-click the dividing line between the base layer and the
  first layer above to include (pointer becomes two overlapping circles); or
  select the first layer above the base and `Layer > Create Clipping Mask`. Add
  more layers by working upward one level at a time.
- **Implicit membership:** if you create a new layer between layers in a clipping
  mask, or drag an unclipped layer between them, that layer **becomes part of the
  clipping mask**.
- **Assigned attributes:** clipped layers take on the base layer's opacity and
  blend-mode attributes.
- **Remove / release:** Alt/Option-click the line separating two grouped layers,
  or select a layer in the mask and `Layer > Release Clipping Mask` (removes the
  selected layer and any layers above it). To release all, select the clipped
  layer just above the base and release.
- **Blend scope:** by default the clipped layers blend with the layers underneath
  using the blending mode of the bottommost layer in the group.
  **Blend Clipped Layers As Group** (advanced blending on the base)
  applies the base's mode to all clipped layers; deselect it to preserve each
  layer's own mode.
- **Merge:** `Merge Clipping Mask` from the Layer menu / panel menu merges a
  clipping mask; the base layer must be a **raster layer**.
- **Knockout integration:** to reveal the base layer of a clipping mask, place
  the layers in a clipping mask and ensure **Blend Clipped Layers As Group** is
  selected for the base; then use Knockout Shallow/Deep on the top layer (see
  `LAY-003`).

### Clipping into / by groups

- Clipping masks and groups interact through the base layer's **Blend Clipped
  Layers As Group** option (`LAY-003`). Clipped layers can include groups, and a
  group can act as a base; the Group spec covers the isolation consequences.
  *(The Help documents the option and the successive-layer rule; the exact
  group-as-base compositing is covered in `LAY-003` and flagged there as an open
  question.)*

### Difference from layer masks

| Aspect | Layer (pixel) mask | Vector mask | Clipping mask |
|---|---|---|---|
| Nature | Resolution-dependent grayscale bitmap | Resolution-independent path | Relationship to a base layer's transparency |
| Created by | Painting / selection | Pen or shape tools | Alt-click dividing line / Layer menu |
| Edges | Painterly, can be soft/gray | Clean, defined, aliased-to-path | Determined by base content (its own alpha) |
| Controls | Grayscale 0–max, Density, Feather, Invert, Mask Edge, Color Range | Density, Feather; path edit; no Invert (Help scopes Invert to layer masks) | Base opacity/mode applied per Blend Clipped Layers As Group |
| Storage | Alpha channel (`-2`/`-3`) | Vector-mask path block | Additional-layer clipping byte on the layer |
| Scope | One layer/group | One layer/group | A successive stack above a base |
| Nondestructive | Yes | Yes | Yes (release restores) |

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Properties panel (CS6) | Dock | n/a | Vector Mask button; Density, Feathering; Delete Mask; Disable/Enable; vector-mask thumbnail. |
| Masks panel (CS5) | Dock | n/a | Superseded by Properties panel in CS6. |
| Layers panel | Dock | `F7` | Vector-mask thumbnail; Shift-click toggles; clipping icons/indentation/underline; link icon. |
| `Layer > Vector Mask` | Menu | n/a | Reveal All, Hide All, Current Path, Delete, Disable, Enable. |
| `Layer > Rasterize > Vector Mask` | Menu | n/a | Converts the vector mask to a layer mask (irreversible). |
| `Layer > Create Clipping Mask` | Menu | `Ctrl+Alt+G` | Creates a clipping mask above the base. |
| `Layer > Release Clipping Mask` | Menu | `Ctrl+Alt+G` | Releases the selected layer and those above. |
| `Layer > Merge Clipping Mask` | Menu | n/a | Merges the clipping mask; base must be a raster layer. |
| Paths panel | Dock | n/a | Named paths / work path; vector-mask path can be edited here. |
| Alt-click dividing line | On-panel | n/a | Creates/releases a clipping mask (pointer = two circles). |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Vector mask state | enum | none | Reveal All / Hide All / Current Path | Add paths. |
| Vector mask Density | percent | 100 | 0–100 | Adjusts mask opacity. |
| Vector mask Feathering | double px | 0 | ≥ 0 (not stated) | Feathers mask edges. |
| Vector mask enabled | bool | on | on/off | Shift-click thumbnail. |
| Vector mask linked | bool | on | on/off | Link icon between thumbnails. |
| Clipping | bool | off | base / clipped / none | PSD clipping byte. |
| Blend Clipped Layers As Group | bool | on | on/off | Base layer advanced-blending option. |
| Clipped stack length | int | 1 | ≥ 1, successive layers only | Multiple clipped layers. |
| Base layer kind | enum | raster | raster required for Merge Clipping Mask | — |

## Algorithms & pipeline

1. **Vector mask = path fill.** The path is converted to a coverage mask
   (non-zero / even-odd fill rules) at the required raster resolution; the
   layer's contribution is multiplied by this coverage. Because the path is
   resolution-independent, re-rasterizing at a different zoom/resolution does not
   lose edge definition (unlike a layer mask). *(mechanism inferred; the
   resolution-independence property is sourced.)*
2. **Density** scales the vector-mask coverage toward fully-blocking at 100%;
   **Feathering** blurs the coverage edges, as for layer masks (`LAY-004`).
3. **Clipping = base alpha as mask.** For each clipped layer, the running
   composite is multiplied by the base layer's alpha (the "non-transparent
   content"). Only content within the base's opaque area survives. *(mechanism
   inferred from the documented observable.)*
4. **Blend scope.** With **Blend Clipped Layers As Group** selected, the base's
   blend mode is used for the whole clipped stack; deselected, each clipped
   layer keeps its own mode. Base opacity likewise applies to the stack
   (documented assignment).
5. **Successive-layer enforcement.** The model must enforce that a clipped run is
   contiguous; inserting or dragging a layer between clipped layers joins the
   run, matching the documented note.
6. **Persistence.** In PSD, clipping is a byte on the layer record; vector masks
   are stored as a path block in the layer's additional-layer information, with
   channel id `-3` when both a user mask and a vector mask exist (`ARCH-008`).
7. **Ordering with layer masks.** When both exist, the vector mask and the layer
   mask both multiply the layer contribution; effects can additionally be
   restricted by **Vector Mask Hides Effects** (`LAY-001`).
8. **Convert vector→layer mask** rasterizes the path coverage into a grayscale
   buffer and discards the path.

## Rust module mapping

- `pictura_core::mask::vector` — `VectorMask { path: PathId, density: u8, feather: f32, enabled: bool, linked: bool }`; rasterization to coverage via the path engine.
- `pictura_core::path` (`ARCH-008`) — `BezierPath`, subpaths, fill rules; shared with the Paths panel and shape layers.
- `pictura_core::mask::clipping` — `ClippingScope { None, Base, Clipped }`; `resolve_clip_base(node) -> Option<NodeId>`; enforcement of successive clipped runs.
- `pictura_core::composite::clipping` — multiply the clipped stack by the base alpha; apply base opacity/mode per `Blend Clipped Layers As Group`.
- `pictura_core::rasterize` — `RasterizeTarget::VectorMask` producing a `RasterMask`.
- `pictura_core::document::layer_ops` — `CreateClippingMask`, `ReleaseClippingMask`, `MergeClippingMask`, `AddVectorMask`, `SetVectorMaskDensity`, `SetVectorMaskFeather`.
- Crossing types: `NodeId`, `PathId`, `MaskRef`, `ClipScope`, `BitDepth`, `Surface`.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `VectorMaskPropertiesPage` | `QWidget` | Properties-panel page: Vector Mask button, Density/Feathering, disable/delete. |
| `ClippingBadgeDelegate` | `QStyledItemDelegate` | Clipping icon, indentation, base underline, link/vector-mask thumbnails. |
| `PathsModel` / `PathsPanel` | `QAbstractListModel` / dock | Named paths + working path; select a path for Current Path. |
| `PathEditInteraction` | `QGraphicsScene` interaction | Pen/Direct Selection editing shared with tools specs. |
| `BlendClippedLayersOption` | `QCheckBox` in `BlendingOptionsDialog` | Base-layer clipping scope. |

## Data-model impact

- `Node.mask` carries an optional `VectorMask` in addition to the raster mask.
  When both exist, PSD uses the `-3` "real user mask" channel (`ARCH-008`).
- `Node.clipping` is already present as the PSD clipping byte; the model should
  expose a derived notion of "is clipped" and "is a clipping base."
- Clipping-run contiguity is an invariant maintained by reorder/insert commands.
- `Blend Clipped Layers As Group` and `Vector Mask Hides Effects` live in the
  node's advanced-blending record.
- Undo: add/edit/delete vector mask, path edits, density/feather, create/release
  clipping, and merge-clipping are commands (`ARCH-009`); path edits can be
  fine-grained (control point) or coalesced per drag.

## Edge cases

- **Vector mask on a shape layer.** The shape's own path and the vector mask
  coexist; `Rasterize > Fill Content` leaves the vector mask while rasterizing
  the shape fill.
- **Rasterizing a vector mask** is irreversible; the result is a layer mask and
  loses resolution independence.
- **No selection vs selection** for clipping creation is irrelevant; clipping is
  positional (successive-layer relationship).
- **Non-contiguous clip** must be prevented or auto-normalized; inserting between
  clipped layers joins the run.
- **Base is not raster** — Merge Clipping Mask must refuse or report; the Help
  requires a raster base.
- **Both masks present** — both affect compositing; the `-3` channel appears only
  when the file contains it.
- **Clipping plus Knockout** — requires Blend Clipped Layers As Group selected on
  the base; otherwise the knockout stopping point differs.
- **32-bit / Lab / CMYK** — vector masks are mode-independent; clipping uses the
  working-space alpha; blend scope respects the 32-bit mode subset.
- **PSB / huge docs** — vector-mask rasterization must tile, not allocate a
  full-canvas coverage buffer.
- **GPU unavailable** — coverage rasterization and clipping run on the CPU
  reference.
- **Empty path / empty base** — a Reveal All vector mask is a no-op; an
  all-transparent base clips everything (stack becomes invisible).

## Parity acceptance criteria

1. Given `Layer > Vector Mask > Current Path` with a drawn path, layer content
   outside the path is clipped and content inside is visible; the mask is
   re-editable without pixel loss.
2. Given a vector mask, Density lowers mask opacity and Feathering softens the
   edges, matching the layer-mask semantics.
3. Given `Layer > Rasterize > Vector Mask`, the mask becomes a layer mask, is no
   longer editable as a path, and cannot be converted back.
4. Given a base layer with three successive layers above it, `Layer > Create
   Clipping Mask` clips all successive layers to the base; the base name is
   underlined and clipped thumbnails are indented with clip icons.
5. Given a new layer inserted between clipped layers, it becomes part of the
   clipping mask.
6. Given a clipping mask, `Layer > Release Clipping Mask` on the clipped layer
   frees that layer and all above; releasing the layer just above the base frees
   all.
7. Given Blend Clipped Layers As Group selected on the base, the base's blend
   mode applies to the whole clipped stack; deselected, each layer keeps its own
   mode.
8. Given `Layer > Merge Clipping Mask` with a raster base, the clipped stack
   merges into the base; a non-raster base is refused.
9. Given a layer with both a user mask and a vector mask, both affect the
   composite and both survive a PSD round-trip.
10. Given a knockout on the top clipped layer with Blend Clipped Layers As Group
    on, the knockout reveals the base layer per the Help's instructions.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  Photoshop CS6 Help (fetched with `curl`, extracted with `pdftotext -layout`).
  Sections used: "Masking layers with vector masks" — Add/edit vector masks,
  reveal all/hide all, Current Path, change opacity/feather, remove, disable,
  convert to layer mask (pp. 171–172); "Revealing layers with clipping masks" —
  create/remove/release (p. 174); "About layer and vector masks" (pp. 176–179);
  "Adjusting mask opacity and edges" (pp. 178–179); "Knockout to reveal content
  from other layers" (p. 180); "Merge layers in a clipping mask" (p. 161);
  "Layer opacity and blending / Group blend effects" (pp. 192–194); "Keys for the
  Layers panel" (pp. 89–90).

## Open questions

- **Vector-mask density/feather ranges.** The Help gives no slider maxima. *Resolves
  with:* a CS6 UI capture or calibration.
- **Fill rule for vector masks.** Whether paths use non-zero, even-odd, or a
  per-path rule is not stated in the layer-mask section. *Resolves with:* the
  pen/path specs (`03-tools/pen-and-path-tools.md`) and CS6 test paths.
- **Group as clipping base.** The compositing of a group base with
  Blend Clipped Layers As Group across nesting is only partly documented.
  *Resolves with:* CS6 reference renders (shared with `LAY-003`).
- **Vector-mask PSD block details.** The exact additional-layer key and structure
  for vector masks is not in the fetched file-format excerpt (flagged in
  `ARCH-008`). *Resolves with:* a CS6 vector-mask PSD byte inspection.
- **Clipping + vector mask ordering.** Whether the base's own vector mask
  further constrains the clipped stack, and in what order, is not documented.
  *Resolves with:* controlled CS6 composites.
- **Clip creation shortcut scope.** `Ctrl+Alt+G` creates/releases a clipping mask
  per the shortcut table; whether it works identically for a group base is not
  stated. *Resolves with:* a CS6 UI test.
