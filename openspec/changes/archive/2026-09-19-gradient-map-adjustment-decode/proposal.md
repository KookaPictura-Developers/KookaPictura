## Why

Roadmap P3 gap G8: the codec's `ADJUSTMENT_KEYS` whitelist had the Gradient Map
key misspelled (two letters transposed), so a real `grdm` block was not
recognised as an adjustment: it landed in `extra_blocks` and `layer.adjustment`
stayed `None`, and `pictura-render`'s `decode_adjustment` was never reached. A
Gradient Map adjustment layer therefore composites as a no-op even though
`pictura-adjust` implements 15 destructive adjustments and
`docs/04-image-ops/adjustments/gradient-map.md` (`ADJ-015`) already defines the
mapping. psd-tools 1.19 reads and writes the `grdm` legacy struct, so the
missing pieces are the corrected codec key, a new op, the decode, an encoder so
the app can create a layer, and the panel wiring.

## What Changes

- `pictura-adjust` gains `Adjustment::GradientMap(GradientMapParams { stops,
  reverse })` and `GradientStop { location, color }`, plus a `gradient_map`
  kernel that maps Rec.601 luminance through the gradient (linear between
  stops), writing RGB and leaving alpha untouched. Validation: at least two
  stops, strictly increasing `location`, each `<= 4096`.
- `decode_adjustment` in `crates/pictura-render/src/composite.rs` gains a
  `decode_gradient_map` helper and a `grdm` arm producing
  `Adjustment::GradientMap(GradientMapParams)`.
  - Versions 1 and 3 (version 3 carries a 4-byte method after the two flag
    bytes).
  - Reads the reverse/dither flags, the unicode gradient name, and the colour
    stops (`u32` location, `u32` midpoint, `u16` mode, four `u16` colour
    components reduced to 8-bit RGB).
  - A truncated payload, an unsupported version, fewer than two stops, or
    non-increasing/out-of-range stop locations returns `None`; it never panics
    and never errors.
- `pictura-render` gains `pub fn encode_gradient_map(stops: &[GradientStop],
  reverse: bool) -> AdjustmentData`, re-exported from the crate root, that
  builds the version-1 `grdm` block `decode_adjustment` reads.
- The app can create one: `helpers.rs::adjustment_layer` gains a `"gradient-map"`
  arm and `panel_group_menu.cpp` gains an `adjustment:gradient-map` entry under
  the Adjustments panel menu.
- `scripts/generate-fixtures.py` gains a `gradient_map()` builder that authors a
  new `gradient_map.psd` (a `Base` pixel layer plus a black→white `grdm`
  adjustment layer) and registers it in `FIXTURES`; the existing
  `adjustment.psd` is left unchanged. A codec oracle assertion proves the `grdm`
  key and payload survive read and whole-`Document` round-trip.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `image-adjustments`: a new Gradient Map adjustment requirement; the
  alpha-preservation and parameter-validation requirements count and validate
  the new variant, and the ImageMagick oracle table gains a Gradient Map row.
- `adjustment-layer-rendering`: the committed-decode requirement adds `grdm`
  and the deferred requirement drops it; a new requirement covers the `grdm`
  encoder and another covers the app authoring path.

## Impact

- `crates/pictura-codec/src/common.rs`: accept `grdm` in `ADJUSTMENT_KEYS` so a
  real Gradient Map block is recognised as an adjustment.
- `crates/pictura-adjust/src/types.rs`: `GradientStop`, `GradientMapParams`, and
  the `Adjustment::GradientMap` variant.
- `crates/pictura-adjust/src/tonal.rs` (and `apply.rs` dispatch): the
  `gradient_map` kernel and its validation.
- `crates/pictura-adjust/src/lib.rs`: export the new types.
- `crates/pictura-adjust/src/tests.rs` and `tests/oracle.rs`: the new variant's
  alpha-preservation case, kernel tests, and one `MAPPING` row (`MAPPING.len()`
  16, `NO_EQUIVALENT` 13).
- `crates/pictura-render/src/composite.rs`: `decode_gradient_map`,
  `encode_gradient_map`, the `grdm` match arm, the crate-root re-export, and
  unit tests.
- `crates/pictura-render/src/lib.rs`: the `encode_gradient_map` re-export.
- `crates/pictura-render/src/tests/adjustment.rs`: drop `grdm` from the deferred
  set and add a Gradient Map composite test.
- `crates/pictura-app/src/cxxqt_object/helpers.rs`: a `"gradient-map"` arm in
  `adjustment_layer`.
- `crates/pictura-app/cpp/panels/panel_group_menu.cpp`: the
  `adjustment:gradient-map` menu entry.
- `crates/pictura-app/cpp/selftest*.cpp`: one new check (code 285) that a
  Gradient Map adjustment layer is added and changes the composite.
- `scripts/generate-fixtures.py` and
  `crates/pictura-codec/tests/fixtures/gradient_map.psd`: the `gradient_map()`
  builder and the new golden fixture.
- `crates/pictura-codec/tests/oracle.rs` and `fixtures/README.md`: the `grdm`
  key/payload assertion and the fixture documentation.
- No new dependency.
- GPU path: `gpu/mod.rs::adjustment_params` has no Gradient Map shader, so a
  document containing one continues to fall back to the CPU composite. Only the
  CPU path gains decoding.

## Out of scope (deferred)

- **The remaining adjustment keys** — `curv`, `mixr`, `clrL`, `selc`, a real
  Photoshop `SoCo` descriptor, and a version-3 `phfl`. Their models are
  ungrounded or have no committed op; they stay no-ops.
- **Gradient Map fidelity beyond plain linear interpolation** — midpoint bias,
  dither, transparency/opacity stops (the mapped output is opaque), and the
  gradient interpolation modes (classic/linear/perceptual/smooth). These are
  the internals `gradient-map.md` flags as closed or inferred, and parsing the
  fields only far enough to reach the stops is deliberate.
- **Non-RGB gradient colour models** — CMYK/Lab stops and the min/max colour
  fields are parsed past, not converted.
