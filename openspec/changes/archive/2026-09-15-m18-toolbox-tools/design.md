## Context

`ImageView` owns the canvas and currently hard-codes pointer behavior (left-drag
pans, wheel zooms). The frame owns the menu registry, documents, docks, and
status bar. `pictura-select` has a coverage-mask `Selection` with boolean ops,
feather/modify, Magic Wand, grow/similar, and color range, but no rectangle,
ellipse, or polygon rasterizers. `pictura-render::document_ops` can resize,
rotate, and flip a document but cannot crop to a region; there is no layer
translation and no pixel sampling.

The app self-test drives public frame/bridge methods, so each tool operation must
be representable as a call the self-test can make without synthesizing mouse
events.

## Goals / Non-Goals

**Goals:**

- A tool registry with a stable `ToolId`, an active tool, letter shortcuts, and
  a Tools dock.
- A context-sensitive options bar whose controls change with the active tool.
- Eight working tools: Move, Marquee, Lasso, Quick Selection, Crop, Eyedropper,
  Hand, Zoom.
- The engine operations the tools need: shape rasterizers, document crop, layer
  translation, sampling.
- Self-test coverage for each operation.

**Non-Goals:**

- Brush/paint, clone/heal, transform/warp, text/vector/shape tools.
- Quick Mask, Refine Edge, transform selection.
- Non-destructive crop regions and a crop shield.
- Marching ants for arbitrary masks; M18 draws a rubber band during the drag and
  the committed selection bounds.
- A foreground-colour UI (the Eyedropper records a colour on the frame only).

## Decisions

### One `ToolController` with a `switch`, not a `Tool` class per tool

`ToolId` is an enum; `ToolController` owns active-tool state, drag state, and the
options widgets, and implements each tool's press/move/release in one switch.
Tool metadata (id, label, letter shortcut, cursor, status hint) lives in a small
table.

- *Why:* eight thin behaviors do not justify eight classes plus a factory. A
  table plus a switch is the smallest thing that works and is trivially
  testable. *Revisit when* painting tools (which carry per-tool engines) arrive.
- *Alternatives considered:* a `Tool` interface with one implementation each
  (rejected: speculative abstraction), and routing tools through the command
  registry (rejected: tools are modal and stateful, not menu commands).

### `ImageView` gains pointer signals, a pan flag, and an overlay polygon

`ImageView` emits `pressed/moved/released` with image-space coordinates and
paints an optional overlay `QPolygonF`. `setPanEnabled(bool)` keeps the existing
left-drag pan for the Hand tool (and space-drag later); otherwise the controller
handles the drag.

- *Why:* the canvas already knows how to map widget↔image coordinates; giving it
  a generic overlay and event surface keeps tool logic out of the widget and
  avoids one widget per tool.
- *Overlay ceiling:* a single polygon; selections needing outline marching ants
  are deferred (`ponytail:` in the code).

### Shape rasterizers live in `pictura-select`

Add `Selection::rect`, `ellipse`, and `polygon` (scanline-fill with even-odd
winding) producing a coverage mask, plus `combine(sel, mode)`. Ellipse uses a
half-open pixel-center test; anti-aliasing is deferred.

- *Why:* the selection model already stores coverage and has combine/feather;
  adding rasterizers there reuses the mask type and PSD round-trip.
- *Quick Selection* is `magic_wand` at the cursor unioned into the selection for
  a brush radius; that is a documented approximation of Adobe's region-growing.

### Lasso uses a streaming bridge API

Rather than marshal a point list across cxx-qt, the bridge exposes
`begin_lasso(mode)`, `lasso_add_point(x, y)`, `end_lasso() -> bool`, with the
pending polygon held in `PictureViewRust`.

- *Why:* cxx-qt marshals Qt types cleanly but lists of points are clumsy; a
  streaming API keeps the bridge surface in already-supported types.

### Crop reuses canvas resizing

`pictura-render` gains `crop_document(doc, x, y, w, h)` implemented as: shift
every layer/selection/channel by `(-x, -y)` and resize the canvas to `w×h`,
reusing the M12 canvas machinery; it clamps to the document. The Crop tool
collects a rectangle and commits on Enter; `Image > Crop` uses the current
selection bounds when the tool has no pending rectangle.

- *Why:* the M12 document-canvas path already offsets layers/masks/channels; a
  crop is a canvas resize with a negative offset.
- *Non-destructive crop region* is deferred; M18 crop is destructive.

### Move translates the active pixel layer's rectangle

`translate_layer(dx, dy)` adds the delta to the topmost pixel layer's `rect`
(and masks), then recomposites. Group/adjustment layers are ignored; an empty
document is a no-op.

- *Why:* the compositor already respects `rect`, so a rect shift moves the layer
  content without copying pixels. *Revisit when* selections/transform land.

### Eyedropper records a foreground colour on the frame

`sample_argb(x, y)` returns the composited pixel as a packed `u32`; the frame
stores it as `foregroundColor()` and shows the hex in the status hint. No Color
panel yet.

### Options bar is a toolbar with a stacked page per tool

`QToolBar` (`Window > Options`) holding a `QStackedWidget`; each tool adds one
page (combine-mode buttons for selection tools, tolerance for Quick Selection,
crop/zoom controls). The Tools dock is a `QToolBar`-backed grid with a
`QActionGroup`. The frame registers both and exposes `activeTool()` /
`setActiveTool()` for the self-test.

## Risks / Trade-offs

- **Event routing regression** → Hand/Zoom/pan behavior is re-expressed through
  the controller; the existing zoom/pan self-test still calls the `ImageView`
  methods directly, so it stays valid.
- **Modal tool state vs. the dock controls** → The M16 debug dock remains; tools
  are additive.
- **Quick Selection fidelity** → Documented as a wand-union approximation, not
  Adobe's region growing; property-checked, not delta-fit.
- **Crop on a document with masks/channels** → Reuse the M12 canvas offset logic
  so masks and extra channels move with the layers; self-test asserts a pixel
  remap.
- **Overlay only handles a polygon** → Enough for marquee/lasso/crop rubber
  bands; a per-pixel outline is a later milestone.

## Migration Plan

Additive. New C++ units (`tools`, `toolbox`, `options_bar`), new bridge methods,
new engine functions. The registry gains tool commands; the frame gains a Tools
dock and options bar. Rollback is reverting the `pictura-app`,
`pictura-select`, `pictura-render` sources and the OpenSpec artifacts.

## Open Questions

- Exact CS6 tool-option defaults per tool (e.g. marquee feather 0, anti-alias on);
  M18 uses the documented common defaults and marks the rest for a capture pass.
- Whether the Move tool should move a selection's pixels or the whole layer when
  a selection exists; M18 moves the layer and notes the divergence.
- Whether crop should offer "Delete Cropped Pixels" as a toggle now; M18 always
  deletes (destructive) and defers the non-destructive option.
