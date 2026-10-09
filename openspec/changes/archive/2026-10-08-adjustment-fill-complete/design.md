# Design

## Context

Two independent code paths already create adjustment and fill layers without an
intervening dialog:

- `frame_menus_adjust.cpp::wireLayerAdjustments` walks a local
  `kLayerAdjustments[16]` table and calls `view->add_adjustment(kind)`.
- `layers_panel.cpp` builds the New Fill / Adjustment menu by hand, with two
  fill actions and a hard-coded five-entry adjustment list labelled with raw
  engine ids.

`helpers.rs::adjustment_layer(kind, mask)` already maps all sixteen kinds, so
the only gaps are the panel menu contents and the greyed `Layer Content
Options…` leaf. The Properties panel already renders the active adjustment by
reading `active_layer_path()` and `adjustment_page(view, path)`; "opening" it is
therefore a matter of making the panel visible and refreshing it.

## Decisions

### D1. Share the adjustment table through a header

`kLayerAdjustments` is moved into a new header,
`crates/pictura-app/cpp/layer_adjustments.h`, exposing:

```cpp
struct LayerAdjustment { const char* kind; const char* leaf; bool ellipsis; };
inline constexpr LayerAdjustment kLayerAdjustments[] = { /* 16 entries */ };
```

Both `frame_menus_adjust.cpp` and `layers_panel.cpp` include it, so the two
creation surfaces cannot drift. The header is registered in `CMakeLists.txt`
per the no-globbing rule. The panel menu labels each kind with its `leaf`
(plus `…` when `ellipsis`), matching the `Layer` menu. The two fill entries and
the Pattern… entry are panel-only and stay in `layers_panel.cpp`.

### D2. Group Fill and Adjustment entries as CS6 does

The menu is: Solid Color…, Gradient…, Pattern… (disabled), a separator, then the
sixteen adjustment kinds in the shared table's CS6 order. This matches the
existing shape (fills, separator, kinds) while completing the list.

### D3. Freeze the Layer Content Options command id

`Layer Content Options…` is currently a `leaf()` whose id is derived from its
path and stays unimplemented. It becomes a frozen
`command_ids::LayerContentOptions` entry added with `implemented = true`, then
given a handler and enabled provider in `frame_menus.cpp` next to the fill
handlers.

### D4. Open Properties via the existing show + refresh path

`PanelColumn::showPanel("propertiesPanel", true)` makes the Properties tab the
visible one; `PropertiesPanel::refresh()` recomputes from `active_layer_path()`,
which the panel already keeps in sync with the selected row. The command is
enabled by matching the current path against the projected rows and testing
`layer_row_has_adjustment` — true for both adjustment and fill layers — so a
pixel/type/group/Background current layer disables it.

### D5. Pattern authoring is deferred

The engine renders and rasterizes a `PtFl` pattern fill, but no encoder
(`encode_pattern_fill`) exists and, more importantly, there is no pattern
preset/library or picker to choose the `Ptrn` id a new fill would reference. Per
the task, no preset subsystem is built: the Pattern… entries stay disabled with
the `— not implemented yet` convention.

## Open Questions

- **Pattern-fill authoring.** What chooses the pattern id for a new `PtFl`
  layer — a Patterns-panel selection, a `.pat` preset library, or a default
  placeholder? Resolving this needs a `PatternFillParams` encoder and a source
  of named patterns in the document; deferred rather than invented here.
- **Whether a fill/adjustment layer's properties page is user-editable from
  Layer Content Options.** The Properties panel already edits adjustments; this
  change only routes to it, no editing behavior is added or changed.

## Risks

- Moving the table is a pure move; the `Layer > New Adjustment Layer` menu and
  the `tst_command_tree::layerAdjustmentMenu` contract must stay byte-identical
  in behavior.
- The panel menu previously labelled entries with engine ids
  (`invert`, `posterize`, …). The new labels are display names; the existing
  self-test checks only the `Gradient…` entry, which is unchanged.
