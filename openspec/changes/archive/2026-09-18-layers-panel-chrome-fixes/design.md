## Context

The controls and filter stages landed the requested widgets but not the
requested arrangement. The header currently stacks filter / blend+opacity+fill+
menu / tree / locks / strip, the eye is drawn inside the tree's indented item
rect, and the filter toggle is a text button defaulting to off. The panel-group
corner menu already drives the same Layer commands through
`LayersPanel::performPanelMenuAction`, so the panel's own menu button is
redundant.

## Decisions

### D1 — Header rows
`QVBoxLayout` order: `filterBar_`, then a `QHBoxLayout` of `blend_` (stretch) and
a `QLabel("Opacity")` + `opacity_`, then a `QHBoxLayout` of the five lock buttons
(stretch) and a `QLabel("Fill")` + `fill_`, then `tree_` (stretch 1), then the
action strip. The labels are plain `QLabel`s owned by `body`.

### D2 — No panel menu button
Delete `panelMenu_` and the local panel `QMenu` from `LayersPanel`; the wired
commands stay reachable through the panel-group widget menu (`performPanelMenuAction`).
`panelMenuTextsForTest` is dropped and the former panel-menu assertion in the
M39 menu check is removed (the row and colour menus are still asserted).

### D3 — Left-anchored visibility toggle
`LayersTreeView : QTreeView` overrides `drawBranches` to draw nothing;
`tree_->setIndentation(0)` and `setRootIsDecorated(false)`. The delegate anchors
the eye at `rect.left() + 2` for every row and indents the thumbnail/name by
`depth * kIndent`, drawing a small chevron (`▸`/`▾`) at the indented position for
an expandable row. `eyeRect` stays the left-anchored eye; a new
`chevronRect(itemRect, depth)` is used by the panel's event filter to toggle
expansion on click (mirroring the eye hit-test). Opening still routes through
`tree_->setExpanded`, so `expandedPaths_` and the filter auto-expand are
unaffected.

### D4 — Filter lightswitch, on by default
`layers_filter_bar.cpp` gains `#include "icons.h"`; two original SVGs
`layers.filterOff.svg` and `layers.filterOn.svg`; `toggle_` shows
`layers.filterOn` and is checked by default, swapping to `layers.filterOff` when
unchecked. `filter_.enabled` follows the toggle. With no criteria active the
proxy still shows every row, so an on-by-default toggle is inert until a
criterion is chosen.

### D5 — Centred popup
`PercentField::showPopup` centres the popup horizontally under the field:
`popup_->move(mapToGlobal(QPoint((width() - popup_->width()) / 2, height())))`.

## Interface freeze

```
class pictura::LayersTreeView : QTreeView   // drawBranches no-op
LayerRowDelegate::chevronRect(QRect, int depth) const -> QRect
LayersPanel: panelMenu_ and panelMenuTextsForTest removed
LayerFilterBar: toggle_ icon layers.filterOn/Off, checked by default
```

## Risks / Trade-offs

- **Branch suppression** removes the stock expand affordance; the delegate
  chevron plus the event-filter click restores it. The chevron hit rect must stay
  in sync with the painted chevron (one helper used by both).
- **Removing the panel menu** changes one M39 self-test assertion; the commands
  themselves are unchanged and covered by the group-menu tests.
- **On-by-default filter** is inert without criteria; the `layers-filtering-search`
  spec is updated so the canonical contract matches.
