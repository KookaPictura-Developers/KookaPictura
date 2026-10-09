# Proposal

## Why

CS6 marks a layer whose advanced blending has been customised with a small
**Blend If** badge on its Layers-panel row. The model already carries the
data (`Layer.blend_if` / `Layer.blending_ranges`), but the panel cannot show
the badge, so a user cannot tell at a glance which layers have customised
blending options.

## What Changes

- Add a row projection `hasBlendIf` (`HasBlendIfRole`) populated from a new
  layer-style bridge read that is true when the layer's Blend If view is
  non-default, or when a raw `blending_ranges` block exists without a typed
  view.
- Paint a compact, text-only `Blend If` chip on the row when the projection is
  set, in the delegate's right-edge badge run so the lock, fx, and mask
  geometry stays in sync. No new icon asset.
- Add a minimal bridge setter to give a layer a customised composite-source
  Blend If range (and restore the default), so the badge can be exercised.
- Cover the read helper with a Rust unit test and the badge with a Qt Test.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `ui/layers-panel`: the row-badge requirement gains the customised-Blend-If
  badge and its absent/default counterpart.

## Impact

- `crates/pictura-app/src/cxxqt_object/layer_style.rs`: bridge read + setter +
  unit test.
- `crates/pictura-app/cpp/panels/layers_panel_model.h`: `hasBlendIf` row field
  and `HasBlendIfRole`.
- `crates/pictura-app/cpp/panels/layers_panel.cpp`: populate the projection.
- `crates/pictura-app/cpp/panels/layers_panel_internal.h`: chip paint + layout.
- `crates/pictura-app/cpp/panels/layers_panel.h` + `layers_panel_test.cpp`:
  test accessors.
- `crates/pictura-app/cpp/tests/tst_layers_panel.cpp`: Qt Test.
- No new dependency, no new binary asset, no `docs/` change.
