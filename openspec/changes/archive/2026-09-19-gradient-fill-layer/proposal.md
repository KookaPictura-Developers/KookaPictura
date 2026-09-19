## Why

Roadmap P3 ("gradient/pattern fill layers"): a real Photoshop gradient fill
layer is a version-16 descriptor in the `GdFl` additional-layer-info block, but
the codec does not even classify `GdFl` as an adjustment key, `decode_adjustment`
has no arm for it, and the app's `Layer > New Fill Layer > Gradient…` leaf is a
disabled placeholder. A Photoshop gradient fill therefore renders as a no-op.
The descriptor DOM already parses the block (`pictura_codec::read_descriptor`),
`pictura-adjust` already has `GradientStop`, psd-tools 1.19 registers `SoCo`/
`GdFl` as `DescriptorBlock`s and exposes the angle, kind, and stops, and its
`composite/paint.py` is a public reference for the five gradient geometries.

## What Changes

- `pictura-codec` accepts `GdFl` in `ADJUSTMENT_KEYS`, so a real gradient fill
  block is recognised as an adjustment and preserved in `layer.adjustment`
  instead of being dropped to `extra_blocks`.
- `pictura-adjust` gains `GradientKind { Linear, Radial, Angle, Reflected,
  Diamond }`, `GradientFillParams { stops, reverse, kind, angle_deg, scale }`
  (reusing the existing `GradientStop { location, color }`), and the
  `Adjustment::GradientFill(GradientFillParams)` variant. `apply` SHALL refuse it
  as `AdjustError::Unsupported`, because a fill is composited generatively, not
  applied to the backdrop.
- `decode_adjustment` in `crates/pictura-render/src/composite.rs` gains a
  `decode_gradient_fill` helper and a `GdFl` arm producing
  `Adjustment::GradientFill`:
  - `Angl` `doub` angle, the `Type` enum (typeID `GrdT`) kind, and the `Grad`
    object (class `Grdn`).
  - Only custom stops (`GrdF` = `CstS`); the `Clrs` list's `Clr ` `RGBC`
    `doub` components on `0..=255` and `Lctn` (`doub` or `long`) in `0..=4096`,
    kept in stored order.
  - Optional `Rvrs` bool and `Scl ` `doub` (defaults false / 100).
  - A colour-noise gradient (`ClNs`), a malformed or missing key, fewer than two
    stops, non-increasing or out-of-range locations, or a non-finite value
    returns `None`; it never panics.
- The renderer composites a gradient fill generatively over the layer rect for
  all five kinds following psd-tools' `draw_gradient_fill` geometry (linear,
  radial, angle, reflected, diamond), with the angle/scale/`reverse` handling,
  opaque output, and the layer's mask/opacity/fill/blend applied through the
  existing `blend_into` path.
- `pictura-render` gains `pub fn encode_gradient_fill(kind, stops, angle_deg) ->
  AdjustmentData`, re-exported from the crate root, building the standard
  version-16 `GdFl` descriptor. The app authors a black-to-white Linear fill.
- `layer-management`'s fill-content subset broadens from `SoCo`/`SolidFill` to
  any fill-content key whose payload decodes to `SolidFill` or `GradientFill`,
  through `decode_adjustment` as the single decoder. `rasterize_fill_content`
  bakes a decoded gradient by rendering it over the layer rect.
- The app: `Layer > New Fill Layer > Gradient…` becomes an enabled command and
  the Layers-panel fill menu gains a `Gradient…` entry, both creating a
  document-sized black-to-white Linear gradient fill layer.
- `scripts/generate-fixtures.py` gains a `gradient_fill()` builder authoring a
  new `gradient_fill.psd` (a `Base` pixel layer plus a channel-stripped
  document-sized layer whose tagged-block key is `GdFl` — psd-tools serializes
  the outer descriptor class as `null`, as it does for `solid_fill.psd` — a
  Linear black-to-white gradient, angle 0), registered in
  `FIXTURES`; existing fixtures are unchanged. A codec oracle proves the `GdFl`
  block survives read and whole-`Document` round-trip and a render test proves
  `decode_adjustment` yields the expected params.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `adjustment-layer-rendering`: the committed-decode requirement adds the `GdFl`
  payload; a new requirement covers generative gradient-fill rendering; another
  covers the `GdFl` encoder; another covers the app authoring/menu path.
- `image-adjustments`: a new `GradientFill` model requirement (with the
  `Unsupported` refusal); the alpha-preservation and parameter-validation
  requirements account for the variant, and the ImageMagick oracle table gains a
  `GradientFill` no-equivalent row.
- `layer-management`: the `Rasterize subset` requirement broadens the
  fill-content predicate to gradient fills and defines rasterizing them.

## Impact

- `crates/pictura-codec/src/common.rs`: `ADJUSTMENT_KEYS` gains `GdFl` (19→20),
  so the block is recognised as an adjustment and preserved (not just dropped to
  `extra_blocks`).
- `crates/pictura-adjust/src/types.rs`: `GradientKind`, `GradientFillParams`,
  and the `Adjustment::GradientFill` variant.
- `crates/pictura-adjust/src/apply.rs`: the `GradientFill` arm returns
  `AdjustError::Unsupported`.
- `crates/pictura-adjust/src/lib.rs`: export the new types.
- `crates/pictura-adjust/src/tests.rs` and `tests/oracle.rs`: the alpha case and
  one no-equivalent oracle row (`MAPPING.len()` 17, `NO_EQUIVALENT` 14).
- `crates/pictura-render/src/composite.rs`: `decode_gradient_fill`,
  `encode_gradient_fill`, the `GdFl` match arm, `composite_gradient_fill` plus
  the geometry/sampling helpers, the doc comment, and unit tests.
- `crates/pictura-render/src/lib.rs`: re-exports.
- `crates/pictura-render/src/document_ops/layer_ops/rasterize.rs`: the
  generalized fill-content predicate and gradient rasterization.
- `crates/pictura-render/src/document_ops/layer_ops/create.rs`:
  `add_gradient_fill`.
- `crates/pictura-render/src/tests/adjustment.rs` and `.../tests/rasterize.rs`:
  decode, generative composite, and fill-content cases.
- `crates/pictura-app/cpp/commands.h`, `cpp/command_tree.cpp`,
  `cpp/frame_menus.cpp`, `cpp/panels/layers_panel.cpp`: the enabled
  `Layer > New Fill Layer > Gradient…` entry and the Layers-panel fill action.
- `crates/pictura-app/src/cxxqt_object/impl_layers_rasterize.rs` (and the
  `cxxqt_object.rs` declaration): the `add_gradient_fill` bridge.
- `crates/pictura-app/cpp/selftest_layers_controls.cpp`: one new check (next free
  code **286**).
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/gradient_fill.psd`, `.../tests/oracle.rs`,
  and `.../tests/fixtures/README.md`: the `gradient_fill()` builder, the new
  golden fixture, and its oracle.
- No new dependency.
- GPU path unchanged: `gpu/mod.rs::adjustment_params` has no `GradientFill`
  shader, so a document containing one keeps falling back to the CPU composite.

## Out of scope (deferred)

- **Pattern fill (`PtFl`).** Its descriptor reference form is ungrounded and
  there is no committed render path.
- **Noise gradients (`ClNs`).** Only custom-stop gradients (`CstS`) decode; a
  noise gradient stays a no-op.
- **Transparency/opacity stops.** Gradient output is opaque; the `Tran`/stop
  opacity fields are parsed past, not modelled.
- **Stop midpoint (`Mdpn`) and non-linear interpolation (`Intr`).** Stops are
  sampled with plain linear interpolation; the midpoint bias and the
  classic/linear/perceptual/smooth modes are not modelled.
- **Non-RGB colour models.** Stops are always read as `RGBC`; a non-RGB
  document mode is not converted.
- **`curv`, `mixr`, `clrL`, `selc`, and a version-3 `phfl`.** Unchanged no-ops.
