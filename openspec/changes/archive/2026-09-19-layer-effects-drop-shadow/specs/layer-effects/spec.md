## ADDED Requirements

### Requirement: The object-based effects descriptor decodes a drop shadow

`pictura-core`'s `Layer` SHALL expose its preserved additional-layer-information
blocks by key, and `pictura-render` SHALL derive a typed `DropShadow { enabled,
present, blend_mode, color, opacity, angle_deg, distance, spread, size,
use_global_angle, knocks_out }` from a layer's `lfx2`
(`OBJECT_BASED_EFFECTS_LAYER_INFO`) tagged block. The block SHALL be read as a
`DescriptorBlock2` (a `u32` version and a `u32` data version, then a version-16
descriptor body) and the top-level `DrSh` object SHALL be decoded. The decoder
SHALL read `enab` as `enabled`, `present` as `present`, `Md  ` (typeID `BlnM`,
a normal Photoshop blend-mode key) as `blend_mode`, `Clr ` (an `RGBC` object
whose `Rd `/`Grn `/`Bl  ` values are on the `0..=255` scale) as `color`,
`Opct` as `opacity` in percent, `uglg` as `use_global_angle`, `lagl` as
`angle_deg` in degrees, `Dstn` as `distance` in pixels, `Ckmt` as `spread` in
percent, `blur` as `size` in pixels, and `layerConceals` as `knocks_out`.
Numeric values SHALL be accepted as either a unit float or a `doub`. A missing
`Md  ` SHALL decode to `Normal`, a missing colour SHALL decode to black, and
absent numeric or boolean keys SHALL take the Photoshop defaults (opacity 75,
angle 120, distance 5, spread 0, size 5, use-global-angle true, knocks-out true).
A finite value outside its documented range SHALL be clamped (`opacity` and
`spread` `0..=100`, `distance` `0..=30000`, `size` `0..=250`). A missing `lfx2`
block, a missing `DrSh`, an unknown data version, a wrong-typed or non-finite
numeric value (including a finite `f64` that overflows to infinity as an `f32`),
a wrong `Md  ` typeID, a non-`RGBC` `Clr `, or a descriptor that fails to parse
SHALL be a no-op (`None`) and SHALL NOT panic. A `DrSh` whose `present` or `enab`
is false SHALL decode but SHALL render nothing.

#### Scenario: A drop shadow descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains a `DrSh` with `enab` true, `present` true, `Md  ` value `mul `, a non-black `Clr `, `Opct` 60, `uglg` false, `lagl` 45, `Dstn` 8, `Ckmt` 20, `blur` 12, and `layerConceals` false
- **THEN** `decode_drop_shadow` returns a `DropShadow` whose `enabled` and `present` are true, `blend_mode` is `Multiply`, `color` is the stored colour, and `opacity`, `angle_deg`, `distance`, `spread`, `size`, `use_global_angle`, and `knocks_out` are the stored values

#### Scenario: Missing drop shadow keys take Photoshop defaults

- **WHEN** a `DrSh` object carries only `enab` true and `present` true
- **THEN** the decoded `DropShadow` has blend mode `Normal`, black colour, opacity 75, angle 120, distance 5, spread 0, size 5, use-global-angle true, and knocks-out true

#### Scenario: Out-of-range numeric values are clamped or rejected

- **WHEN** a `DrSh` carries `blur` 1e30 and `Dstn` 1e30
- **THEN** `decode_drop_shadow` returns a `DropShadow` with `size` 250 and `distance` 30000, and rendering it does not panic
- **AND WHEN** a `DrSh` carries `blur` 1e300 (a finite `f64` that overflows `f32`)
- **THEN** `decode_drop_shadow` returns `None` and rendering does not panic

#### Scenario: A disabled drop shadow decodes but is inert

- **WHEN** a `DrSh` object has `present` true and `enab` false
- **THEN** `decode_drop_shadow` returns a `DropShadow` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Malformed or absent effects are a no-op

- **WHEN** a layer has no `lfx2` block, or its `lfx2` block lacks a `DrSh`, has an unknown data version, or fails to parse
- **THEN** `decode_drop_shadow` returns `None` and does not panic

### Requirement: A drop shadow composites behind the layer content

For an enabled `DropShadow` on a visible non-group layer with `present` true, the CPU compositor SHALL composite the shadow into the running canvas immediately before the layer's own content, so the shadow lies over the layers below and behind the layer content. It SHALL build a coverage matte from the
layer's content alpha — a pixel layer's alpha channel, a solid fill's payload
alpha, an opaque gradient or embedded smart-object coverage, or a pattern fill's
resolved pattern alpha — multiplied by the layer mask. It SHALL offset the matte
by `distance` pixels along the effect angle using `dx = -distance·cos(angle)` and
`dy = +distance·sin(angle)` in screen coordinates (y down), dilate it by a
spread radius derived from `spread` (a max filter), Gaussian-blur it by a radius
of `size` pixels using the crate's existing blur, and multiply it by
`opacity/100`. It SHALL composite
the result with the effect's `blend_mode` and `opacity`, without applying the
layer's own opacity or fill a second time. A zero-area, fully off-canvas, or
empty matte, `size` 0 with no offset, or `opacity` 0 SHALL be a no-op. A layer
with no effect, or with a disabled or not-present effect, SHALL composite
byte-identically to the same document without the effect. The shadow SHALL be
clipped to the canvas, SHALL NOT panic, and SHALL NOT add a dependency.

#### Scenario: The shadow is offset along the angle

- **WHEN** an opaque square layer over an opaque backdrop has a drop shadow with `distance` 4, `blur` 0, `spread` 0, opacity 100, and `angle_deg` 0
- **THEN** the backdrop pixel four pixels left of the square is darkened and the pixel four pixels right of the square is unchanged

#### Scenario: Distance scales the shadow offset

- **WHEN** the same layer is rendered with `distance` 2 and with `distance` 6 at `angle_deg` 0
- **THEN** the darkened region for `distance` 6 extends farther from the content than for `distance` 2

#### Scenario: Size softens the shadow

- **WHEN** the same layer is rendered with `size` 0 and with `size` 6
- **THEN** the blurred shadow transitions across more pixels than the hard shadow and both darken the backdrop near the content

#### Scenario: Opacity and colour shape the shadow

- **WHEN** a shadow with `opacity` 50 and a red `color` is composited over a white backdrop
- **THEN** the shadowed pixels are a partially transparent red tint rather than black, and lowering `opacity` toward 0 approaches the backdrop

#### Scenario: Spread dilates the shadow

- **WHEN** the same layer is rendered with `spread` 0 and with a positive `spread`
- **THEN** the positive-spread shadow covers no fewer pixels than the unspread shadow, and `spread` 100 with a non-zero `size` produces a harder edge than `spread` 0

#### Scenario: The layer mask shapes the matte

- **WHEN** a shadowed layer's mask hides half the canvas
- **THEN** the shadow is absent where the mask is zero and present where the mask is 255

#### Scenario: The shadow composites behind the content

- **WHEN** a shadow is offset so that it overlaps the layer's own opaque content
- **THEN** the overlapping pixel carries the content colour and not the shadow colour

#### Scenario: A disabled or absent effect is a no-op

- **WHEN** a layer carries no `lfx2` block, or a `DrSh` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect
