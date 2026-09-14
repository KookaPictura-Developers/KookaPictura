# Custom Shape Tool

- **Spec ID:** `TOOL-058` (Custom Shape tool and drawing), `TOOL-059` (shape libraries, presets, defining custom shapes)
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the Custom Shape tool inherits CS6's vector-layer/fill-and-stroke overhaul and the Shape/Path/Fill-Pixels mode selector; custom shapes are drawn as fully vector objects with editable Fill/Stroke. The preset picker, shape sets, and `Edit > Define Custom Shape` workflow are carried from earlier versions.
- **Depends on:** `ARCH-008` document-model (`Shape` node), `03-tools/shape-tools.md`, `03-tools/pen-and-path-tools.md`, `03-tools/path-selection-tools.md`, `10-workflow-io/presets-manager.md`, `02-ui-ux/panels/tool-presets-panel.md`, `05-layers/vector-masks-and-clipping-masks.md`.

## CS6 behavior

The **Custom Shape tool** (nested at the bottom of the shape tool fly-out; `U`, `Shift+U`) draws arbitrary preset outlines chosen from the **Custom Shape picker** in the options bar.

- The picker shows thumbnail previews of the current shapes; scroll or resize the picker (drag its bottom-right corner) to see more.
- Only a handful of shapes load by default. The picker's **gear icon** menu lists the bundled shape sets (for example Animals, Music, Nature, …) and an **All** entry that loads every set. Loading prompts to **Replace** (show only the new category) or **Append** (add below the existing shapes). **Reset Shapes** restores the defaults.
- Selecting a shape is a double-click on its thumbnail (or a single click, depending on the build); the chosen shape then draws by click-drag.
- The tool honor's CS6's **Shape / Path / Fill Pixels** mode selector. In Shape mode each custom shape lands on its own shape layer with editable Fill and Stroke.
- Draw modifiers: **Shift** constrains to the shape's designed proportions; **Alt/Option** draws from the center; **Shift+Alt/Option** both. Release keys after the mouse button.
- Drawing options for the Custom Shape tool: Unconstrained, Fixed Size (W×H), Proportional (W/H), **Defined Proportions** (renders using the proportions with which the shape was created), **Defined Size** (renders at the size it was created), and From Center.
- Clicking once (instead of dragging) opens the shape's geometry dialog (**Create Custom Shape**); entering W/H and clicking OK draws the shape at the click point. With From Center on, the click maps to the shape's center.
- After drawing, W/H fields in the options bar show the shape size; the link icon locks the aspect ratio when editing one value.

### Defining and saving custom shapes

- `Edit > Define Custom Shape` (with a path selected in the Paths panel — a shape layer's vector mask, a work path, or a saved path) prompts for a name and adds the shape to the Shape picker. A raster selection cannot be defined directly; it must first be converted to a path.
- **Save Shapes** from the picker's pop-up menu saves the new shape as part of a shape library.
- **Preset Manager** (`Edit > Preset Manager`) manages shape sets: set **Preset Type** to **Custom Shapes**, select thumbnails (click; Shift-click for a range; Ctrl/Cmd-click for a discontiguous set), then **Save Set** to write a `.CSH` file, **Load** to load one, and `Reset Custom Shapes` to restore defaults. Thumbnail size and list view are configurable.
- Custom shapes appear in the picker/preset manager in black regardless of the color later applied in the document.

### Where custom shapes are used

- The picker also appears for the Pen tool and other shape tools when their Shape/Pixels geometry is chosen; `Custom Shape` is one of the shape tool options.
- A custom shape is geometry only (a path); it has no intrinsic color. Color/gradient/pattern and stroke come from the layer's Fill/Stroke appearance.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel: Custom Shape | Tool | `U` (Shift+`U`) | Bottom of shape fly-out |
| Options bar: mode selector | Buttons | n/a | Shape / Path / Fill Pixels |
| Options bar: Custom Shape picker thumbnail | Pop-up | n/a | Grid of shape thumbnails |
| Shape picker: gear menu | Menu | n/a | Shape sets, All, Replace/Append, Reset Shapes, Save Shapes, Load, Preset Manager |
| Options bar: geometry options | Pop-up | n/a | Unconstrained/Fixed Size/Proportional/Defined Proportions/Defined Size/From Center |
| Options bar: W / H + link | Fields | n/a | Resize after draw |
| Options bar: Fill / Stroke | Pop-ups | n/a | See `03-tools/shape-tools.md` |
| `Edit > Define Custom Shape` | Dialog | n/a | Paths panel path required |
| `Edit > Preset Manager` | Dialog | n/a | Custom Shapes preset type |
| Paths panel | Dock | n/a | Source path for Define Custom Shape |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Selected shape | preset | first default shape | bundled/custom presets | Picker thumbnail |
| Drawing mode | enum | Shape | Shape / Path / Fill Pixels | CS6 selector |
| Geometry | enum | Unconstrained | Unconstrained / Fixed Size / Proportional / Defined Proportions / Defined Size / From Center | |
| Width / Height | number | — | > 0 | Fixed/Proportional/dialog |
| Aspect link | toggle | off | on/off | Locks W:H |
| Fill | enum + value | Solid (Help does not state) | None / Solid / Gradient / Pattern | |
| Stroke | enum + value | None | None / Solid / Gradient / Pattern | |
| Stroke width | number | 3 pt | > 0 | Align Edges needs px |
| Shape name | string | "Custom Shape N"/from dialog | — | Define Custom Shape / Shape Name dialog |
| Shape set | file | bundled defaults | `.CSH` files | Save Set / Load in Preset Manager |

## Algorithms & pipeline

Behavioral parity target: given the same shape preset and draw gesture, the resulting path geometry, proportions, and fill/stroke match CS6 within the tolerance of `11-cross-cutting/testing-strategy.md`. The binary `.CSH` format is Adobe-proprietary and not public; exact file-format parity is **behavioral parity only, format TBD** (see Open questions).

- **Shape representation:** a custom shape is a named vector path (one or more subpaths) with a design-time aspect ratio and size. Defined Proportions/Defined Size modes use those stored attributes; Unconstrained stretches to the drag box.
- **Drawing:** map the drag box (or center/click) to the path's coordinate space with an affine transform; Shift preserves the design aspect; Alt centers. Fill/stroke apply as in `03-tools/shape-tools.md`.
- **Define Custom Shape:** snapshot the selected path (plus its bounds/aspect) into the current shape library.
- **Shape sets:** a library is a collection of named paths. Save Set writes the selected entries to a `.CSH` file; Load appends a `.CSH` to the picker; Reset restores the bundled defaults. The picker is a virtualized thumbnail grid.
- **Pattern fill:** if the shape's fill is Pattern, the layer references a pattern preset; loading patterns uses the same picker gear workflow (`Load Patterns`).

## Rust module mapping

- `pictura_vector::shape` — custom-shape geometry instantiation from a stored path + design bounds/aspect (shared with `03-tools/shape-tools.md`).
- `pictura_vector::preset::shapes` — shape library model: `ShapePreset { name, path: VectorPath, design_bounds: Rect, aspect }`, `ShapeLibrary`, load/save `.CSH` (deferred format), bundled defaults.
- `pictura-prefs`/`pictura-script` — preset manager integration (load/save/reset/replace/append) and migration (`Edit > Presets > Migrate`).
- `pictura-vector::boolean` / `stroke` — downstream operations on the instantiated shape.

Crossing types: `VectorPath` (geometry), `ShapePreset`, `FillSpec`/`StrokeSpec` (appearance, on the document `Shape` node).

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `CustomShapeToolHandler` | `QObject` | Drag/click drawing, Shift/Alt modifiers, geometry modes |
| `ShapePickerPopup` | `QWidget`/`QMenu` | Virtualized thumbnail grid, scroll, resize, gear menu |
| `ShapePresetModel` | `QAbstractItemModel` | Presets for the current library; thumbnail + name roles |
| `ShapePresetDelegate` | `QStyledItemDelegate` | Thumbnail painting (black silhouette), selection highlight |
| `ShapeNameDialog` | `QDialog` | Name entry for Define Custom Shape |
| `PresetManagerDialog` | `QDialog` | Custom Shapes preset type: select, Save Set, Load, Reset, thumbnail size, list view |
| `ShapeLibrary` (Rust-backed) | n/a | Actual load/save/reset logic exposed through `pictura-qt` |

`QPainterPath` renders the thumbnail silhouettes; `QFileDialog` handles `.CSH` open/save. Widgets (not QML) per `ARCH-003`.

## Data-model impact

- A custom shape is **not** document data; it is a document-*independent* preset. Drawing one produces a normal `Shape` node whose vector mask holds the instantiated path.
- `Edit > Define Custom Shape` adds an entry to the user's shape library; `Save Shapes` / Preset Manager writes `.CSH`. These live in the presets store (`10-workflow-io/presets-manager.md`), not in PSD/XMP document metadata.
- The shape's design bounds/aspect must be preserved in the preset so Defined Proportions/Defined Size work; this is not represented in the document once a shape is drawn (the drawn path is the record).
- Undo: drawing a custom shape is a normal shape-layer command; defining a preset is a preset-store change (not document history), though the picker's "replace/append" is session UI state.

## Edge cases

- **No path selected for Define Custom Shape.** The command is unavailable; a raster selection must become a path first.
- **Multi-subpath shapes** (holes, letters with counters) must preserve fill rule/winding; the classic example is a shape with an enclosed counter.
- **Design aspect missing.** Presets created before a format change may lack bounds; fall back to path bounding box.
- **Picker state.** Replace vs Append vs Reset must leave the picker consistent; a missing `.CSH` file or corrupt library must report an error and not clear the current set.
- **Duplicate names.** Defining a shape with an existing name should disambiguate (CS6 behavior TBD).
- **Large libraries** (All sets loaded) require a virtualized, resizable grid; avoid building every thumbnail eagerly.
- **Color independence.** Thumbnails are always black; a shape's actual fill/stroke is per-layer.
- **Aliasing/edge snapping.** Custom-shape edges are subject to Align Edges/Snap to Pixel Grid like other vector shapes.
- **PSB/huge documents.** Custom shapes can be drawn at very large sizes; geometry uses f64.
- **GPU fallback.** Thumbnail grid renders on CPU; canvas falls back to CPU compositing.

## Parity acceptance criteria

1. Given the default picker, loading **All** appends the bundled sets below the defaults; **Replace** shows only the chosen set; **Reset Shapes** restores the defaults.
2. Given a custom shape with Defined Proportions, drawing with Shift produces the design aspect ratio within sub-pixel tolerance; Unconstrained stretches to the drag box.
3. Given a single click with no drag, the Create Custom Shape dialog appears and OK draws the shape at the click position with the entered W/H.
4. Given a path selected in the Paths panel, `Edit > Define Custom Shape` adds a named entry that appears in the picker and draws identically to the source path.
5. Given selected shapes in the Preset Manager, **Save Set** writes a library that **Load** restores with the same names and geometry.
6. Given a multi-subpath custom shape (e.g. a letter with a counter), the drawn shape respects the fill rule so the counter is open.
7. Given a custom shape drawn on a shape layer, changing Fill to a gradient/pattern and adding a dashed stroke updates the shape without redrawing.
8. Given Fill Pixels mode, the custom shape rasterizes on the current layer with foreground color and no vector layer is created.
9. Given a `.CSH` fixture, the library model loads it and exposes the same number/names of presets (format support contingent on the open `.CSH` question).

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — established: Custom Shape tool location and `Shift+U`; draw a custom shape via the Custom Shape pop-up panel; category arrow with Replace/Append; `Edit > Define Custom Shape` + Shape Name dialog; Save Shapes; shape tool options `Defined Proportions` and `Defined Size`; `Unconstrained`; Preset Manager handles custom shapes; Fill/Stroke and rasterized-shape options; the Shape/Path/Fill Pixels modes.
- `https://www.photoshopessentials.com/basics/how-to-use-the-custom-shape-tool-in-photoshop-cs6` — CS6-specific workflow: setting mode to Shape, the picker thumbnail and grid, the gear icon and shape sets (Animals/Music/Nature/All), Replace vs Append vs Reset Shapes, resizable picker, double-click selection, Fill options (None/Solid/Gradient/Pattern), stroke + width default 3 pt + Align Edges requiring px, Shift/Alt/Shift+Alt draw modifiers, Create Custom Shape dialog on click, W/H + link, Shape layers.
- `https://www.photoshopessentials.com/basics/custom-shape-sets` — Preset Manager as Custom Shapes: select with click/Shift/Ctrl, **Save Set** to a `.CSH`, **Load** a set, **Reset Custom Shapes**, thumbnail size/list view, tooltips.
- `https://planetphotoshop.com/cs6-vector-tools.html` — CS6 vector layers, fill/stroke appearance, Snap to Pixel Grid/Align Edges, vector masks via Properties panel.
- `https://bjango.com/articles/photoshopcs6vectorshapes` — CS6 vector-layer presentation and pixel snapping.
- `https://doc.qt.io/qt-6/qpainterpath.html` — path geometry used for shape thumbnails and instantiation.
- `https://html.duckduckgo.com/html/?q=Photoshop+CS6+custom+shape+tool+shape+libraries+define+custom+shape+preset+manager` — discovery for the secondary sources above.

## Open questions

- **`.CSH` binary format.** Not public. *Resolves with:* the Adobe File Formats / preset documentation or a documented reader; until then `.CSH` support is "load our own format, import best-effort, do not assert byte parity".
- **Bundled shape sets and their exact contents/order in CS6.** *Resolves with:* a CS6 install's preset directory listing (names only; assets are Adobe-copyright and must not be redistributed — see `OVR-004`).
- **Whether clicking or double-clicking selects a shape** (help/tutorial wording varies by build). *Resolves with:* a CS6 UI capture.
- **Shape picker Replace/Append persistence** across sessions is not documented. *Resolves with:* preference/capture tests.
- **Duplicate-name and rename behavior** for `Define Custom Shape`. *Resolves with:* CS6 tests.
- **How CS6 migrates presets** (`Edit > Presets > Migrate`) into the shape library, and whether `.CSH` is versioned. *Resolves with:* the CS6 Help migration section (`10-workflow-io/presets-manager.md`).
- **Raster-selection → custom shape conversion path.** The Help requires a path; whether a one-step conversion exists is unverified. *Resolves with:* the CS6 Help paths section.
