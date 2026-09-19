## Why

Roadmap P3 gap G8: the codec preserves the `phfl` (Photo Filter) block, but
`pictura-render`'s `decode_adjustment` returns `None` for it, so a Photo Filter
adjustment layer composites as a no-op even though `pictura-adjust` already
implements the operation. The payload is a fixed struct that psd-tools 1.19
reads and writes, so the missing pieces are the decode, an encoder so the app
can create a layer, and the panel wiring.

## What Changes

- `decode_adjustment` in `crates/pictura-render/src/composite.rs` gains a
  `decode_photo_filter` helper and a `phfl` arm producing
  `Adjustment::PhotoFilter(PhotoFilterParams { color, density,
  preserve_luminosity })`.
  - Version 2 only: `u16` version, `u16` colour space, four `u16` colour
    components, `u32` density, `u8` luminosity.
  - Version 3 (`3I` CIE XYZ) returns `None` (deferred).
  - A truncated payload, a version other than 2, a colour component above 255,
    or a density above 100 returns `None`; it never panics and never errors.
- `pictura-render` gains `pub fn encode_photo_filter(color, density,
  preserve_luminosity) -> AdjustmentData`, re-exported from the crate root, that
  builds the version-2 `phfl` block the decoder reads and clamps density to
  `0..=100`.
- The app can create one: `helpers.rs::adjustment_layer` gains a `"photo-filter"`
  arm and `panel_group_menu.cpp` gains an `adjustment:photo-filter` entry under
  the Adjustments panel menu.
- `scripts/generate-fixtures.py` gains an `adjustment()` builder that reproduces
  the documented existing 5 adjustment layers plus a `PhotoFilter` layer, and
  `crates/pictura-codec/tests/fixtures/adjustment.psd` is regenerated. The
  committed golden fixture changes and the codec oracle test is updated to match;
  the regeneration must leave the existing layers' bytes unchanged apart from
  appended-layer offsets.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `adjustment-layer-rendering`: the committed-decode requirement adds `phfl`
  (Photo Filter) and the deferred requirement drops it; a new requirement covers
  the `phfl` encoder and another covers the app authoring path.

## Impact

- `crates/pictura-render/src/composite.rs`: `decode_photo_filter`,
  `encode_photo_filter`, the `phfl` match arm, the crate-root re-export, and
  unit tests.
- `crates/pictura-render/src/tests/adjustment.rs`: drop `phfl` from the deferred
  set and add a Photo Filter composite test.
- `crates/pictura-app/src/cxxqt_object/helpers.rs`: a `"photo-filter"` arm in
  `adjustment_layer`.
- `crates/pictura-app/cpp/panels/panel_group_menu.cpp`: the
  `adjustment:photo-filter` menu entry.
- `crates/pictura-app/cpp/selftest*.cpp`: one new check (code 283) that a
  Photo Filter adjustment layer is added and changes the composite.
- `scripts/generate-fixtures.py` and
  `crates/pictura-codec/tests/fixtures/adjustment.psd`: the `adjustment()`
  builder and the regenerated golden fixture.
- `crates/pictura-codec/tests/oracle.rs`: the adjustment-layer name/payload
  assertions gain `PhotoFilter`.
- No new dependency: `pictura-adjust`'s `PhotoFilter` op already exists. No
  `pictura-adjust` change.
- GPU path: `gpu/mod.rs::adjustment_params` has no Photo Filter shader, so a
  document containing one continues to fall back to the CPU composite, exactly
  as it does today. Only the CPU path gains decoding.

## Out of scope (deferred)

- **Curves (`curv`).** The implemented `Adjustment::Curves` is a single
  composite curve, while Photoshop stores per-channel curves, and the legacy
  `curv` channel-bitmap order is ungrounded: no real Photoshop fixture in this
  repository contains any adjustment key. Decoding it now would either bind the
  wrong curve or invent a mapping. This blocker is recorded here so the docs can
  capture it later; it is deferred until a real Photoshop `curv` baseline exists.
- **Channel Mixer (`mixr`).** The Adobe spec's byte count and field description
  disagree and psd-tools treats the remainder as opaque; the 12-value
  `ChannelMixerParams` layout is not groundable.
- **`phfl` version 3 (XYZ).** Needs a CIE XYZ to sRGB colour transform that is
  not confidently groundable here; version 2 is the committed form.
