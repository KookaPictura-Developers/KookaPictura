# Shape Tools

- **Spec ID:** `TOOL-056` (Rectangle, Rounded Rectangle, Ellipse, Polygon incl. star, Line), `TOOL-057` (Shape/Path/Pixel modes, shape options, fill & stroke, Properties panel)
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Yes` — CS6 makes the Line and Shape tools create **fully vector-based objects** ("Vector Layers" replacing the CS5 shape-layer presentation), with **Fill** and **Stroke** appearance directly on the options bar (None / Solid Color / Gradient / Pattern; stroke line type, Align, Caps, Corners, presets), copies shape attributes between layers, and adds **Snap to Pixel Grid**, **Align Edges**, and expanded **Path Operations / Alignment / Arrangement** menus.
- **Depends on:** `ARCH-008` document-model (`Shape` node + vector mask), `ARCH-002` rust-core-design, `ARCH-003` qt6-ui-design, `03-tools/pen-and-path-tools.md`, `03-tools/path-selection-tools.md`, `03-tools/custom-shape.md`, `05-layers/vector-masks-and-clipping-masks.md`, `02-ui-ux/panels/properties-panel.md`.

## CS6 behavior

The shape tool group shares the `U` slot (cycle with `Shift+U`):

| Tool | Draws | Geometry options |
|---|---|---|
| **Rectangle** | Axis-aligned rectangle/square | Unconstrained, Fixed Size (W×H), Proportional (W/H), From Center, Square, Snap To Pixels |
| **Rounded Rectangle** | Rectangle with corner radius | as Rectangle + Radius (corner radius); Square; Snap To Pixels |
| **Ellipse** | Ellipse/circle | Unconstrained, Fixed Size, Proportional, From Center, Circle |
| **Polygon** | Regular polygon; **star** with Indent Sides By | Sides, Radius, Smooth Corners / Smooth Indents, Indent Sides By (creates a star: % of radius taken by the points) |
| **Line** | Straight line | Weight (px), Arrowheads Start/End (Width 10–1000%, Length 10–5000%, Concavity −50%…+50%) |
| **Custom Shape** | Preset shape outline | Unconstrained, Fixed Size, Defined Proportions, Defined Size, From Center | See `03-tools/custom-shape.md` |

### Drawing modes (CS6 mode selector in the options bar)

| Mode | Result |
|---|---|
| **Shape** | A new **shape layer**: a fill layer defining color plus a linked vector mask defining the outline (both editable). Multiple shapes per layer are allowed. |
| **Path** | A **work path** on the current layer (temporary unless saved); can become a selection, vector mask, or raster fill/stroke. |
| **Fill Pixels** | Raster pixels painted directly on the current layer, using the foreground color; only shape tools work here (not Pen). Options: Mode (blend mode), Opacity, Anti-aliased. |

### CS6 vector fill and stroke

- Drawing in Shape mode produces a vector layer whose **Fill** and **Stroke** are set in the options bar and remain editable.
- **Fill**: None (transparent), Solid Color, Gradient, or Pattern. Solid Color offers recent swatches and a Color Picker; Gradient and Pattern offer preset thumbnails.
- **Stroke**: None, Solid Color, Gradient, or Pattern, plus a width (default **3 pt**) and a **Stroke Options** panel: line type (solid / dashed / dotted), **Align** (inside / center / outside), **Caps** (Butt / Round / Square), **Corners** (Miter / Round / Bevel), and **More Options** (detailed stroke dialog with presets).
- **Align Edges** (options bar, on by default) removes anti-aliasing along the shape's vertical and horizontal edges and aligns them to the pixel grid. It works when the stroke width is specified in **pixels** (not points).
- Shape attributes can be copied/pasted: Fill or Complete Stroke from the appearance panel's Options, from the right-click context menu on a vector layer, or `Copy/Paste Shape Attributes` (both fill and stroke) from the Layers-panel context menu.
- For shapes other than Line, stroke width is also reachable via `Layer > Layer Style > Stroke`; the two mechanisms must not be conflated (see Open questions).

### Shape layers and the Properties panel

- A shape layer is a fill layer plus a vector mask. The **Properties panel** (introduced in CS6 in place of the CS5 Masks panel) shows controls for the selected layer's components: vector mask **Density** and **Feather**, mask enable/disable, plus adjustment-layer and Smart Object properties. It also lets you modify the layer components selected in the Layers panel.
- Important scoping note: the task brief describes "live shape properties" (directly editing a shape's W/H and per-corner radius in the Properties panel) as new/expanded in CS6. The CS6 Help PDF documents the CS6 Properties panel for masks/adjustments/shape components but does **not** document interactive corner-radius live-shape editing. Multiple Adobe community answers state that **Live Shapes were introduced in Photoshop CC**, not CS6; in CS6 the corner radius is set in the options bar before/while drawing and cannot be changed afterwards via a live property. This spec therefore treats interactive live-shape corner editing as **out of CS6 parity scope** unless a CS6 source confirms otherwise (see Open questions).
- Editing an existing shape: double-click the shape layer thumbnail to change the fill color; edit the vector mask outline with Direct Selection and the pen tools; move with the Move tool; apply layer styles.

### Pixel snapping and edge behavior

- **Snap Vector Tools and Transforms to Pixel Grid** (global preference, enabled by default) aligns drawn/moved vector paths and anchor points to the pixel grid. Combined with **Align Edges**, vector objects render crisp at 100%.
- Nudging: with snapping on, a nudge is exactly 1 px regardless of zoom; with snapping off, the nudge is zoom-dependent (1 px at 100%, 0.5 px at 200%, 0.33 px at 300%, …).
- Dragging multiple selected points and dragging with the Move tool are relative; dragging a single point snaps it to the grid.

### Path operations and alignment

CS6 replaces the CS5 boolean buttons with options-bar drop-down menus: **Path Operations** (Combine Shapes / Add To Shape Area, Subtract From Shape Area, Intersect Shape Areas, Exclude Overlapping Shape Areas, Merge Shape Components), **Path Alignment**, and **Path Arrangement** (distribute). Shift starts additive mode while drawing; Option subtracts; Shift+Option intersects.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel: Rectangle / Rounded Rectangle / Ellipse / Polygon / Line / Custom Shape | Tool | `U` (Shift+`U`) | Grouped slot |
| Options bar: mode selector | Buttons | n/a | Shape / Path / Fill Pixels |
| Options bar: Fill swatch | Pop-up | n/a | None / Solid / Gradient / Pattern |
| Options bar: Stroke swatch + width | Pop-up + field | n/a | Default width 3 pt |
| Options bar: Stroke Options | Pop-up | n/a | Line type, Align, Caps, Corners, More Options |
| Options bar: geometry options (gear) | Pop-up | n/a | Per-tool geometry |
| Options bar: Path Operations | Menu | n/a | Combine/Subtract/Intersect/Exclude/Merge |
| Options bar: Path Alignment / Arrangement | Menus | n/a | Align/Distribute components |
| Options bar: Align Edges | Toggle | n/a | CS6 |
| Options bar: W / H (and link) | Fields | n/a | Resize after drawing |
| Properties panel | Dock | n/a | Vector mask Density/Feather; layer components |
| Layers panel | Dock | `F7` | Vector shape layer thumbnail/badge |
| `Layer > New Fill Layer` | Menu | n/a | Legacy Solid Color/Gradient/Pattern fill layers |
| `Layer > Layer Style > Stroke` | Menu | n/a | Alternative stroke for non-Line shapes |
| `Edit > Preferences > General` | Dialog | `Ctrl+K` | Snap Vector Tools and Transforms to Pixel Grid |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Mode | enum | Shape | Shape / Path / Fill Pixels | CS6 selector |
| Fill | enum + value | Solid? (Help does not state) | None / Solid / Gradient / Pattern | Recent swatches + picker |
| Stroke | enum + value | None | None / Solid / Gradient / Pattern | |
| Stroke width | number | 3 pt | > 0; units px/pt/… | px required for Align Edges |
| Stroke line type | enum | Solid | Solid / Dashed / Dotted | |
| Stroke Align | enum | Center (Help does not state) | Inside / Center / Outside | |
| Stroke Caps | enum | Butt | Butt / Round / Square | |
| Stroke Corners | enum | Miter | Miter / Round / Bevel | |
| Weight (Line) | px | 1 (Help does not state) | > 0 | Line only |
| Arrowheads | toggles | off | Start / End | Line only |
| Arrowhead Width | percent of line width | 100 | 10–1000 | |
| Arrowhead Length | percent of line width | 100 | 10–5000 | |
| Arrowhead Concavity | percent | 0 | −50…+50 | |
| Radius (Rounded Rect) | px | 0 | ≥ 0 | Corner radius |
| Radius (Polygon) | px | — | ≥ 0 | Center to outer points |
| Sides (Polygon) | int | 5 | 3–100 (Help gives no explicit max) | |
| Indent Sides By (Polygon) | percent | 0 | 0–100 | Non-zero makes a star |
| Smooth Corners / Smooth Indents | toggle | off | on/off | Polygon |
| Unconstrained / Fixed Size / Proportional / From Center / Square / Circle / Defined Proportions / Defined Size | geometry modes | Unconstrained | as listed | Per tool |
| Fixed/Proportional W, H | number | — | > 0 | |
| Snap To Pixels | toggle | off | on/off | Rectangle/Rounded only |
| Fill Pixels: Mode | enum | Normal | 27 blend modes | Raster mode |
| Fill Pixels: Opacity | percent | 100 | 0–100 | Raster mode |
| Fill Pixels: Anti-aliased | toggle | on | on/off | Raster mode |
| Align Edges | toggle | on | on/off | Per vector layer |
| Vector Mask Density | percent | 100 | 0–100 | Properties panel |
| Vector Mask Feather | px | 0 | ≥ 0 | Properties panel |
| Path operation | enum | — | Combine/Subtract/Intersect/Exclude/Merge | |

## Algorithms & pipeline

Behavioral parity target: for the same drawing gesture and options, the generated path geometry, fill, and stroke match CS6 within the tolerance of `11-cross-cutting/testing-strategy.md`. Adobe's exact stroke-outline and anti-alias implementation is closed; those are **behavioral parity only, algorithm TBD**.

- **Shape generation** produces a `VectorPath` (see `03-tools/pen-and-path-tools.md`) for each tool:
  - Rectangle: four `LineTo` segments; From Center offsets by half W/H; Fixed Size/Proportional constrain; Shift constrains to square.
  - Rounded Rectangle: rectangle with elliptical corner arcs of radius r (clamped to half the smaller dimension). `kurbo::RoundedRect`/`QPainterPath::addRoundedRect` are the reference.
  - Ellipse: two/four cubic approximations of a full ellipse (Qt and kurbo both use Bézier approximation); Shift = circle.
  - Polygon: n points on a circle of `Radius`, starting angle chosen so the polygon is upright; star = alternate outer/inner points where inner radius = Radius × (Indent Sides By)/100; Smooth Corners/Indents rounds the corners/indents.
  - Line: a single `MoveTo`+`LineTo` with stroke weight and optional arrowheads; arrowheads are generated geometry (triangles/curves) parameterized by Width/Length/Concavity.
- **Shape layer construction:** create/reuse a shape layer; the generated path becomes the layer's vector mask; fill and stroke become the layer's appearance (Solid Color/Gradient/Pattern). This maps to `ARCH-008`'s `NodeKind::Shape { path, fill, stroke }`.
- **Path operations:** Combine = union of the existing component(s) with the new one; Subtract = difference; Intersect = intersection; Exclude = symmetric difference; Merge = union of all selected components. All use the path fill rule and are computed as area operations (implicitly closing open subpaths).
- **Stroke rendering:** stroke-to-outline with width, align (inside/center/outside), caps, joins/miter, dash/dot pattern. `kurbo::stroke`/`dash` (Rust) and `QPainterPathStroker` (Qt preview) are references.
- **Pixel snapping:** quantize shape path coordinates and the whole-object vertical/horizontal edges to the pixel grid when the global Snap preference is on and Align Edges is on. Align Edges removes AA on vertical/horizontal edges but does not snap non-edge interior points.
- **Fill Pixels:** rasterize the shape geometry through the scan-line/coverage rasterizer at the document depth/color space, compositing with the chosen blend mode and opacity; anti-aliased or not.

## Rust module mapping

- `pictura_vector::shape` — shape generators: `RectSpec`, `RoundedRectSpec`, `EllipseSpec`, `PolygonSpec` (`sides`, `radius`, `indent`, `smooth`), `LineSpec` (`weight`, arrowhead params), each producing a `VectorPath`.
- `pictura_vector::stroke` — stroke-to-outline, caps/joins/miter/align, dash/dot, shared with the pen spec.
- `pictura_vector::boolean` — Path Operations and Merge.
- `pictura_vector::snap` — pixel-grid and Align Edges quantization.
- `pictura_vector::rasterize` — fill-pixels path-to-mask rendering (or hand geometry to `pictura-render`).
- `pictura-render` consumes shape fill (solid/gradient/pattern) and stroke appearance for compositing.

Crossing types: `VectorPath`, `FillSpec { None | Solid | Gradient | Pattern }`, `StrokeSpec`, `ShapeGeometry` options; document `Shape` node per `ARCH-008`.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `ShapeToolHandler` | `QObject` | Per-tool drag/click drawing and geometry clamping; Shift/Alt modifiers |
| `ShapeOptionsBar` | `QWidget` | Mode selector, Fill/Stroke pop-ups, width, Stroke Options, geometry gear, path ops, Align Edges, W/H |
| `FillStrokePopup` | `QWidget`/`QMenu` | None/Solid/Gradient/Pattern editors |
| `StrokeOptionsPopup` | `QWidget` | line type, Align, Caps, Corners, presets |
| `ShapeGeometryPopup` | `QWidget` | Per-tool geometry options (Radius/Sides/Indent/arrows/…) |
| `ShapePreviewOverlay` | `QGraphicsPathItem` | Live path preview while drawing |
| `PropertiesPanelShapePage` | `QWidget` | Vector mask Density/Feather; component selection; (no CS6 live corner editing) |
| `ShapesPanel`/picker | `QAbstractItemModel` + delegate | Custom shape preset grid (see custom-shape spec) |

Previews are `QPainterPath`; `QPainterPathStroker` previews stroke outline; the canvas overlay is managed by `ARCH-003`'s `QGraphicsView`.

## Data-model impact

- `ARCH-008`'s `NodeKind::Shape { path, fill, stroke }` is the target. It must gain typed `FillSpec`/`StrokeSpec` and the geometry parameters needed to keep a shape editable/reconstructable (for CS6 parity), or at minimum the resulting path plus the appearance.
- CS6 vector layers change the **Layers panel presentation** (vector shape badge) though functionally similar to CS5's fill-layer+mask; the document graph can represent both.
- Vector mask **Density** and **Feather** (Properties panel) are layer-mask parameters, serialized per `ARCH-008` (PSD mask flag bit 4 / 8-byte double feather).
- Copy/paste Shape Attributes operates on FillSpec/StrokeSpec, not on pixels.
- `Align Edges` and the global Snap preference: Align Edges is a per-vector-layer flag that must serialize (PSD additional-layer info; key not sourced — Open question); the Snap preference is application state.
- Undo: one record per committed shape draw, per fill/stroke/geometry change, and per path operation. Pixel-mode fills are destructive pixel operations with pixel backups.
- Custom-shape presets are external `.CSH` files, not document data (see `03-tools/custom-shape.md`).

## Edge cases

- **Radius/Sides bounds.** Radius > half the smaller side must clamp; Sides < 3 must be rejected; Indent Sides By at 0 is a polygon, at 100 a degenerate star; Smooth Corners with large radius must not self-intersect badly.
- **Line with zero length.** No-op / no layer creation.
- **Fill Pixels on non-raster layers.** Not allowed on vector/type/mask/fill/adjustment/Smart Object layers.
- **Align Edges without pixel stroke width.** Has no effect on non-integer stroke widths; must be documented, not silently "fixed".
- **Sub-pixel positions.** Preserve f64 internally; only snap on render/snap settings so toggling snapping is reversible.
- **Boolean edge cases.** Empty/overlapping/self-intersecting components; open subpaths treated as closed; preserve winding.
- **CMYK/Lab/Bitmap/Indexed.** Fill Pixels and gradients must render in the document color space; some mode/depth combinations restrict color features.
- **Huge/PSB canvases.** Shape bounds up to 300,000 px; avoid tessellating off-canvas geometry.
- **GPU unavailable.** Stroke/fill previews and rasterization fall back to CPU.
- **Gradient/Pattern fill dependencies.** Pattern fill requires a loaded pattern preset; missing presets must degrade to a clear error, not a crash.
- **Undo across mode switches.** Changing Shape→Path→Pixels mid-session must leave history consistent.

## Parity acceptance criteria

1. Given Rectangle with Shift, the drawn shape is a square; with Alt/Option, it is drawn from the center; with Fixed Size W×H, it is exactly that size.
2. Given Rounded Rectangle with Radius r, corner arcs have radius min(r, half the smaller side) within sub-pixel tolerance.
3. Given Polygon with Sides n and Radius R, the vertices lie on a circle of radius R; with Indent Sides By k>0, alternate vertices lie at radius R·k/100, producing a star.
4. Given Line with Weight w and Arrowheads Start/End, both arrowheads are generated with the specified Width/Length/Concavity percentages relative to the line width.
5. Given Shape mode, drawing creates a shape layer with an editable vector mask and editable Fill/Stroke; switching to Path mode creates a work path and no layer.
6. Given Fill Pixels mode, the shape is rasterized with the chosen blend mode, opacity, and anti-aliasing; no vector layer is created.
7. Given two overlapping components and Subtract From Shape Area, the overlapping area is removed and layers below show through; Combine/Intersect/Exclude produce union/intersection/symmetric-difference respectively.
8. Given a shape with Fill = None and Stroke = dashed 10 px inside-aligned, the rendered stroke has no fill, is dashed, and lies inside the outline.
9. Given `Align Edges` on and an integer stroke width in pixels, the shape's vertical/horizontal edges show no anti-aliasing; with it off, an anti-aliased fringe appears.
10. Given Snap Vector Tools and Transforms to Pixel Grid on, drawing/nudging lands on the pixel grid; toggling it off restores sub-pixel positioning.
11. Given `Copy/Paste Shape Attributes` from layer A to layer B, B receives A's fill and complete stroke, while the heart/butterfly example's independent color changes remain per-layer.
12. Given Vector Mask Density < 100 and Feather > 0 in the Properties panel, the shape's mask is applied with those values and round-trips through PSD.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — established: shape tool set and shortcuts; drawing modes (Shape Layers / Paths / Fill Pixels); create-shape steps and Shift/Alt constraints; Polygon star via Indent Sides By; shape tool options (Arrowheads Width 10–1000% / Length 10–5000% / Concavity −50…+50, Circle, Defined Proportions/Size, Fixed Size, From Center, Proportional, Radius, Sides, Smooth Corners/Indents, Snap To Pixels, Square, Unconstrained, Weight); Add/Subtract/Intersect/Exclude shape area operations; draw-a-wheel example; Define Custom Shape and Save Shapes; rasterized shape options (Mode/Opacity/Anti-Aliased); CS6 Merge Shape Components and Path Operations/Alignment/Arrangement; vector mask via Properties panel (CS5 Masks panel); shape layer = fill layer + linked vector mask; Fill Path/Stroke Path options; CS6 What's-New "Vector layers" (Line and Shape tools create fully vector objects; dashed strokes; gradients) and "Intuitive path editing" (Direct Selection adjusts related segments; Constrain Path Dragging; Align Edges); Properties panel context (masks/adjustments/3D), not live-shape corner editing; `Snap To Pixels` options.
- `https://bjango.com/articles/photoshopcs6vectorshapes` — CS6 pixel snapping moved to global **Snap Vector Tools and Transforms to Pixel Grid**; nudge-vs-zoom table; **Align Edges** snaps layer edges while non-edge points stay unsnapped; vector layers appear as vector shapes in the Layers panel; boolean options moved with sort order significance; Shift/Option/Shift+Option add/subtract/intersect while drawing.
- `https://planetphotoshop.com/cs6-vector-tools.html` — "Vector Layers replace Shape Layers"; Shape/Path/Pixels mode; click-to-open shape geometry dialog (From Center sticky); Fill and Stroke appearance with None/Solid/Gradient/Pattern; Stroke Options with Align/Caps/Corners and preset save; Copy/Paste Fill/Stroke and Copy/Paste Shape Attributes; Snap to Pixel Grid and Align Edges behavior; Vector Mask via `Window > Properties` with Density/Feather.
- `https://www.photoshopessentials.com/basics/how-to-use-the-custom-shape-tool-in-photoshop-cs6` — CS6 Custom Shape tool workflow, shape mode, fill/stroke swatches, stroke width default 3 pt, Align Edges requires pixel stroke width, Shape layers per shape.
- `https://doc.qt.io/qt-6/qpainterpath.html` — shape constructors (`addRect`, `addRoundedRect`, `addEllipse`, `addPolygon`), fill rule, boolean ops, `elementAt`/`setElementPositionAt`.
- `https://doc.qt.io/qt-6/qpainterpathstroker.html` — stroke outline width/cap/join/dash/miter.
- `https://docs.rs/kurbo` — `Rect`, `RoundedRect`, `Ellipse`, `Circle`, `stroke`, `dash` primitives.
- `https://html.duckduckgo.com/html/?q=Photoshop+CS6+shape+tools+live+shape+properties+rectangle+rounded+ellipse+polygon+star+line` and `https://search.brave.com/search?q=%22live+shapes%22+Photoshop+introduced+CC+2014+corner+radius+CS6+properties+panel+difference` — discovery queries; the Brave results surfaced Adobe-community statements that Live Shapes arrived in Photoshop CC, not CS6.

## Open questions

- **CS6 vs CC "Live Shape" properties.** The brief says live shape properties were new/expanded in CS6; the CS6 Help does not document interactive corner-radius editing and community sources attribute Live Shapes to CC. *Resolves with:* a CS6 screenshot/Help page confirming or refuting, and a correction in `OVR-002` if needed.
- **Stroke mechanism precedence.** CS6 shows both options-bar vector stroke and `Layer > Layer Style > Stroke`; which one applies to which tools, and how they serialize to PSD, is not sourced. *Resolves with:* the CS6 Help stroke section and a CS6-authored PSD corpus.
- **Fill/Stroke Align default** (inside/center/outside) and **Line weight default** are not stated by the Help. *Resolves with:* a clean CS6 install reference.
- **Maximum Sides for Polygon** is not documented. *Resolves with:* the CS6 geometry dialog behavior.
- **PSD serialization of vector fill/stroke appearance and Align Edges.** The relevant additional-layer keys are not in the fetched excerpts. *Resolves with:* the Adobe File Formats Specification and a CS6 PSD corpus.
- **Anti-aliased vector-edge rendering algorithm** (and how Align Edges suppresses it) is closed. *Resolves with:* the screenshot-diff harness and calibrated tolerances.
- **Gradient/Pattern fill representation on a vector layer** vs a legacy fill layer needs a source. *Resolves with:* CS6 PSD inspection.
