# Proposal

## Why

The Layer Style engine, dialog, and menu-bar wiring landed (#109), but the
Layers panel still stubs every style affordance: `LayersPanel::openLayerStyle`
is an explicit no-op, the row menu marks Blending Options / Copy / Paste /
Clear Layer Style as not implemented, no row paints an `fx` badge, and the
filter's Effect dimension is disabled. The panel is where CS6 users reach layer
styles, so the feature is not usable from the surface it belongs to.

## What Changes

- Open the `LayerStyleDialog` on Blending Options (`effectKey == ""`) from a
  row double-click outside the name and from the panel's `fx` strip button,
  gated on the layer's ability to carry a style.
- Flip the row-menu rows `blendingOptions`, `copyLayerStyle`,
  `pasteLayerStyle`, and `clearLayerStyle` to implemented, dispatched through
  the existing layer-style bridge, with per-row enablement for Copy (has a
  style), Paste (style clipboard non-empty), and Clear (has a style).
- Paint the `layers.fx` badge on any row whose layer carries a layer style,
  sharing the badge geometry with the existing adjustment badge.
- Make the panel `fx` strip button active and let an `Alt`-click on a row's fx
  region or the strip button toggle all effects through
  `layer_style_set_all_visible`.
- Enable the filter's `Effect` dimension, populated from
  `layer_style_effect_names()`, matching rows whose layer carries that effect.
- Defer the Blend-If badge: no advanced-blending flag is modelled.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui/layers-panel` — the action strip's fx button and the row fx badge become
  active, and the panel gains the layer-style row-menu commands, the FX toggle,
  and the Effect filter dimension.

## Impact

- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}` — dialog launch, fx
  strip button, fx Alt-click, style effect projection into the row.
- `crates/pictura-app/cpp/panels/layers_panel_menu.cpp` — implemented style
  rows and dispatch.
- `crates/pictura-app/cpp/panels/layers_panel_internal.h` — fx badge paint and
  hit-target geometry.
- `crates/pictura-app/cpp/panels/layers_panel_model.h` — `HasLayerStyleRole`
  and `StyleEffectsRole` row projections.
- `crates/pictura-app/cpp/panels/layers_filter_bar.{h,cpp}` and
  `layers_filter_proxy.{h,cpp}` — the Effect filter dimension.
- `crates/pictura-app/src/cxxqt_object/layer_style.rs` — per-row style
  projection and the effect-name list in the app bridge.
- Qt Test in `tst_layers_panel.cpp` (and a refreshed `tst_command_tree.cpp`
  panel-menu expectation).
- No dependencies, no `docs/` changes, no new self-test exit codes.
