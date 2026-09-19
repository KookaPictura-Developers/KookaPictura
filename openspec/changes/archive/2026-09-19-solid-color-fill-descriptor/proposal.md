## Why

Roadmap P3 gap G8: `pictura-render`'s `decode_adjustment` only accepts the
in-house 4-byte `SoCo` payload, so a real Photoshop-authored solid-color fill
layer — a version-16 descriptor with a `Clr ` `RGBC` object — decodes to `None`
and renders as a no-op. `pictura-codec`'s descriptor DOM already parses that
block (`pictura_codec::read_descriptor`), and psd-tools registers `SoCo` as a
`DescriptorBlock`, so the bytes and their independent oracle both exist. The app
also authors the non-standard 4-byte form, so files we produce are not
Photoshop-readable.

## What Changes

- `decode_adjustment` in `crates/pictura-render/src/composite.rs` gains a
  `decode_solid_fill` helper and keeps a `SoCo` arm that returns
  `Adjustment::SolidFill([u8; 4])`.
  - The 4-byte `[r, g, b, a]` in-house arm is unchanged.
  - A payload that parses as a version-16 descriptor whose `Clr ` object carries
    `Rd  ` / `Grn ` / `Bl  ` `doub` values on the `0..=255` scale decodes to
    `Adjustment::SolidFill([r, g, b, 255])`, rounding and clamping each
    component to `0..=255` and forcing alpha 255.
  - A malformed payload (not a descriptor, missing `Clr ` or a component, a
    non-`doub` or non-finite component) returns `None`; it never panics.
- `is_fill_content_layer` treats a descriptor-form `SoCo` as solid fill content
  too, by routing through `decode_adjustment` as the single decoder instead of a
  hand-rolled 4-byte length check. `rasterize_fill_content` bakes the decoded
  RGBA.
- `pictura-render` gains
  `pub fn encode_solid_color_fill(color: [u8; 3]) -> AdjustmentData`,
  re-exported from the crate root, that builds the standard version-16 `SoCo`
  descriptor (`Clr ` / `RGBC` / `Rd  ` / `Grn ` / `Bl  `). The app's
  `add_solid_fill` switches to it, so authored files are Photoshop-standard.
- `scripts/generate-fixtures.py` gains a `solid_fill()` builder authoring a new
  `solid_fill.psd` (a `Base` pixel layer plus a `SoCo` descriptor fill layer,
  colour `(10, 20, 30)`), registered in `FIXTURES`; the existing fixtures are
  unchanged. A codec oracle assertion proves the block survives read and
  whole-`Document` round-trip, and a render test proves `decode_adjustment`
  yields `SolidFill([10, 20, 30, 255])`, matching psd-tools'
  `SolidColorFill.data`.
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `adjustment-layer-rendering`: the committed-decode requirement adds the
  `SoCo` payload (both forms) and the deferred requirement drops "a real `SoCo`
  descriptor"; a new requirement covers the `SoCo` encoder and another covers
  the app authoring the standard descriptor.
- `layer-management`: the `Rasterize subset` requirement broadens the
  fill-content predicate to the descriptor form as well as the 4-byte form.

## Impact

- `crates/pictura-render/src/composite.rs`: `decode_solid_fill`,
  `encode_solid_color_fill`, the `SoCo` match arm and doc comment, and unit
  tests.
- `crates/pictura-render/src/lib.rs`: the `encode_solid_color_fill` re-export.
- `crates/pictura-render/src/document_ops/layer_ops/rasterize.rs`:
  `is_fill_content_layer` and `rasterize_fill_content` decode through
  `decode_adjustment`.
- `crates/pictura-render/src/document_ops/layer_ops/create.rs`:
  `add_solid_fill` builds the descriptor via the encoder.
- `crates/pictura-render/src/tests/adjustment.rs` and `.../tests/rasterize.rs`:
  the descriptor decode, the descriptor-form fill-content case, and the updated
  alpha expectation.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/solid_fill.psd`, `.../tests/oracle.rs`,
  and `.../tests/fixtures/README.md`: the `solid_fill()` builder, the new golden
  fixture, and its oracle.
- No new dependency. No `pictura-adjust` change: `Adjustment::SolidFill`
  already exists.
- GPU path unchanged: `gpu/mod.rs::adjustment_params` has no `SolidFill` shader,
  so a document with a solid fill keeps falling back to the CPU composite.

## Out of scope (deferred)

- **Gradient (`GdFl`) and pattern (`PtFl`) fills.** Their descriptor schemas are
  only partly grounded and there is no committed render path; they stay no-ops.
- **The remaining adjustment keys** — `curv`, `mixr`, `clrL`, `selc`, and a
  version-3 `phfl` — stay no-ops.
- **Non-RGB colour.** The `Clr ` object is always `RGBC`; a non-RGB document
  colour mode is not converted.
- **Non-opaque solid fills.** The standard `SoCo` descriptor has no alpha, so an
  authored fill is always opaque.
