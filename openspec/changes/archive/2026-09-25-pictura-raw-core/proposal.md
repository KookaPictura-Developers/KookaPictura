# Proposal: pictura-raw-core

## Why

The P2.5 smart-object work models the Camera Raw Filter and can edit one `Fltr`
key in place, but there is no way to apply the filter's 11 PV2012 Basic controls
to pixels or to author the filter onto a converted smart object. This change
ships Pictura Raw as the engine + codec core so a later batch can
wire the UI. On disk it stays Photoshop's standard camera-raw smart filter
(`filterID 2683`, name `"Camera Raw Filter"`) inside `SoLd.filterFX`.

## What Changes

- Add `pictura_core::PicturaRawSettings`, a thin alias of `CrsSettings` (the 11 optional
  PV2012 Basic controls); no new struct.
- Add `pictura-codec/src/pictura_raw.rs`: `decode_pictura_raw_settings` (tolerant),
  `encode_pictura_raw_fltr` (the grounded `Fltr` key map/types), and `attach_pictura_raw_filter`
  (update an existing filter; insert one into a preserved `SoLd`/`SoLE`; or
  record it for the writer to author). The writer now emits `filterFX` when a
  converted embedded object carries smart filters.
- Add `pictura-adjust/src/pictura_raw.rs`: `render_pictura_raw`, an `f32` pipeline (white
  balance, exposure, contrast, highlights/shadows/whites/blacks, clarity,
  vibrance, saturation) that is a byte-identical no-op for default settings.
- Add `pictura-render/src/document_ops/pictura_raw.rs`: `apply_pictura_raw`, which bakes the
  filtered source into the layer proxy and attaches the settings.
- No UI, menu, dialog, or C++ change in this batch.

## Capabilities

### New Capabilities
- `pictura-raw`: the Pictura Raw engine core — settings alias, `Fltr` codec,
  authoring, CPU render pipeline, and the bake-into-proxy document op.

### Modified Capabilities
- (none)

## Impact

- New files: `crates/pictura-codec/src/pictura_raw.rs`,
  `crates/pictura-adjust/src/pictura_raw.rs`,
  `crates/pictura-render/src/document_ops/pictura_raw.rs`, plus OpenSpec artifacts.
- Modified: `pictura-core` `crs.rs`/`lib.rs`, `pictura-codec` `lib.rs` and
  `smart_writer.rs`, `pictura-adjust` `lib.rs`, `pictura-render`
  `document_ops/mod.rs` and `lib.rs`.
- No new dependency. Document-level `FXid`/`FEid`/`FMsk` render caches are not
  authored (the baked proxy carries the pixels).
