## ADDED Requirements

### Requirement: The object-based effects descriptor decodes an outer glow

`pictura-render` SHALL derive a typed `OuterGlow { enabled, present, blend_mode,
color, opacity, spread, size, technique }` from a layer's `lfx2`
(`OBJECT_BASED_EFFECTS_LAYER_INFO`) tagged block. The block SHALL be read as a
`DescriptorBlock2` (a `u32` version and a `u32` data version, then a version-16
descriptor body) and the top-level `OrGl` object SHALL be decoded. The decoder
SHALL read `enab` as `enabled`, `present` as `present`, `Md  ` (typeID `BlnM`,
a normal Photoshop blend-mode key) as `blend_mode`, `Clr ` (an `RGBC` object
whose `Rd `/`Grn `/`Bl  ` values are on the `0..=255` scale) as `color`,
`Opct` as `opacity` in percent, `Ckmt` as `spread` in percent, `blur` as `size`
in pixels, and `GlwT` (typeID `BETE`, values `SfBL` Softer or `PrBL` Precise) as
`technique`. Numeric values SHALL be accepted as either a unit float or a
`doub`. A missing `Md  ` SHALL decode to `Screen`, a missing colour SHALL decode
to the Photoshop default `#FFFFBE`, a missing `GlwT` or an unknown technique
value SHALL decode to Softer, and absent numeric keys SHALL take the Photoshop
defaults (opacity 75, spread 0, size 5). A finite value outside its documented
range SHALL be clamped (`opacity` and `spread` `0..=100`, `size` `0..=250`). A
missing `lfx2` block, a missing `OrGl`, an unknown data version, a wrong-typed or
non-finite numeric value (including a finite `f64` that overflows to infinity as
an `f32`), a wrong `Md  ` typeID, a wrong `GlwT` typeID, a non-`RGBC` `Clr `, or
a descriptor that fails to parse SHALL be a no-op (`None`) and SHALL NOT panic.
An `OrGl` whose `present` or `enab` is false SHALL decode but SHALL render
nothing.

#### Scenario: An outer glow descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains an `OrGl` with `enab` true, `present` true, `Md  ` value `scrn`, a non-default `Clr `, `Opct` 60, `GlwT` value `PrBL`, `Ckmt` 20, and `blur` 10
- **THEN** `decode_outer_glow` returns an `OuterGlow` whose `enabled` and `present` are true, `blend_mode` is `Screen`, `color` is the stored colour, `opacity` is 60, `technique` is `Precise`, `spread` is 20, and `size` is 10

#### Scenario: Missing outer glow keys take Photoshop defaults

- **WHEN** an `OrGl` object carries only `enab` true and `present` true
- **THEN** the decoded `OuterGlow` has blend mode `Screen`, colour `#FFFFBE` (255, 255, 190), opacity 75, technique `Softer`, spread 0, and size 5

#### Scenario: Out-of-range numeric values are clamped or rejected

- **WHEN** an `OrGl` carries `blur` 1e30 and `Ckmt` 1e30
- **THEN** `decode_outer_glow` returns an `OuterGlow` with `size` 250 and `spread` 100, and rendering it does not panic
- **AND WHEN** an `OrGl` carries `blur` 1e300 (a finite `f64` that overflows `f32`)
- **THEN** `decode_outer_glow` returns `None` and rendering does not panic

#### Scenario: A disabled outer glow decodes but is inert

- **WHEN** an `OrGl` object has `present` true and `enab` false
- **THEN** `decode_outer_glow` returns an `OuterGlow` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Malformed or absent effects are a no-op

- **WHEN** a layer has no `lfx2` block, or its `lfx2` block lacks an `OrGl`, has an unknown data version, a wrong-typed `GlwT`, or fails to parse
- **THEN** `decode_outer_glow` returns `None` and does not panic

### Requirement: An outer glow composites behind the layer content

For an enabled `OuterGlow` on a visible non-group layer with `present` true, the CPU compositor SHALL composite the glow into the running canvas immediately before the layer's own content, with no offset, so the glow lies over the layers below and behind and around the layer content. It SHALL build a coverage matte
from the layer's content alpha — a pixel layer's alpha channel, a solid fill's
payload alpha, an opaque gradient or embedded smart-object coverage, or a pattern
fill's resolved pattern alpha — multiplied by the layer mask. It SHALL dilate the
matte by a spread radius derived from `spread` (a max filter), Gaussian-blur it
by a radius of `size` pixels using the crate's existing blur, and multiply the
result by the exterior mask `1 − matte` so the glow is zero where the content is
opaque. It SHALL multiply the result by `opacity/100` and composite it with the
effect's `blend_mode` and `color`, without applying the layer's own opacity or
fill a second time. `Technique::Softer` and `Technique::Precise` SHALL both
render as this blurred exterior matte. A zero-area, fully off-canvas, or empty
matte, or `opacity` 0 SHALL be a no-op. A layer with no effect, or with a
disabled or not-present effect, SHALL composite byte-identically to the same
document without the effect. The glow SHALL be clipped to the canvas, SHALL NOT
panic, and SHALL NOT add a dependency.

#### Scenario: The glow surrounds the content

- **WHEN** an opaque square layer over an opaque white backdrop has an outer glow with `size` 3, `spread` 0, opacity 100, and a non-white colour
- **THEN** backdrop pixels adjacent to the square are tinted toward the glow colour and pixels beyond the glow's reach are unchanged

#### Scenario: The glow is exterior and leaves the content unchanged

- **WHEN** the content pixel of the glowing layer is fully opaque
- **THEN** the composited content pixel keeps the content colour, because the glow is knocked out by the content and composited behind it

#### Scenario: Spread extends the glow

- **WHEN** the same layer is rendered with `spread` 0 and with a positive `spread`
- **THEN** the positive-spread glow covers no fewer pixels than the unspread glow, and `spread` 100 with a non-zero `size` produces a harder edge than `spread` 0

#### Scenario: Size softens the glow

- **WHEN** the same layer is rendered with `size` 0 and with `size` 6
- **THEN** the `size` 6 glow transitions across more backdrop pixels than the `size` 0 result (which, with `spread` 0, is a no-op), so a larger `size` widens the glow's edge

#### Scenario: Opacity and colour shape the glow

- **WHEN** a glow with `opacity` 50 and a red `color` is composited over a white backdrop
- **THEN** the glowing pixels are a partially transparent red tint rather than black, and lowering `opacity` toward 0 approaches the backdrop

#### Scenario: The layer mask shapes the glow

- **WHEN** a glowing layer's mask hides half the canvas
- **THEN** the glow is absent where the mask is zero and present where the mask is 255

#### Scenario: A disabled or absent glow is a no-op

- **WHEN** a layer carries no `lfx2` block, or an `OrGl` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect

#### Scenario: The glow is bounded to the content bbox plus reach

- **WHEN** a small content layer with a large `size` is composited on a much larger canvas
- **THEN** pixels outside the content rect padded by the spread and blur reach are byte-identical to the same document without the effect
