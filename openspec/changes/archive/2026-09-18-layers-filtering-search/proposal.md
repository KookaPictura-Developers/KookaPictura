## Why

CS6 added a filter/search row at the top of the Layers panel so a large
document can be narrowed to the layers that match a name, kind, effect, mode,
attribute, or color label. The panel has no such row today, so every layer is
always listed. This change adds the row as a pure view concern.

## What Changes

- **Filter row.** The panel gains a filter/search row above the blend/opacity
  header: a dimension popup (Name, Kind, Effect, Mode, Attribute, Color,
  defaulting to Kind), a criteria control that swaps with the dimension, and an
  on/off switch. The Effect dimension is present but disabled until layer
  effects exist (the styles stage); every other dimension works now on data the
  row projection already exposes.
- **View-only predicate.** Filtering is a `QSortFilterProxyModel` over the layer
  tree. It never mutates the document, adds no history state, and is not
  persisted. Name is a case-insensitive substring; Kind is a multi-select over
  the kinds the model has (pixel, adjustment, group, background); Mode is a
  blend-mode key; Color is a label; Attribute is one of Visible, Hidden,
  Locked, Has Mask, Clipped. Active criteria are combined with AND, and Kind
  values with OR.
- **Ancestor promotion.** A group shown only because a descendant matches is
  visible so the hierarchy stays navigable; non-matching siblings are hidden.
  Toggling the filter off restores the full tree.

## Capabilities

### New Capabilities

- `layers-filtering-search`: the Layers-panel filter/search row, its six
  dimensions, the view predicate, ancestor promotion, and the view-only /
  live-update contract.

### Modified Capabilities

<!-- none -->

## Impact

- New `crates/pictura-app/cpp/panels/layers_filter_proxy.{h,cpp}` and
  `layers_filter_bar.{h,cpp}`.
- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}` and
  `layers_panel_test.cpp` — model proxy wiring, selection/expansion through the
  proxy, test hooks.
- `crates/pictura-app/cpp/theme.cpp`, `CMakeLists.txt`, and a new
  `crates/pictura-app/cpp/selftest_layers_filter.cpp` self-test TU.
- No Rust, document, codec, or compositor change; no new dependencies
  (`QSortFilterProxyModel`/`QComboBox`/`QStackedWidget` only).
