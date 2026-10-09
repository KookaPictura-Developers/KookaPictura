# Design

## Context

- The layer-style engine and dialog (#109) already expose, over `PictureView*`:
  `layer_style_can_edit`, `layer_style_has`, `layer_style_copy`,
  `layer_style_can_paste`, `layer_style_paste`, `layer_style_clear`,
  `layer_style_any_visible`, `layer_style_set_all_visible`, and
  `layer_style_value` (`crates/pictura-app/src/cxxqt_object/layer_style.rs`).
- `pictura_render::layer_style_effect_names()` returns the ten effect keys in
  CS6 list order; it is not yet surfaced by the app bridge.
- The Layers panel row delegate paints an `fx` badge for adjustment content
  only; its right-edge badge geometry is duplicated between `paint`,
  `badgesRight`, and `nameRect`.
- `LayerFilter` (`layers_filter_proxy.h`) carries name / kinds / mode / color /
  attribute; the Effect dimension exists in the filter bar as a disabled combo.

## Approach

### Dialog launch

`LayersPanel::openLayerStyle(path)` becomes: gate on `view_` and
`layer_style_can_edit(*view_, path)`, then construct
`LayerStyleDialog(view_, path, QString(), this)` (empty effect key = Blending
Options) and run it through the existing `runDialog(dialog, this)` helper from
`dialogs.h`, mirroring `frame_menus_layer_style.cpp`. On return, `refresh()`.
The double-click path already calls it for a non-background row outside the
name/eye/mask controls; the `fx` strip button calls it with `currentPath()`.

### Row menu

The row menu stays data-driven. `RowSpec` gains a trailing `Enable` enum
(`Always`, `StyleEdit`, `HasStyle`, `CanPasteStyle`) with a default of
`Always`, so only the four style rows add an initializer.
`populateRowMenu` evaluates the policy against `view_` plus the row `path`:

- `blendingOptions` — appears for pixel, background, group, and type rows;
  enabled only where `layer_style_can_edit` is true (pixel/type), disabled
  otherwise (group/background). The kind mask stays the existing
  `Pixel | Background | Group` plus `Type`.
- `copyLayerStyle` — enabled when `layer_style_has(path)`.
- `pasteLayerStyle` — enabled when `layer_style_can_paste()`.
- `clearLayerStyle` — enabled when `layer_style_has(path)`.

Adjusted rows still show the style command block; `performRowAction` dispatches
`blendingOptions` to `openLayerStyle(path)` and copy/paste/clear to the bridge
on the row path, recording through the bridge's own history.

### fx badge

The model projects two new roles appended to `LayerRole` (so existing role
numbers do not shift): `HasLayerStyleRole` (bool) and `StyleEffectsRole`
(QStringList of present effect keys). `LayerRow` gains `hasStyle` and
`styleEffects`; `refresh()` fills them — `hasStyle` from the new
`layer_row_has_style(i)` bridge read, and, for styled rows only,
`styleEffects` from `layer_style_value(path, "<effect>.exists")` over the
bridge's effect-name list.

The delegate gets a single `showsFx(index)` predicate
(`(HasAdjustmentRole || HasLayerStyleRole) && !LayerRowShapeRole`) used by
`paint`, `badgesRight`, and `nameRect`, plus an `fxRect(itemRect, index)`
hit-target mirroring the paint walk. Sharing the predicate keeps the badge,
the mask/link geometry, and the name elision in agreement.

### FX toggle

`Alt`-clicking a row's `fxRect` calls
`layer_style_set_all_visible(*view_, !layer_style_any_visible(*view_, true))`.
The `fx` strip button is enabled: a plain click opens Blending Options for the
active layer, an `Alt`-click performs the same all-effects toggle. Both are
one undo state each via the bridge.

### Effect filter dimension

`LayerFilter` gains `QString effect`. The filter bar populates the Effect combo
from `layer_style_effect_names()` (humanised label, key as item data) via a new
bridge `layer_style_effect_names()` wrapper, and stops disabling the filter
when the dimension is active. The proxy matches `StyleEffectsRole` against
`filter.effect`, and `hasActiveCriteria` counts it.

## Deferred

- **Blend-If badge** — the delegate's badge vocabulary has no marker for an
  advanced-blending (Blend If) override, and the engine models no such flag, so
  no badge is drawn for it. Add when a `blendIf` predicate lands on the layer.
- **Shape / smart-object style rows** — the row menu's style block keeps its
  `Pixel` (and, for Blending Options, `Type`) kind mask; a shape row keeps its
  shape-specific block. Extending the style rows to shape/smart-object kinds is
  out of scope here.
