## Context

The frame's only panel is the M16 debug dock (`layersPanel`): a flat
`QListWidget` plus buttons. The document model already carries per-layer blend,
opacity, name, visibility, masks, and channels; the bridge exposes visibility and
a top-level layer list but not blend/opacity/name/thumbnails. The history stack
stores document+selection snapshots with no labels, so a History panel cannot
name the edits. There is no Navigator, Color, Info, or Histogram surface.

## Goals / Non-Goals

**Goals:**

- Real Layers and History panels backed by the document and history models.
- Navigator with thumbnail, proxy rectangle, and zoom controls.
- Color + Swatches panels with a shared foreground/background colour state that
  the M18 Eyedropper feeds.
- Info + Histogram panels driven by the composite image and cursor position.
- Bridge support for layer properties/thumbnails and labeled history.

**Non-Goals:**

- Group-tree expansion in the Layers panel (top-level rows only); drag-reorder;
  layer lock flags (would add a field to the frozen `Layer` struct); clipping,
  link, colour labels, and the filter/search row.
- Swatch library file I/O, Info colour samplers, and Histogram source/cache
  states.
- History branching beyond the existing bounded two-stack model.

## Decisions

### Panels are `QDockWidget` subclasses in `cpp/panels/`

Each panel owns its widget and is constructed with no document, then bound via
`setView(PictureView*)` / `setCanvas(ImageView*)` when the active document
changes; the frame calls `refresh()` on document/history changes. Frozen
interface:

```cpp
class LayersPanel   : public QDockWidget { public: explicit LayersPanel(QWidget* = nullptr);
  void setView(PictureView*); void refresh(); };
class HistoryPanel  : public QDockWidget { public: explicit HistoryPanel(QWidget* = nullptr);
  void setView(PictureView*); void refresh(); };
class NavigatorPanel: public QDockWidget { public: explicit NavigatorPanel(QWidget* = nullptr);
  void setCanvas(ImageView*); void refresh(); };
class ColorState    : public QObject { public: QColor foreground() const; QColor background() const;
  void setForeground(const QColor&); void setBackground(const QColor&);
  signals: void foregroundChanged(QColor); void backgroundChanged(QColor); };
class ColorPanel    : public QDockWidget { public: explicit ColorPanel(ColorState*, QWidget* = nullptr); };
class SwatchesPanel : public QDockWidget { public: explicit SwatchesPanel(ColorState*, QWidget* = nullptr); };
class InfoPanel     : public QDockWidget { public: explicit InfoPanel(QWidget* = nullptr);
  void setView(PictureView*); void setCursorPosition(const QPointF&); void refresh(); };
class HistogramPanel: public QDockWidget { public: explicit HistogramPanel(QWidget* = nullptr);
  void setView(PictureView*); void refresh(); };
```

- *Why:* panels depend on the active document, not the frame; a bind/refresh
  contract keeps them decoupled and lets the frame wire them once. The frame
  supplies stable `objectName`s and a `Window > Panels` toggle per panel.
- *Alternatives considered:* one monolithic panel class (rejected: unrelated
  concerns); a `QAbstractItemModel` per panel for Layers (used inside
  `LayersPanel`, not as a separate public type).

### Layers panel is a `QTreeView` + a small model, top-level rows only

Columns: visibility, thumbnail, name, blend mode, opacity. Selection highlights
a row; the panel's toolbar/context menu offers Add Adjustment, Delete Layer, and
Move Up/Down. Blend mode is a combo of the 27 PSD keys; opacity a 0–255 slider.
Group children are not expanded (a group row shows its kind).

- *Why:* covers the visible, editable state without the `Layer` struct churn of
  lock flags or a hierarchical model.
- *Deferral note:* group expansion, drag-reorder, locks, labels, filter/search.

### Bridge layer-property and thumbnail additions

- `layer_blend(i) -> QString` (PSD key), `set_layer_blend(i, key) -> bool`
- `layer_opacity(i) -> i32`, `set_layer_opacity(i, i32) -> bool`
- `set_layer_name(i, name) -> bool`
- `layer_thumbnail(i, size) -> QImage` — the layer's RGBA content scaled to
  `size`; null for group/adjustment layers
- `move_layer(i, delta) -> bool` — swap with a neighbour in the bottom-first list
The setters capture history and mark dirty; they reuse the existing Layer fields
(no `pictura-core` change).

### Labeled history and jump

`History` entries gain a `label: String` supplied at each capture site (e.g.
"Filter: Clouds", "Rotate 90° CW", "Apply Invert"). The bridge adds
`history_count()`, `history_index()`, `history_label(i)`, `history_jump(i)`,
`history_add_snapshot(label)`. The stored snapshots are unchanged (document +
selection); only labels and jump are new.

- *Why:* the M14 model already has ordered snapshots and a position; naming and
  jumping are additive and do not change undo/redo semantics.
- *Snapshots* are captured as extra entries marked as named restore points; the
  panel lists them above the linear states.

### Navigator and Histogram are view-side; no bridge work

Navigator reads `ImageView` zoom/offset and the composite `QImage`; Histogram
computes 256-bin per-channel counts in C++ from `view->image()`. Info reuses the
existing `sample_argb` and `selection_count` and a new cursor-position signal
from `ImageView::mouseMoved`.

- *Why:* the data is already in the composite image on the Qt side; a bridge
  round-trip would add marshalling for no benefit.

### Colour state lives on the frame

`ColorState` holds foreground/background; the frame owns one instance, the M18
Eyedropper sets the foreground, and Color/Swatches panels bind to it.

## Risks / Trade-offs

- **Core API churn** → No `pictura-core` change: lock flags are deferred rather
  than added to `Layer`.
- **History label drift** → Labels are set at the same call sites that capture
  snapshots, so a labelled state always has a snapshot.
- **Panel/frame coupling** → The bind/refresh interface keeps panels from
  reaching into the frame; the frame is the only wiring point.
- **Thumbnail cost** → Layers thumbnails render on refresh; the panel throttles
  to document changes and a small fixed size, and group/adjustment rows skip it.
- **`Layer` struct literals in tests** → Avoided by not adding fields.

## Migration Plan

Additive. The M16 debug dock is replaced by the Layers panel; the M16/M17/M18
self-test hooks remain. Rollback is reverting `cpp/panels/`, the bridge additions,
`history.rs`, `frame.{h,cpp}`, `commands.*`, and the OpenSpec artifacts.

## Open Questions

- Whether the Layers panel should offer a "Pass Through" entry for group rows;
  deferred with group expansion.
- Exact CS6 column order/width and thumbnail size; M20 uses a practical default.
- Whether History snapshots should render in a separate top group or inline;
  M20 lists snapshots first, then linear states.
