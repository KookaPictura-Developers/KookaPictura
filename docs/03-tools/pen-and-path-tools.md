# Pen and Path Tools

- **Spec ID:** `TOOL-052` (Pen, Freeform Pen, Magnetic Pen, Add/Delete Anchor Point, Convert Point), `TOOL-053` (path-drawing behavior: rubber band, auto add/delete, editing)
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — CS6 adds **Align Edges** and **Constrain Path Dragging** options bar controls, moves boolean/path operations into CS6 drop-down menus (`Merge Shape Components`, `Path Operations`, `Path Alignment`, `Path Arrangement`), and adds true vector fill/stroke appearance. The Pen/Freeform/Magnetic/Anchor/Convert tool set and drawing gestures are unchanged from CS5.
- **Depends on:** `ARCH-008` document-model (`pictura_core::path`, `Path` nodes), `ARCH-002` rust-core-design, `ARCH-003` qt6-ui-design, `03-tools/path-selection-tools.md`, `03-tools/shape-tools.md`, `02-ui-ux/panels/paths-panel.md`, `08-selection/paths-and-vector-selection.md`, `05-layers/vector-masks-and-clipping-masks.md`.

## CS6 behavior

Paths are vector outlines of straight and curved segments. Anchor points mark segment ends; curved segments carry direction lines ending in direction points. A path is a **work path** (temporary, in the Paths panel) until saved. Paths can become selections, vector masks, clipping paths, or be filled/stroked with pixels. A path may have multiple disconnected **path components** (e.g. one per shape in a shape layer).

The pen tool group (all share the `P` slot; cycle with `Shift+P`):

| Tool | Behavior |
|---|---|
| **Pen** | Click creates corner points and straight segments; drag creates smooth points and direction lines. Shift constrains to 45°. Close a path by clicking the first (hollow) anchor (circle icon appears); leave open with Ctrl/Cmd-click away or by switching tools. Alt-drag breaks/splits direction lines. |
| **Freeform Pen** | Drag freehand; anchor points are added automatically. **Curve Fit** (0.5–10.0 px) controls sensitivity: higher = simpler path, fewer anchors. Continue an existing path by dragging from an endpoint; close by dragging onto the start point (circle indicator). |
| **Magnetic Pen** | A **Freeform Pen option** (Magnetic checkbox in the options bar), not a separate tool. Draws a path that snaps to image edges: click to set the first **fastening point**, move/drag along the edge, click to place fastening points manually, Delete removes the last. Double-click closes with a magnetic segment, Alt/Option-double-click closes with a straight segment, Enter/Return ends open. |
| **Add Anchor Point** | Click a path segment to add an anchor. |
| **Delete Anchor Point** | Click an anchor to remove it. |
| **Convert Point** | Drag out of a corner to make a smooth point; click a smooth point to make a corner; drag a direction point to make independent handles. |

**Auto Add/Delete** (Pen options bar, on by default) makes the Pen switch to Add Anchor Point over a segment and Delete Anchor Point over an anchor. Deselect Auto Add/Delete to override this (useful when starting a new path on top of an existing one). **Rubber Band** (Pen options pop-up) previews segments between clicks. The Convert Point tool is reachable from the Pen by holding Alt/Option, and from Direct Selection by Ctrl+Alt/Cmd+Option.

### Drawing/editing gestures documented by CS6 Help

- **Straight segments:** click, click, … — the last anchor is solid (selected), previous anchors hollow.
- **Curves:** press and drag from the anchor; direction line length/slope sets the curve. Drag **opposite** the previous direction line for a C-curve, the **same** direction for an S-curve. Extend the direction line about one third of the distance to the next anchor as a rule of thumb.
- **Straight→curve / curve→straight / curve-corner-curve:** documented step sequences; Alt/Option-drag splits direction lines at a corner.
- **Editing:** select a component with Path Selection; select a segment/anchor or marquee with Direct Selection. Move a segment, change straight-segment length/angle, reshape curves via anchors/direction points; Shift constrains to 45°.
- **Extend / connect:** hover an open path's endpoint, click, then click the other path's endpoint (merge symbol appears) to join; or draw a new path and click the existing endpoint.
- **Nudge:** arrow keys move 1 px; Shift+arrow moves 10 px (applies to anchors/segments).
- **Delete a segment:** Direct Selection + Backspace/Delete (again deletes the rest of the path). Note: do **not** use Delete/Backspace or Cut/Clear to delete *anchor points* — those delete the connecting segments too; use the Delete Anchor Point tool.
- **Cut an opening in a closed path:** add two anchors with Add Anchor Point, then delete the segment between them.

### Magnetic Pen options (shared with Magnetic Lasso)

| Option | Type | Range | Meaning |
|---|---|---|---|
| Width | px | 1–256 | Edge-detection distance from the pointer |
| Contrast | % | 1–100 | Required contrast for an edge (higher for low-contrast images) |
| Frequency | 0–100 | | Rate at which anchor points are set (higher anchors faster) |
| Pen Pressure | toggle | — | Stylus pressure reduces width (requires tablet) |

Dynamic modifiers: Alt/Option-drag = freehand; Alt/Option-click = straight segment; `[` / `]` decrease/increase width by 1 px.

### CS6 changes that touch the pen/path tools

- **Align Edges** (options bar) reduces anti-aliased edges by aligning the object edge to the pixel grid; with **Snap Vector Tools and Transforms to Pixel Grid** (global preference, on by default) anchor points and dragged paths snap to the pixel grid. Nudging behavior changes with snapping: with snapping on, nudge always moves exactly 1 px; with it off, the nudge is tied to zoom (1 px at 100%, 0.5 px at 200%, …).
- **Constrain Path Dragging** (options bar, CS6) restricts a Direct Selection drag to the segments between the selected anchors, matching pre-CS6 behavior. By default CS6 adjusts the related segments too (a more "Illustrator-like" transform).
- Path selections are **remembered while the document is open**, and `Shape Layer via Copy` (Ctrl/Cmd+J) duplicates the *selected points*. Clicking the path with Path/Direct Selection deselects all vector points.
- `Cmd/Ctrl+Shift+H` hides the Target Path outline (helps edit effects near a vector edge); clicking the canvas with Path/Direct Selection brings it back.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel: Pen | Tool | `P` (Shift+`P` cycles) | Click/drag drawing |
| Tools panel: Freeform Pen | Tool | `P`, Shift+`P` | Curve Fit option |
| Tools panel: Add Anchor Point | Tool | — | Auto via Pen+Auto Add/Delete |
| Tools panel: Delete Anchor Point | Tool | — | Auto via Pen+Auto Add/Delete |
| Tools panel: Convert Point | Tool | — | Alt from Pen; Ctrl+Alt from Direct Selection |
| Pen/Freeform options bar: Magnetic | Option | — | Converts Freeform Pen to Magnetic |
| Pen/Freeform options bar: Auto Add/Delete | Option | — | Pen only |
| Pen options pop-up: Rubber Band | Option | — | Preview next segment |
| Magnetic options: Width/Contrast/Frequency/Pen Pressure | Options | `[` `]` | Width ±1 px |
| Paths panel | Dock | n/a | Work path, save, fill/stroke, make selection, clipping path |
| `View > Show > Target Path` | Menu | `Ctrl+Shift+H` | Toggle active path outline |
| `Edit > Define Custom Shape` | Menu | n/a | See `03-tools/custom-shape.md` |
| `Filter > Vanishing Point` | Dialog | `Ctrl+Alt+V` | Separate path-like authoring surface |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Curve Fit (Freeform) | px | 2.0 (Help gives range only) | 0.5–10.0 | Higher = simpler path |
| Magnetic Width | px | 10 (Help gives range only) | 1–256 | Dynamic `[` `]` |
| Magnetic Contrast | % | 20 (Help gives range only) | 1–100 | Higher for low contrast |
| Magnetic Frequency | int | 60 (Help gives range only) | 0–100 | Anchor rate |
| Pen Pressure | toggle | off | on/off | Tablet only |
| Auto Add/Delete | toggle | on | on/off | Pen |
| Rubber Band | toggle | off | on/off | Pen |
| Constrain Path Dragging | toggle | off | on/off | CS6 |
| Align Edges | toggle | on | on/off | CS6; per layer |
| Snap Vector Tools and Transforms to Pixel Grid | toggle (pref) | on | on/off | Global |
| Fill Path: Mode | enum | Normal | 27 blend modes | Includes Clear |
| Fill Path: Opacity | percent | 100 | 0–100 | |
| Fill Path: Feather Radius | px | 0 | ≥ 0 | |
| Fill Path: Anti-aliased | toggle | on | on/off | |
| Stroke Path: tool | enum | current tool | Brush, Pencil, Eraser, … | |
| Stroke Path: Simulate Pressure | toggle | off | on/off | Wacom only |
| Path operation (Path Selection) | enum | — | Combine/Add, Subtract, Intersect, Exclude, Merge Shape Components (CS6) | CS5 used buttons |
| Path alignment/arrangement | enum | — | Align 6 ways; Distribute 6 ways | CS6 drop-downs |

## Algorithms & pipeline

### Path representation and drawing (proposed)

Behavioral parity target: given the same pointer stream in the same document, the path geometry (anchor positions, direction lines, curve fit) matches CS6 within sub-pixel tolerance, and the editing gestures produce the same topology (which segments/anchors move, where points are inserted). Adobe's exact curve fitting and magnetic-edge heuristics are closed; those are **behavioral parity only, algorithm TBD**.

A Bézier-path engine is the natural representation (`kurbo::BezPath` here, mirroring the PSD path format used by `ARCH-008`: 8.24 fixed-point records, nonzero/winding fill):

- **Representation:** ordered subpaths; each element is `MoveTo`, `LineTo`, `QuadTo`, or `CubicTo`; anchors are derived (segment endpoints) with optional paired direction handles (smooth) or independent handles (corner).
- **Pen state machine:** `Idle → Pressed(anchor pending) → Dragging(direction handle) → Committed`. Distinguish click (corner) from drag (smooth) by pointer movement beyond a small threshold during press. Closing tested by proximity to the first anchor of the active subpath.
- **Rubber band:** on cursor move between clicks, render a provisional segment from the last anchor to the cursor (a straight preview, or a curve preview using the current handle when present).
- **Freeform:** sample the pointer, simplify to a Bézier path with a tolerance from Curve Fit. This is a curve-fitting problem (fit a series of cubics to sample points with error bound ε); `kurbo::fit_to_bezpath_opt`/`ParamCurveFit` is the candidate.
- **Smooth-point constraint:** moving one handle of a smooth anchor moves the opposite handle to maintain collinearity (equal and opposite direction), unless the handles are independent (corner). CS6 documents that editing an existing smooth point with Direct Selection changes only the dragged side, while drawing with the Pen changes both.
- **Auto Add/Delete:** hit-test the cursor against segments (distance to curve ≤ pick radius) and anchors (distance ≤ pick radius); switch tool mode accordingly when Auto Add/Delete is on.
- **Anchor insert:** insert at nearest curve parameter `t`, splitting the cubic into two cubics that preserve the original geometry (de Casteljau); delete merges adjacent segments when possible.
- **Convert point:** corner→smooth computes tangent from neighbours; smooth→corner removes/zeroes handles; independent-handles mode detaches the opposite handle.
- **Snapping:** when Snap Vector Tools and Transforms to Pixel Grid is on, quantize anchor/path coordinates to the pixel grid; Align Edges additionally quantizes the whole-object vertical/horizontal edges. Nudging applies the documented 1 px / zoom-dependent rule.

### Magnetic Pen (proposed heuristic)

The Magnetic Pen snaps to the strongest edge within `Width` px. A standard approach: sample a perpendicular line profile around the cursor, compute a gradient/contrast response, accept edges whose contrast ≥ `Contrast`%, and place an anchor when accumulated path length/curvature exceeds `Frequency`-derived spacing. This is *inferred*, not sourced; see Open questions.

### Path operations

Boolean combine/subtract/intersect/exclude and `Merge Shape Components` operate on path fill areas with the path's fill rule. Non-closed paths are treated as implicitly closed for area operations. Fill rule is winding/nonzero in Photoshop paths (PSD), so boolean output must preserve or recompute winding. `QPainterPath` implements these as `united`, `subtracted`, `intersected`, and `QPainterPath::simplified` (recursively removes overlapping subpaths), and `QT`'s `OddEvenFill`/`WindingFill` maps to Photoshop's fill rule for previews.

## Rust module mapping

Proposed crate `pictura-vector` (new; depends on `pictura-core`, `pictura-text` for outlines):

- `pictura_vector::path` — `VectorPath` (subpaths, elements), fill rule, segment iteration, `to_kurbo`/`from_kurbo`, 8.24 fixed-point conversion for PSD `Path` records.
- `pictura_vector::anchors` — derived anchor/handle model, smooth/corner classification, insert/delete/split/convert operations (de Casteljau).
- `pictura_vector::editor` — active-path editing session: selection sets (anchors, segments, components), drag semantics, and the CS6 Constrain Path Dragging switch.
- `pictura_vector::pen` — Pen tool state machine (click/drag, rubber band, close/open, extend/connect).
- `pictura_vector::freeform` — pointer sampling and Bézier curve fitting (`kurbo` fit API), Curve Fit → tolerance mapping.
- `pictura_vector::magnetic` — edge-snapping tracer (image gradient sampling, Width/Contrast/Frequency, fastening points).
- `pictura_vector::boolean` — combine/subtract/intersect/exclude/merge over path areas, fill-rule aware.
- `pictura_vector::stroke` — stroke-to-outline, caps/joins/miter/dash (`kurbo::stroke`, `kurbo::dash`).
- `pictura_vector::snap` — pixel-grid snapping and Align Edges quantization.

Crossing types: `VectorPath`, `AnchorId`, `SegmentRef`, `Vec2`/`Point` in document coordinates; `kurbo::BezPath` is the internal geometry. The document-model `Path` node (already proposed in `ARCH-008`) holds `Vec<VectorPath>` plus name/clipping-path flags.

Crates verified: `kurbo` 0.13 (Bézier geometry, stroke, dash, fit, nearest-point). A dedicated boolean-operation crate (`lyon`/`geo-booleanop`) is a candidate but was not verified here; keep the boolean seam behind `pictura_vector::boolean`.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `PathToolHandler` | `QObject` | Pen/Freeform/Magnetic/Anchor/Convert state machine; routes pointer events |
| `PathEditOverlay` | `QGraphicsPathItem` tree | Anchor squares, direction handles/lines, rubber-band preview, selection highlight |
| `PenOptionsBar` | `QWidget` | Mode (Shape/Path/Pixels), Auto Add/Delete, Rubber Band, Curve Fit, Magnetic group, Align Edges, Constrain Path Dragging |
| `MagneticOptionsWidget` | `QWidget` | Width/Contrast/Frequency/Pen Pressure |
| `PathOperationsMenu` | `QMenu`/`QToolButton` | Combine/Subtract/Intersect/Exclude/Merge, Alignment, Arrangement (CS6 drop-downs) |
| `PathsPanel` | `QDockWidget` + `QAbstractListModel` | Work path, named paths, fill/stroke, make selection, clipping path |
| `FillStrokePathDialog` | `QDialog` | Fill/stroke options, feather/anti-alias, simulate pressure |

Preview geometry is exchanged as `QPainterPath` (built from `pictura-vector` output); `QGraphicsView` handles hit-testing and transforms per `ARCH-003`. `QPainterPathStroker` is the reference for stroke previews (width, cap, join, dash, miter limit).

## Data-model impact

- `ARCH-008` already models `pictura_core::path::BezierPath`, subpaths, fill rules, and the PSD 8.24 fixed-point/26-byte path records. This spec adds: the anchor/handle editing model, the selection set (transient UI state, not serialized), and the per-layer Align Edges flag.
- Shape layers and vector masks reference a path; a path component per shape means a shape layer's vector mask is one `VectorPath` with multiple subpaths.
- `Edit > Define Custom Shape` exports a path to the shapes preset library (see `03-tools/custom-shape.md`), stored as a `.CSH` preset, not document data.
- Work path → saved path → named path transitions and clipping-path naming are Paths-panel commands with undo records.
- Undo granularity: one record per committed draw (path created/modified), per anchor add/delete/convert, and per structural path edit (component move/copy/delete). Drag sessions coalesce into one record.
- Path fill/stroke are destructive pixel operations on a raster layer (unless done as a shape layer's fill/stroke appearance); their undo follows `ARCH-009` pixel-backup records.

## Edge cases

- **Open vs closed paths.** Filling an open path implicitly closes it; stroking an open path does not. Boolean operations treat non-closed paths as implicitly closed.
- **Self-intersecting paths.** Fill rule (nonzero/winding vs even-odd) changes the result; Photoshop uses winding.
- **Single anchor / zero-length segment.** Must not panic; insert/delete/convert are no-ops.
- **Empty path / path with only `MoveTo`.** `QPainterPath::isEmpty` semantics; work-path to selection yields an empty selection.
- **Path beyond canvas.** Off-canvas portions are retained (not clipped) so panning re-reveals them.
- **Very many anchors.** Hit-testing and boolean ops must stay within interactive budgets; avoid O(n²) per event.
- **8/16/32-bit and CMYK/Lab.** Paths are resolution/color-independent outlines, but Fill/Stroke Path render at the document depth and color space; feather/anti-alias must respect depth.
- **Raster layers locked or non-raster target.** Cannot fill/stroke a path on a mask, text, fill, adjustment, or Smart Object layer.
- **GPU-unavailable fallback.** Overlay and stroke previews fall back to the CPU/raster path.
- **Undo across tools.** Switching tools mid-draw should commit or cancel cleanly and leave history consistent.
- **PSB scale.** Coordinate math in document pixels up to 300,000; use `f64` geometry and checked conversions to the 8.24 fixed point.
- **Pixel snapping interaction.** With snapping on, editing exactly on a 0.5 px boundary is documented as possible but fragile; preserve sub-pixel coordinates internally.

## Parity acceptance criteria

1. Given a click-drag-click sequence, the Pen produces one smooth point (drag) and one corner point (click) whose direction lines match the drag vectors within sub-pixel tolerance.
2. Given two clicks on the first anchor, the path closes; given a switch to another tool, it stays open.
3. Given the Rubber Band option on, a provisional segment is visible between the last anchor and the cursor before the next click.
4. Given a freehand drag, increasing Curve Fit reduces the resulting anchor count monotonically for the same input stream.
5. Given Auto Add/Delete on, clicking a Pen over a segment inserts an anchor and clicking over an anchor deletes it; with it off, the Pen starts a new path instead.
6. Given a smooth anchor, dragging one direction handle moves the opposite handle to remain collinear; given a corner anchor, only the dragged side changes.
7. Given Constrain Path Dragging on, Direct Selection of a segment moves only the segments between the selected anchors, matching CS5; off, adjacent segments adjust.
8. Given `Align Edges` on and a snap-grid-aligned rectangle, its vertical/horizontal edges have no anti-aliased fringe; with it off, an anti-aliased fringe appears.
9. Given Snap Vector Tools and Transforms to Pixel Grid on, a nudge moves exactly 1 px regardless of zoom; off, the nudge distance follows the documented zoom table (1 px at 100%, 0.5 px at 200%, …).
10. Given the Magnetic Pen tracing a high-contrast edge with Width/Contrast within range, fastening points land on the edge and Delete removes the last fastening point; double-click closes with a magnetic segment and Alt-double-click with a straight segment.
11. Given Fill Path with feather F px and anti-aliasing on, the boundary transition width matches a reference within tolerance and no fill appears outside the layer bounds.
12. Given a path and `Merge Shape Components`, all overlapping components become one component whose fill area is the union.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — official Photoshop CS6 Help. Established: pen tool family and shortcuts; straight/curved drawing sequences and C/S curves; Alt/Option direction-line splitting; path completion (close/open), extend and connect; Auto Add/Delete and Rubber Band; Convert Point gestures; Add/Delete Anchor Point and the "don't use Delete/Cut for anchors" warning; selection via Path/Direct Selection; nudge (1 px / Shift 10 px); segment/anchor deletion; Freeform Pen Curve Fit range 0.5–10.0 and endpoint continue/close; Magnetic Pen options (Width 1–256, Contrast 1–100, Frequency 0–100, Pen Pressure) and dynamic `[`/`]`, Alt-drag/Alt-click, Delete last fastening point, Enter/double-click/Alt-double-click completion; work path definition; fill/stroke path options (mode/opacity/feather/anti-alias/Simulate Pressure); CS6 Merge Shape Components / Path Operations / Path Alignment / Path Arrangement drop-downs; `View > Show > Target Path`.
- `https://bjango.com/articles/photoshopcs6vectorshapes` — CS6 pixel snapping moved to the global preference **Snap Vector Tools and Transforms to Pixel Grid**; nudge-vs-zoom table (1 px at 100%, 0.5 at 200%, 0.33 at 300%, …); **Align Edges** aligns layer edges to the pixel grid while non-edge contents scale unsnapped; vector selections remembered while the document is open and Ctrl/Cmd+J duplicates selected points; `Cmd+Shift+H` hides the Target Path.
- `https://bjango.com/articles/photoshopcs6` — corroborates Align Edges vs Snap Vector Tools/Transforms and that Align Edges preserves non-edge point relationships.
- `https://planetphotoshop.com/cs6-vector-tools.html` — CS6 vector-layer workflow, shape/path/pixels mode selector, Align Edges removes anti-aliasing on vertical/horizontal edges, vector mask via Properties panel (Density/Feather).
- `https://docs.rs/kurbo` — `kurbo` 0.13 geometry API: `BezPath`, `PathEl`, `stroke`, `dash`, `fit_to_bezpath*`, `ParamCurveFit`, `ParamCurveNearest`, path offset/simplify modules.
- `https://doc.qt.io/qt-6/qpainterpath.html` — path composition (`moveTo`/`lineTo`/`cubicTo`/`quadTo`), `ElementType`, fill rule, `contains`/`intersects`, boolean ops (`united`/`subtracted`/`intersected`/`simplified`), `toFillPolygons`, `setElementPositionAt`.
- `https://doc.qt.io/qt-6/qpainterpathstroker.html` — stroke outline generation, width/cap/join/dash/miter, `WindingFill` requirement.
- `https://html.duckduckgo.com/html/?q=Photoshop+CS6+pen+tool+freeform+pen+magnetic+add+delete+anchor+point+convert+point` — discovery for the secondary sources above.

## Open questions

- **Magnetic Pen algorithm.** Edge detection, contrast normalization, and anchor-spacing heuristics are undocumented. *Resolves with:* a controlled edge corpus and comparison against CS6 traces, or a published description.
- **Freeform Pen curve-fit algorithm and default Curve Fit value.** The Help gives the range but not the default or the fitting method. *Resolves with:* captured CS6 sessions across Curve Fit values.
- **Exact nudge/snap rounding at fractional zooms.** Bjango notes an apparent 0.5 px nudge at 66.7% that looks like a bug. *Resolves with:* a documented decision on whether to reproduce it.
- **Which boolean library** is adequate (correctness on self-intersections, robustness) is unverified. *Resolves with:* an evaluation of `lyon`/`geo-booleanop`/`kurbo` against a path corpus.
- **Fill-rule and winding semantics on boolean output** need confirmation against PSD `Path` records. *Resolves with:* a PSD path corpus and `psd-tools`/CS6 readback.
- **Pen behavior over an existing path with Auto Add/Delete off** during a shape-layer draw (CS6 adds shape area on Shift) needs a reference. *Resolves with:* captured CS6 gesture tests.
- **Whether CS6's "Intuitive path editing" default (adjusting related segments) applies to all vector objects or only paths/shapes** is not fully specified. *Resolves with:* the CS6 Help `Adjust path components` section and controlled tests.
- **Performance targets for very dense paths** (thousands of anchors) are not specified. *Resolves with:* `ARCH-006`/`11-cross-cutting/performance-targets` benchmarks.
