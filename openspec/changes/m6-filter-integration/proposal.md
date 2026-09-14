## Why

M6 built `pictura-filters` as pure destructive math over a planar `PixelBuffer`,
but a filter cannot yet reach a layer: there is no render-level entry point that
owns a layer's rect/channel layout and no app command that picks a layer and
confines the apply to the active selection. This change wires the finished M6
filters into the render crate and the app so they become usable, reusing the M5-C
selection→`LayerMask` path.

## What Changes

- `pictura-render`: add `apply_filter(layer, filter, mask) -> Result<(),
  FilterError>` — destructive application of a `pictura_filters::Filter` to a
  pixel layer's color channels, confined by an optional document-coordinate
  `LayerMask` coverage (`None` = full frame), with channel `-1` (transparency)
  never modified and `rect` / opacity / blend unchanged. Bad layer data returns
  `FilterError::InvalidParams` instead of panicking; filter-level validation
  errors propagate.
- `pictura-render` gains a `pictura-filters` dependency, the same way it already
  depends on `pictura-adjust`.
- `pictura-app`: add `PictureView::apply_filter(kind)` — applies a filter to the
  topmost pixel layer (the last bottom-first `doc.layers` entry with
  `adjustment.is_none() && !is_group`) using the active selection as the mask,
  recomposites, and emits `changed`; a filter combo + "Apply Filter" button in
  the dock; and a headless `--self-test` that proves the change is confined to
  the selection (pixels outside unchanged).
- Out of scope (later): non-destructive Smart Filters, filter dialogs/preview,
  per-layer filter targeting, undo, filters on adjustment/group layers, and
  16/32-bit pixel math.

## Capabilities

### New Capabilities

- `filter-application`: the render-level `apply_filter` contract — destructive
  application to a pixel layer's color channels `0,1,2`, planar 3-channel
  extraction and validation, document-coordinate mask confinement (including
  `default_color` outside the mask rect and full-frame when `None`), alpha and
  layer-metadata preservation, and typed errors instead of panics.
- `filter-app-ui`: the app-side filter command and controls — `PictureView::
  apply_filter(kind)`, the topmost-pixel-layer selection rule, the filter-kind →
  `Filter` defaults, the active-selection mask, recomposite + `changed`, the dock
  combo/button, and the headless selection-confinement self-test.

### Modified Capabilities

None. No existing capability's requirements change.

## Impact

- Code: `crates/pictura-render/src/lib.rs` and `crates/pictura-render/Cargo.toml`
  (plus render unit tests); `crates/pictura-app/src/cxxqt_object.rs` and
  `crates/pictura-app/cpp/main.cpp`.
- Dependencies: `pictura-render` → `pictura-filters` (new, in-workspace). No new
  external crates.
- No changes to `docs/`; the M6C brief and the filter/selection specs are the
  contract and are not modified.
- Follows `docs/dev/m6c-filter-integration.md`,
  `docs/06-filters/filters-overview.md`, and `docs/08-selection/selection-model.md`.
