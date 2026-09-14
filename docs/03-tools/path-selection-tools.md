# Path Selection Tools

- **Spec ID:** `TOOL-054` (Path Selection), `TOOL-055` (Direct Selection)
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the tools themselves are unchanged, but CS6 moves path operations into drop-down menus (`Path Operations`, `Path Alignment`, `Path Arrangement`), adds **Constrain Path Dragging**, and lets the Direct Selection drag adjust related segments by default. Selection of vector points is remembered while the document is open.
- **Depends on:** `ARCH-008` document-model (`pictura_core::path`), `ARCH-003` qt6-ui-design, `03-tools/pen-and-path-tools.md`, `03-tools/shape-tools.md`, `02-ui-ux/panels/paths-panel.md`, `08-selection/paths-and-vector-selection.md`.

## CS6 behavior

Two selection tools share the `A` shortcut and operate on paths/vector shapes:

- **Path Selection** (solid/black arrow, often the visible default; the Direct Selection tool may hide it) selects an entire **path component** (for a shape layer, the whole shape). A click anywhere inside the component selects it; with several components, only the one under the pointer is selected. `Show Bounding Box` in the options bar displays the transform bounding box.
- **Direct Selection** (hollow/white arrow) selects **path segments and anchor points**. Click a segment's anchor, or drag a marquee over part of a segment. Alt/Option-clicking inside a path with Direct Selection selects the whole path/component. Holding Ctrl/Cmd (or Cmd) over an anchor temporarily activates it from most other tools.

Selection feedback: selected anchor points are filled squares, unselected are hollow squares, direction points are filled circles; selecting a component/segment reveals its anchors and (for curves) direction lines/points plus a targeting outline.

| Action | Path Selection | Direct Selection |
|---|---|---|
| Select whole component | Click inside | Alt/Option-click inside |
| Select segment/anchor | — | Click anchor or marquee segment |
| Add to selection | Shift-click | Shift-click |
| Move whole path/component | Drag | — |
| Reshape (anchors/handles) | — | Drag anchor/handle |
| Copy while moving | Alt/Option-drag | — |

Documented operations:

- **Move** a path/component by dragging (shift-click to multi-select first). Dragging beyond the canvas keeps the hidden part. Dragging onto another open image copies the path to that image.
- **Change overlap mode** (`Path Operations` drop-down in CS6; buttons in CS5): **Combine Shapes** (CS6; Add To Shape Area in CS5), **Subtract From Shape Area**, **Intersect Shape Areas**, **Exclude Overlapping Shape Areas**, and **Merge Shape Components** (CS6; Combine in CS5) to merge overlapping components into one.
- **Align/distribute** the selected components of a single path via `Path Alignment` / `Path Arrangement` (CS6; buttons in CS5). Use the Move tool to align shapes on separate layers.
- **Copy/paste** components between documents; **delete** a component with Backspace/Delete; duplicate a path via the Paths panel New Path button (Alt/Option-drag to rename).
- **Show/hide** the target path via `View > Show > Target Path` (or `View > Extras`), and `Ctrl/Cmd+Shift+H`.
- Editing a path segment with Direct Selection changes adjacent segments too in CS6 unless **Constrain Path Dragging** is enabled.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel: Path Selection | Tool | `A` (Shift+`A` cycles) | Black arrow |
| Tools panel: Direct Selection | Tool | `A`, Shift+`A` | White arrow |
| Options bar: Show Bounding Box | Toggle | n/a | Path Selection |
| Options bar: Path Operations drop-down | Menu | n/a | Combine/Subtract/Intersect/Exclude/Merge |
| Options bar: Path Alignment / Path Arrangement | Menus | n/a | Align/Distribute components |
| Options bar: Constrain Path Dragging | Toggle | n/a | CS6 |
| Options bar: Align Edges | Toggle | n/a | CS6, path/vector |
| Paths panel | Dock | n/a | Named/working paths, fill/stroke, make selection |
| `View > Show > Target Path` | Menu | `Ctrl+Shift+H` | Toggle outline |
| Context menu (right-click on vector layer) | Menu | n/a | Copy/Paste Fill or Complete Stroke; Copy/Paste Shape Attributes |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Show Bounding Box | toggle | off | on/off | Path Selection |
| Constrain Path Dragging | toggle | off | on/off | CS6 |
| Align Edges | toggle | on | on/off | Per vector layer |
| Path operation | enum | — | Combine, Subtract, Intersect, Exclude, Merge | CS6 drop-down |
| Path alignment | enum | — | Top, Vertical Centers, Bottom, Left, Horizontal Centers, Right | Single path only |
| Path arrangement | enum | — | Top, Vertical Centers, Bottom, Left, Horizontal Centers, Right | Requires ≥ 3 components |

## Algorithms & pipeline

Behavioral parity target: hit-testing, selection sets, and drag semantics must match CS6 for anchors/segments/components; the exact pick radius and marquee semantics are **behavioral parity only, algorithm TBD**.

- **Hit testing:** point-in-component test uses the path fill rule (ray casting against the flattened path); segment picking uses nearest-point-on-Bézier within a screen-space pick radius (`kurbo::ParamCurveNearest`); anchor picking uses distance to anchor ≤ pick radius; direction-handle picking uses distance to the handle point.
- **Selection model:** a set of `AnchorId`/`SegmentRef`/`ComponentId`; selection persists for the document session (CS6 remembers vector selections). Marquee selection selects anchors/segments whose control geometry intersects the marquee.
- **Drag semantics:** Direct Selection drag on an anchor moves that anchor; on a handle moves that handle (and its opposite if smooth); on a segment moves the segment. With Constrain Path Dragging off, adjacent related segments adjust (CS6 default); on, only segments between selected anchors move.
- **Boolean/align/distribute:** operate over component areas (fill rule) or component bounds; these are the same operations exposed via `pictura-vector::boolean` in `03-tools/pen-and-path-tools.md`.

## Rust module mapping

- `pictura_vector::select` — `SelectionSet`, `ComponentId`, `AnchorId`, `SegmentRef`, marquee hit-testing.
- `pictura_vector::hit` — anchor/handle/segment/component hit tests over `kurbo` geometry with screen-space tolerances.
- `pictura_vector::editor` — drag semantics and the Constrain Path Dragging switch (shared with the pen spec).
- `pictura_vector::boolean` — Combine/Subtract/Intersect/Exclude/Merge.
- `pictura_vector::align` — component align/distribute over component bounds.
- `pictura-qt` bridge exposes the current `SelectionSet` to the overlay for painting.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `PathSelectionToolHandler` | `QObject` | Pointer handling for whole-component selection/move |
| `DirectSelectionToolHandler` | `QObject` | Anchor/segment/handle selection and drag |
| `PathSelectionOverlay` | `QGraphicsItem` | Anchor squares, direction handles, targeting outline, marquee rectangle |
| `PathOpsToolBar` | `QWidget` | Path Operations / Alignment / Arrangement drop-downs, Show Bounding Box, Constrain Path Dragging, Align Edges |
| `PathsPanel` | `QDockWidget` | Path list, fill/stroke/make-selection actions |

Preview geometry uses `QPainterPath`; the canvas `QGraphicsView` supplies mapping and transforms.

## Data-model impact

- Selection state is transient UI state, not persisted in the document (but preserved across tool switches and layer edits within a session, matching CS6).
- Boolean/align/distribute/merge are structural edits to a path node and produce one undo record each.
- `Align Edges` is a per-layer/document vector flag (see `03-tools/shape-tools.md`).
- Copy/paste of components uses the document clipboard in a vector form (path data), not a rasterized representation.

## Edge cases

- **Empty selection.** Move/align/distribute are no-ops; distribute requires ≥ 3 components.
- **Multiple components vs one path.** Operations apply only within a single path; cross-layer alignment uses the Move tool.
- **Open components.** Area operations treat them as implicitly closed; moving/aligning works on bounds regardless.
- **Sub-pixel and zoom.** Pick tolerances must be screen-space (constant on-screen) so selection does not become impossible when zoomed out.
- **Hidden/locked layers.** Selection and editing respect layer locks.
- **Very dense paths.** Hit-testing must be bounded; maintain a spatial index or cache flattened segments per pointer event.
- **GPU fallback.** Overlay painting falls back to raster.

## Parity acceptance criteria

1. Given a multi-component path, Path Selection selects only the component under the pointer; Shift adds components.
2. Given Direct Selection, clicking an anchor selects it and reveals its direction handles for a curve; marquee selects all anchors whose geometry intersects the marquee.
3. Given a selected component, dragging moves the whole component without changing its shape; Alt/Option-drag copies it.
4. Given overlapping components and `Merge Shape Components`, the result is a single component whose fill area is their union, and the operation is one undo step.
5. Given three components and a `Path Arrangement` distribute option, their bounds are evenly distributed along the chosen axis within tolerance.
6. Given `Constrain Path Dragging` off, dragging one segment adjusts the related segments; on, only segments between the selected anchors move.
7. Given vector points selected, switching to another layer and back preserves the selection while the document is open.
8. Given a vector layer whose Target Path is hidden with `Ctrl/Cmd+Shift+H`, clicking it with Path/Direct Selection restores the outline.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — established: Path Selection selects a whole component by clicking inside (Alt-click for Direct Selection); Direct Selection selects segments/anchors via click or marquee; Shift adds; Ctrl/Cmd temporarily activates Direct Selection; selection highlighting (filled/hollow squares, filled circles); Show Bounding Box; move/copy/delete components; CS6 `Path Operations` (Combine/Subtract/Intersect/Exclude/Merge Shape Components), `Path Alignment` and `Path Arrangement` drop-downs; align/distribute only within one path; `View > Show > Target Path` / `View > Extras`; Constrain Path Dragging.
- `https://bjango.com/articles/photoshopcs6vectorshapes` — CS6 remembers vector point selections while the document is open; Cmd/Ctrl+J duplicates selected points; Path/Direct Selection clicking deselects; boolean path options moved in CS6.
- `https://doc.qt.io/qt-6/qpainterpath.html` — path hit-testing (`contains`, `intersects`), boolean ops, fill rule, `elementAt`/`elementCount`, `setElementPositionAt`.
- `https://docs.rs/kurbo` — nearest-point and segment iteration used for hit-testing.
- `https://html.duckduckgo.com/html/?q=Photoshop+CS6+pen+tool+freeform+pen+magnetic+add+delete+anchor+point+convert+point` — discovery for the secondary sources above.

## Open questions

- **Exact pick radius** (screen px) for anchors, handles, and segments in CS6 is undocumented. *Resolves with:* controlled selection tests at multiple zoom levels.
- **Marquee inclusion rule** (intersect vs fully-enclosed) is not stated. *Resolves with:* CS6 gesture captures.
- **Whether Constrain Path Dragging has a per-tool or global default** is unclear. *Resolves with:* the options-bar default and preference documentation.
- **Cross-document copy format** (vector payload vs rasterization) for pasting paths between Photoshop documents and other apps. *Resolves with:* the File Formats Specification path clipboard behavior and tests.
- **Performance ceiling for marquee selection on dense paths.** *Resolves with:* the performance benchmarks in `11-cross-cutting/`.
