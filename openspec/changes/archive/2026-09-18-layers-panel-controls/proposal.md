## Why

The Layers-panel header controls do not match CS6's input model: Opacity and
Fill are raw 0–255 spin boxes with no slider or label scrubbing, and the lock
strip names only three lock bits when CS6 also exposes a structural nesting
lock. This change makes the controls behave like the requested UI and closes the
nesting gap end-to-end (model, PSD, bridge, panel).

## What Changes

- **Opacity and Fill as percentages.** Both controls SHALL present 0–100 %, with
  a text field, a popup slider, and drag-on-label scrubbing. The view converts
  to the stored 0–255 value with `round(pct * 255 / 100)`; the bridge keeps
  reporting and editing 0–255, so no stored-format change.
- **Five-icon lock strip.** The strip becomes alpha, paint, position, nesting,
  full (all), as icon toggles. **BREAKING** (PSD/semantics): a new `nesting`
  lock bit (`0x08`) joins `LockFlags`; `LockFlags::all()` becomes `0x0F`, and the
  PSD `lspf` block reads/writes the fourth bit. A nesting-locked layer keeps its
  structural parent: Group Layers and Ungroup Layers are refused when the
  selection contains one, while within-container Move Up/Down stays allowed.
- **Clipping-mask row indicator.** A clipped row additionally draws a
  clipping-mask curve indicator, on top of the existing indentation and base
  underline.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `layers-panel`: percent Opacity/Fill controls; the fifth `nesting` lock flag
  in the lock strip and the row projection; the structural refusal for a
  nesting-locked layer; the clipping-mask row indicator.

## Impact

- `crates/pictura-core/src/lib.rs` — `LockFlags::NESTING`, `all()`/`is_all()`.
- `crates/pictura-codec/src/read.rs` / `write.rs` — `lspf` fourth bit.
- `crates/pictura-render/src/document_ops/layer_ops/properties.rs` — nesting
  refusal in the grouping ops.
- `crates/pictura-app/src/cxxqt_object/impl_layers.rs` / `helpers.rs` — the
  `nesting` flag name and the widened full-lock mask.
- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}`,
  `layers_panel_internal.h`, new `percent_field.{h,cpp}` — the controls and the
  clip indicator.
- `crates/pictura-app/cpp/theme.cpp`, `CMakeLists.txt`, `assets/icons/`,
  `assets/pictura.qrc` — styles, build entry, five lock SVGs and a clip SVG.
- No new dependencies; Qt6/QSlider/QToolButton only.
