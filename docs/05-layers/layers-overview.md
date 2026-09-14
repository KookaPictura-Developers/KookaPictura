# Layers Overview

- **Spec ID:** `LAY-001`
- **Status:** `Draft`
- **Parity tier:** `Core` — pixel, adjustment, fill, type, shape, smart-object, group layers. `Extended-only` — 3D and video layers (see `00-overview/cs6-editions-and-constraints.md`, `00-overview/feasibility-and-non-goals.md`).
- **New in CS6:** `Changed` — the layer kinds and the bottom-to-top compositing core are unchanged from CS5. CS6 adds Layers-panel filtering/search, the Properties panel (which absorbs the CS5 Masks and Adjustments panels), layer color labels via right-click, simultaneous lock/blend/color edits across a multiple selection, a `Rasterize Layer Style` command, an opacity/blend readout for hidden layers, and `00` / `Shift+00` shortcuts to set layer/fill opacity to 0%.
- **Depends on:** `01-architecture/document-model.md` (`ARCH-008`), `01-architecture/gpu-rendering-pipeline.md` (`ARCH-006`), `05-layers/blend-modes.md`, `05-layers/layer-management-ui.md` (`LAY-002`), `05-layers/layer-groups.md` (`LAY-003`), `05-layers/layer-masks.md` (`LAY-004`), `05-layers/vector-masks-and-clipping-masks.md` (`LAY-005`), `05-layers/smart-objects.md`, `05-layers/layer-styles.md`.

> All module and type names below are **design proposals**. Behaviors are taken
> from the official CS6 Help reference (cited under `## Sources`); anything
> inferred is marked *(inferred)*. Unverified version claims live under
> `## Open questions`.

## CS6 behavior

A Photoshop document is a canvas plus a stack of layers. The Help compares layers
to "sheets of stacked acetate": transparent areas let lower layers show through,
and layer opacity makes content partially transparent. A new image has a single
layer; the number of additional layers, effects, and groups is limited only by
memory.

### Layer kinds

The CS6 Help documents the following layer kinds. (It does **not** document
"artboard" layers — see the note below.)

| Kind | Behavior (sourced) |
|---|---|
| Pixel / raster | Holds pixels; supports painting, filters, masks, styles, blend mode, opacity/fill. |
| Adjustment | Holds color/tonal adjustment data; applies to all layers **below** it (or to the clipped base); can be confined with a layer mask. Contains no pixels, so it increases file size far less than a pixel layer. |
| Fill | Fills with **Solid Color**, **Gradient**, or **Pattern**. Unlike an adjustment layer, a fill layer does **not** affect layers underneath it. |
| Type | Editable text; a type layer can be rasterized. Lock Transparency and Lock Image are on by default. |
| Shape | Vector shape; a shape layer can be rasterized ("Shape", or "Fill Content" leaving the vector mask). |
| Smart Object | A container holding one or more layers of content; transformable non-destructively; carries smart filter effects. |
| Group | Folder of layers passed through as a unit; nestable; can carry blend mode, opacity, and masks. |
| Video (Extended) | Imported video clip; can be masked, transformed, styled, painted per frame, or a frame rasterized to a standard layer. Timeline-driven. |
| 3D (Extended) | Scene-graph layer (meshes, materials, lights); rasterizable to a flat layer. |
| Background | Special bottom layer created with a white or colored canvas; see below. |

**Artboard caveat (contradiction, not asserted as fact).** The CS6 Help PDF
contains no "artboard" feature (the only PDF occurrence is an Illustrator SWF
export option). Adobe's own community answer states artboards "were added to
Photoshop CC and are not available in CS6." `01-architecture/document-model.md`
currently lists "CS6 adds artboards" — that claim is reproduced nowhere in the
primary CS6 source and is treated here as **unverified** pending resolution (see
`## Open questions`). This spec does not model an artboard layer kind for CS6
parity.

### Z-order and compositing order

- Panel order is display order, **topmost entry first**; compositing walks
  **bottom-to-top**.
- When layers are merged, data on the top layers replaces overlapping data on
  lower layers; the intersection of transparent areas stays transparent.
- The **Background** layer is by definition always at the bottom of the stacking
  order. `Layer > Arrange > Send To Back` therefore places an item directly
  above the background, not at the absolute bottom.
- `Layer > Arrange` reorders within the current container (inside a group the
  command affects the group's internal order). `Reverse` reverses at least two
  selected layers.

### Background layer semantics

- Created when a new image is made with a **white background** or a **colored
  background**; it is the bottommost image and is labeled **Background**.
- An image can have **only one** background layer. You **cannot** change its
  stacking order, blending mode, or opacity.
- Convert a background **into** a regular layer: double-click *Background* in the
  Layers panel, or `Layer > New > Layer From Background`.
- Convert a layer **into** a background: `Layer > New > Background From Layer`.
  Transparent pixels become the background color and the layer drops to the
  bottom. Renaming a layer "Background" does **not** create a background.
- `Layer > Flatten Image` merges all visible layers into the background,
  discards hidden layers, and fills remaining transparent areas with **white**.
- Cropping with the Crop tool's **Hide** option is unavailable for images that
  contain only a background layer.

### Blending context: groups isolate; knockouts punch through

- A group's blend mode defaults to **Pass Through**, meaning "the group has no
  blending properties of its own." With Pass Through, layers inside the group
  keep blending against layers outside it.
- Choosing any group blend mode **other than** Pass Through changes the order of
  assembly: the group's layers are put together first, the composite group is
  then treated as a single image and blended with the rest of the image. In that
  case "none of the adjustment layers or layer blending modes inside the group
  will apply to layers outside the group." This is the group **isolation**
  behavior: a non-Pass-Through group is a new backdrop for its children.
- **Knockout** (advanced blending option on the top layer) makes a layer "punch
  through" lower layers to reveal content from elsewhere:
  - **Shallow** knocks out to the first possible stopping point — the first layer
    after the layer group, or the base layer of a clipping mask.
  - **Deep** knocks out to the background; with no background, to transparency.
  - With no group or clipping mask, either option reveals the background layer
    (or transparency if the bottom layer is not a background).
  - The knockout is made visible by lowering **fill opacity** or by changing the
    blend mode.
- **Blend Clipped Layers As Group** (advanced blending) controls whether the base
  layer's blend mode applies to all clipped layers or only to the base; see
  `05-layers/vector-masks-and-clipping-masks.md` and `LAY-003`.
- **Blend Interior Effects As Group** applies the layer's blend mode to effects
  that modify opaque pixels (Inner Glow, Satin, Color/Gradient Overlay) without
  changing effects that touch only transparent pixels (Outer Glow, Drop Shadow).
- **Transparency Shapes Layers** restricts effects/knockouts to opaque areas;
  **Layer Mask Hides Effects** / **Vector Mask Hides Effects** restrict effects
  to the masked area.

### 32-bit (HDR) document behavior

The Help lists what works in a 32-bpc document, including: **New layers,
duplicate layers, adjustment layers** (only Levels, Vibrance, Hue/Saturation,
Channel Mixer, Photo Filter, Exposure), **fill layers, layer masks, layer
styles, supported blending modes, and Smart Objects**. Tools allowed for
painting on HDR images include Brush, Pencil, Pen, Shape, Clone Stamp, Pattern
Stamp, Eraser, Gradient, Blur, Sharpen, Smudge, and History Brush; the Text tool
adds 32-bpc text layers.

Only these blend modes are available for 32-bit images: **Normal, Dissolve,
Darken, Multiply, Lighten, Darker Color, Linear Dodge (Add), Lighter Color,
Difference, Subtract, Divide, Hue, Saturation, Color, Luminosity**.

Layer blend modes in general follow `05-layers/blend-modes.md`. Two documented
restrictions: there is **no Clear blending mode for layers** (Clear is a tool /
Fill / Stroke mode), and for **Lab** images Color Dodge, Color Burn, Darken,
Lighten, Difference, Exclusion, Subtract, and Divide are unavailable.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Layers panel | Dock | `F7` | Lists all layers, groups, and layer effects; hooks for visibility, lock, blend mode, opacity, fill, masks, styles, and (CS6) filtering. See `LAY-002`. |
| Layers panel menu | Menu | n/a | Panel Options, New Layer/Group, Duplicate, Delete, Merge, Flatten, and the blending/advanced-blending entry points. |
| Layer menu | Menu | n/a | `New`, `Rasterize`, `Arrange`, `Layer Mask`, `Vector Mask`, `Create/Release Clipping Mask`, `Group/Ungroup`, `Smart Objects`, `Merge`, `Flatten`. |
| `Layer > New` | Menu | `Ctrl+Shift+N` | Layer, Group, Layer From Background, Background From Layer, Layer Via Copy/Cut, Group From Layers. |
| `Layer > Arrange` | Menu | `Ctrl+[` / `Ctrl+]` etc. | Bring Forward/Send Backward/Bring to Front/Send to Back/Reverse. |
| `Layer > Flatten Image` | Menu | n/a | Merges visible layers into background; discards hidden layers; fills transparent with white. |
| Properties panel | Dock | n/a | CS6; contextual layer/mask/adjustment editor. |
| Timeline panel | Dock | n/a | Extended video/frame-animation layer content. |
| Move tool options bar | Tool bar | `V` | Auto-Select Layer/Group, Show Transform Controls. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Layer kind | enum | Pixel | Pixel, Adjustment, Fill, Type, Shape, Smart Object, Group, Video (Ext), 3D (Ext), Background | See kind table. |
| Layer name | string | "Layer N" | — | Rename via double-click, `Layer Properties`, or Alt-double-click. |
| Blend mode | enum | Normal | 27 layer modes (no Clear, no Behind) + group Pass Through | CS6 Help descriptions define each. 32-bit subset listed above. |
| Opacity | percent | 100 | 0–100 | Affects layer styles and blending. Not editable on Background/locked layers. |
| Fill | percent | 100 | 0–100 | Affects only pixels/shapes/text, not effects. Not available for groups. |
| Visibility | bool | on | — | Eye-icon column; only visible layers print. |
| Lock | flags | none | Lock All, Transparent Pixels, Image Pixels, Position | Type/shape: Transparent+Image on by default and not deselectable. |
| Knockout | enum | None | None, Shallow, Deep | Advanced blending; requires lower fill opacity / blend change to show. |
| Color label | enum | none | layer color palette | CS6: right-click the layer/group. |
| Blend If | channel slider set | — | gray/color channels | Tonal-range blending; see `05-layers/blend-modes.md`. |
| Channels excluded from blend | set | all included | any channel | Advanced Blending. |

## Algorithms & pipeline

1. **Backdrop.** Begin from the document canvas: the Background layer's pixels,
   or a fully transparent canvas if no background exists.
2. **Bottom-to-top walk.** Process layers in reverse panel order (panel is
   top-first, paint is bottom-first).
3. **Per-layer contribution.** Compute `pixels × layer mask × vector mask ×
   fill-opacity`, run any layer styles as internal passes, then composite onto
   the running backdrop using the layer's blend mode and opacity. Blend formulas
   are owned by `05-layers/blend-modes.md`; the PSD keys and 27-mode enumeration
   are in `01-architecture/document-model.md`.
4. **Groups.**
   - Pass Through (default): children composite against the parent backdrop, so
     child blend modes and internal adjustment layers see outside content.
   - Non-Pass-Through: children first composite into an isolated group buffer;
     the buffer is then blended into the parent with the group's mode and
     opacity. The Help states internal adjustments/blends then no longer apply
     outside the group.
5. **Fill vs adjustment.** Adjustment layers apply to the accumulated composite
   below them (or to the clipped base) at their stack position; fill layers do
   not affect layers underneath.
6. **Clipping.** A clipping mask constrains a layer to the opaque area of its
   base; the base's opacity and mode may be applied to the whole clipped stack
   depending on **Blend Clipped Layers As Group**. See `LAY-005`.
7. **Knockout.** When the top layer's Knockout is Shallow/Deep, the layer's shape
   is composited against the layer found by skipping down to the shallow
   stopping point or the background/transparency, instead of against the
   immediately preceding backdrop *(inferred mechanism; observable behavior is
   sourced)*.
8. **32-bit.** Compositing is floating-point per channel; only the supported
   modes/tools listed above participate. Behavior parity only — Adobe's exact
   float rounding is closed.

## Rust module mapping

Proposals, consistent with `ARCH-008`:

- `pictura_core::document` — `Document`, owns the node arena; `Document::composite()` drives the walk.
- `pictura_core::node` — `NodeId`, `Node`, `NodeKind { Pixel, Adjustment, Fill, Text, Shape, SmartObject, Group, Artboard?, Video, ThreeD, Background }`, child ordering (`insert_at`, `move`, `remove`), `stack_position()`.
- `pictura_core::layer` — per-kind resolvers: `pixel::contribution`, `adjustment::apply`, `fill::render`, `group::isolate`.
- `pictura_core::composite` — the compositor: `composite(node, backdrop, ctx) -> Surface`, group isolation, knockout resolution, `BlendScope` handling (clipped/interior/transparency-shapes).
- `pictura_core::mask` — raster/vector masks and clipping-base resolution (shared with `LAY-004`/`LAY-005`).
- `pictura_core::depth` — `BitDepth { U8, U16, F32 }` and mode gating helpers (`blend_modes_available(mode, depth)`, Lab exclusions, HDR subset).
- `pictura_render` (`ARCH-006`) — GPU implementation of the same compositor; CPU reference lives in `pictura_core::composite`.

Crossing types: `NodeId(u64)`, `BlendMode`, `Rect`, `TileDelta`, `Surface`/`PixelBuffer`, `ColorSpace`.

## Qt6 component mapping

Per `01-architecture/qt6-ui-design.md` (Widgets shell).

- `LayersModel` (`QAbstractItemModel`) — tree over `NodeKind::Group` children; roles for id, kind, name, visibility, lock flags, blend mode, opacity, fill, clipping, color label, style badge, mask thumbnails.
- `LayersFilterProxyModel` (`QSortFilterProxyModel`) — CS6 panel filtering by name/kind/effect/mode/attribute/color label (`LAY-002`).
- `LayersPanel` (dock, `QTreeView`) — hosts the model, filter bar, blend/opacity/fill controls, lock strip, and panel menu.
- `LayerKindIconDelegate` / `MaskThumbnailDelegate` (`QStyledItemDelegate`) — kind icons, clip/indent/underline rendering, mask and style thumbnails.
- `BlendingOptionsDialog` (`QDialog`) — blend mode, Blend If, knockout, and advanced-blending scope options.
- `PropertiesPanel` (dock) — selected node's contextual editor, including masks (CS6 behavior).

## Data-model impact

- `NodeKind` covers every CS6 layer kind plus `Background` as a flag/kind with
  constrained fields. 3D and video are opaque node kinds so files round-trip even
  when not rendered.
- Constrained fields: a background has no reorder/blend/opacity; the model should
  reject those edits at the command layer rather than store-and-ignore.
- Group isolation requires a persisted `pass_through: bool` (maps to the PSD
  `pass` blend key); `knockout: None | Shallow | Deep` and the advanced-blending
  booleans (Blend Clipped/Interior, Transparency Shapes, Layer/Vector Mask Hides
  Effects, excluded channels, Blend If) are additional PSD fields.
- Undo: create/delete/reorder/visibility/lock/blend/opacity/fill/color-label are
  commands with records per `ARCH-009`; compositing itself is derived state.
- Artboards: **not** added to the CS6 model pending source resolution.

## Edge cases

- **No background.** The bottom layer is unconstrained; flatten fills transparent
  with white regardless.
- **Hidden layers.** Not printed, not merged by Merge Visible/Flatten (Flatten
  discards them); CS6 correctly displays opacity/blend of hidden layers.
- **Locked / background layers.** Reject opacity, blend, and reorder; partial
  locks reject only their scoped edits.
- **Type/shape layers.** Transparency and image locks are forced on; Layer Via
  Copy/Cut requires rasterizing first.
- **Adjustment/fill as a merge target** is not allowed; adjustment/fill layers
  are invisible to Merge Visible unless rasterized.
- **Groups.** Pass Through vs isolated produces observably different composites;
  group Fill is unavailable (Opacity only).
- **32-bit.** Only the supported modes/tools; unsupported blend modes must be
  filtered from menus. Unsupported filters/tools raise a mode/depth warning.
- **Lab / CMYK.** Lab excludes eight blend modes; CMYK Hard Mix uses the
  maximum color value 100.
- **Bitmap / Indexed.** Normal is called Threshold in bitmap/indexed; layer
  capabilities are severely restricted (see `04-image-ops/image-modes.md`).
- **PSB / huge documents.** Compositor must tile and stream; do not allocate a
  full-canvas surface per layer.
- **GPU unavailable.** Fall back to the CPU reference compositor.
- **1-px / empty documents.** Zero-area layers are legal; the walk must not
  panic on empty buffers.

## Parity acceptance criteria

1. Given layers `[A, B, C]` bottom-to-top, changing order to `[A, C, B]` produces
   exactly the composite of the new order (per `11-cross-cutting/testing-strategy.md` tolerance).
2. Given a group set to Pass Through, a child's blend mode interacts with the
   parent backdrop; switching the group to Normal composites children into an
   isolated buffer first, and the two composites differ as the blend formulas
   specify.
3. Given a Background layer, reorder/blend/opacity edits are refused; `Layer From
   Background` converts it to a movable regular layer with no pixel change.
4. Given a layer with Knockout = Deep over a background, lowering fill opacity
   reveals the background; with no background it reveals transparency.
5. Given a 32-bit document, the blend-mode menu offers exactly the 15 listed
   modes and unsupported adjustment/fill types are absent.
6. Given a Lab document, the eight excluded modes are unavailable.
7. Given `Layer > Flatten Image`, all visible layers merge, hidden layers are
   discarded, and leftover transparent pixels become white.
8. Given a Smart Object layer, it survives compositing and file round-trip and is
   transformable without editing source pixels.
9. Given an adjustment layer, it affects only content below it (or its clipped
   base) and leaves underlying pixel data unchanged after delete/hide.
10. Given a fill layer, it does not alter layers beneath it, unlike an adjustment
    layer.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference (fetched with `curl`; extracted with
  `pdftotext -layout`). Sections used: "About layers" / "Layers panel overview" /
  "Convert background and layers" / "Sample from all visible layers"
  (pp. 156–158); "Merging layers" / "Flatten all layers" (pp. 161–162);
  "Selecting, grouping, and linking layers" (p. 163); "Moving, stacking, and
  locking layers" (p. 165); "About layer and vector masks" (pp. 176–179);
  "Knockout to reveal content from other layers" (p. 180); "Layer opacity and
  blending" including group Pass Through and advanced-blending scope options
  (pp. 192–194); "Blending mode descriptions" and the 32-bit blend-mode note
  (pp. 141–142); "Features that support 32-bpc HDR images" (p. 140);
  "Paint on HDR images" (p. 140); "About adjustment and fill layers" (p. 288);
  "What's new in CS6" — Layers enhancements (p. 7) and JDI Layers list (p. 9).
- `https://community.adobe.com/t5/photoshop-ecosystem-discussions/how-to-put-artboards-in-photoshop-cs6/m-p/10833212`
  — Adobe community answer: "Artboards where added to Photoshop CC and are not
  available in CS6." Established the artboard version contradiction.
- `https://html.duckduckgo.com/html/?q=Photoshop+artboards+introduced+which+version+CS6+or+CC+2015`
  — search results page (via SearXNG) surfacing the community answer above and
  CC-2015 artboard coverage; search-results page, not a primary reference.

## Open questions

- **Artboards in CS6.** `01-architecture/document-model.md` asserts CS6 adds
  artboards; the CS6 Help PDF contains none and Adobe's community states they
  are CC-only. *Resolves with:* an Adobe CS6 release-note/"what's new" source, or
  a CS6-made PSD containing an artboard, or correction of `document-model.md`.
- **Exact knockout compositing math.** Shallow/Deep stopping points are
  documented, the per-pixel mechanism is not. *Resolves with:* controlled CS6
  test PSDs and pixel diffs.
- **Group isolation and clipping interaction.** How Pass Through groups, clipping
  masks, and internal adjustment layers compose in combination is only partly
  documented. *Resolves with:* reference renders from CS6.
- **32-bit layer-kind support matrix.** Which layer kinds (beyond those listed)
  can exist in `Lr32` and render identically is not itemized. *Resolves with:*
  CS6-authored 32-bit PSDs.
- **"Add Mask by Default" scope.** It is an Adjustments-panel preference for new
  adjustment/fill layers; whether any other layer kind auto-adds a mask is not
  stated. *Resolves with:* a default-preferences dump / UI capture.
- **Video/3D node fidelity.** Whether opaque round-trip (without render) is an
  acceptable parity bar for Extended kinds is deferred to
  `00-overview/feasibility-and-non-goals.md`.
