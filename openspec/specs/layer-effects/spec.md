# layer-effects Specification

## Purpose
TBD - created by archiving change layer-effects-drop-shadow. Update Purpose after archive.
## Requirements
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

### Requirement: The object-based effects descriptor decodes an inner shadow

`pictura-render` SHALL derive a typed `InnerShadow { enabled, present,
blend_mode, color, opacity, angle_deg, distance, choke, size, use_global_angle,
knocks_out }` from a layer's `lfx2` (`OBJECT_BASED_EFFECTS_LAYER_INFO`) tagged
block. The block SHALL be read as a `DescriptorBlock2` (a `u32` version and a
`u32` data version, then a version-16 descriptor body) and the top-level `IrSh`
object SHALL be decoded. The decoder SHALL read `enab` as `enabled`, `present` as
`present`, `Md  ` (typeID `BlnM`, a normal Photoshop blend-mode key) as
`blend_mode`, `Clr ` (an `RGBC` object whose `Rd `/`Grn `/`Bl  ` values are on the
`0..=255` scale) as `color`, `Opct` as `opacity` in percent, `uglg` as
`use_global_angle`, `lagl` as `angle_deg` in degrees, `Dstn` as `distance` in
pixels, `Ckmt` as `choke` in percent, `blur` as `size` in pixels, and
`layerConceals` as `knocks_out`. Numeric values SHALL be accepted as either a
unit float or a `doub`. A missing `Md  ` SHALL decode to `Multiply`, a missing
colour SHALL decode to black, an unknown blend-mode key SHALL decode to
`Multiply`, and absent numeric or boolean keys SHALL take the Photoshop defaults
(opacity 75, angle 120, distance 5, choke 0, size 5, use-global-angle true,
knocks-out true). A finite value outside its documented range SHALL be clamped
(`opacity` and `choke` `0..=100`, `distance` `0..=30000`, `size` `0..=250`). A
missing `lfx2` block, a missing `IrSh`, an unknown data version, a wrong-typed or
non-finite numeric value (including a finite `f64` that overflows to infinity as
an `f32`), a wrong `Md  ` typeID, a non-`RGBC` `Clr `, or a descriptor that fails
to parse SHALL be a no-op (`None`) and SHALL NOT panic. An `IrSh` whose `present`
or `enab` is false SHALL decode but SHALL render nothing.

#### Scenario: An inner shadow descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains an `IrSh` with `enab` true, `present` true, `Md  ` value `mul `, a non-black `Clr `, `Opct` 60, `uglg` false, `lagl` 45, `Dstn` 8, `Ckmt` 20, `blur` 12, and `layerConceals` false
- **THEN** `decode_inner_shadow` returns an `InnerShadow` whose `enabled` and `present` are true, `blend_mode` is `Multiply`, `color` is the stored colour, `opacity` is 60, `angle_deg` is 45, `distance` is 8, `choke` is 20, `size` is 12, `use_global_angle` is false, and `knocks_out` is false

#### Scenario: Missing inner shadow keys take Photoshop defaults

- **WHEN** an `IrSh` object carries only `enab` true and `present` true
- **THEN** the decoded `InnerShadow` has blend mode `Multiply`, black colour, opacity 75, angle 120, distance 5, choke 0, size 5, use-global-angle true, and knocks-out true

#### Scenario: Out-of-range numeric values are clamped or rejected

- **WHEN** an `IrSh` carries `blur` 1e30, `Dstn` 1e30, and `Ckmt` 1e30
- **THEN** `decode_inner_shadow` returns an `InnerShadow` with `size` 250, `distance` 30000, and `choke` 100, and rendering it does not panic
- **AND WHEN** an `IrSh` carries `blur` 1e300 (a finite `f64` that overflows `f32`)
- **THEN** `decode_inner_shadow` returns `None` and rendering does not panic

#### Scenario: A disabled inner shadow decodes but is inert

- **WHEN** an `IrSh` object has `present` true and `enab` false
- **THEN** `decode_inner_shadow` returns an `InnerShadow` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Malformed or absent effects are a no-op

- **WHEN** a layer has no `lfx2` block, or its `lfx2` block lacks an `IrSh`, has an unknown data version, a wrong-typed `Md  `, or fails to parse
- **THEN** `decode_inner_shadow` returns `None` and does not panic

### Requirement: An inner shadow composites inside the layer content

An enabled `InnerShadow` SHALL be composited inside the layer content. On a
visible non-group, non-destructive-adjustment layer with `present` true the CPU
compositor SHALL composite it into the running canvas **after** the layer's own
content, so the shadow darkens the content from within. It SHALL build a
coverage matte `M` from the layer's content
alpha — a pixel layer's alpha channel, a solid fill's payload alpha, an opaque
gradient or embedded smart-object coverage, or a pattern fill's resolved pattern
alpha — multiplied by the layer mask. It SHALL form the inverted matte `1 − M`,
offset it by `distance` pixels along the effect angle using
`dx = -distance·cos(angle)` and `dy = +distance·sin(angle)` in screen coordinates
(y down), erode it by a choke radius derived from `choke` (a min filter),
Gaussian-blur it by a radius of `size` pixels using the crate's existing blur,
and multiply the result by `M`, so the effect is confined to the content
interior. It SHALL multiply the result by `opacity/100` and composite it with the
effect's `blend_mode` and no extra layer opacity or fill. `knocks_out` SHALL be
decoded but SHALL NOT change the render. A zero-area, fully off-canvas, or empty
matte, or `opacity` 0 SHALL be a no-op. Every pixel outside the content coverage
`M` SHALL be byte-identical to the same document without the effect. A layer with
no effect, or with a disabled or not-present effect, SHALL composite
byte-identically to the same document without the effect. The shadow SHALL be
clipped to the canvas, SHALL NOT panic, and SHALL NOT add a dependency.

#### Scenario: The shadow is interior and leaves exterior pixels unchanged

- **WHEN** an opaque square layer over an opaque backdrop has an inner shadow with a non-zero `distance` and colour, and the composite is compared with the same document without the effect
- **THEN** pixels outside the square's coverage are byte-identical, and at least one interior pixel near an edge is darkened toward the shadow colour

#### Scenario: The shadow appears on the interior edge away from the offset

- **WHEN** an opaque square layer has an inner shadow with `distance` 4, `size` 0, `choke` 0, opacity 100, and `angle_deg` 0
- **THEN** an interior pixel near the right edge is darkened and an interior pixel near the left edge is unchanged

#### Scenario: Distance widens the interior band

- **WHEN** the same layer is rendered with `distance` 2 and with `distance` 6 at `angle_deg` 0
- **THEN** the darkened interior band for `distance` 6 extends farther from the edge than for `distance` 2

#### Scenario: Choke narrows the interior band

- **WHEN** the same layer is rendered with `choke` 0 and with a positive `choke` at a non-zero `size`
- **THEN** the positive-choke shadow covers no more interior pixels than the unchoked shadow

#### Scenario: Size softens the interior edge

- **WHEN** the same layer is rendered with `size` 0 and with `size` 6
- **THEN** the shadowed interior transition for `size` 6 spans more pixels than the hard `size` 0 band

#### Scenario: Opacity and colour shape the shadow

- **WHEN** an inner shadow with `opacity` 50 and a red `color` is composited over an opaque white content layer
- **THEN** the shadowed interior pixels are a partially transparent red tint rather than black, and lowering `opacity` toward 0 approaches the unshadowed content

#### Scenario: The layer mask shapes the matte

- **WHEN** an inner-shadowed layer's mask hides half the canvas
- **THEN** the shadow is absent where the mask is zero and present where the mask is 255

#### Scenario: The shadow is bounded to the content rect

- **WHEN** a small content layer with a large `size` is composited on a much larger canvas
- **THEN** every pixel outside the content rect is byte-identical to the same document without the effect

#### Scenario: A disabled or absent effect is a no-op

- **WHEN** a layer carries no `lfx2` block, or an `IrSh` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect

### Requirement: The object-based effects descriptor decodes an inner glow

`pictura-render` SHALL derive a typed `InnerGlow { enabled, present,
blend_mode, color, opacity, choke, size, source, technique }` from a layer's
`lfx2` (`OBJECT_BASED_EFFECTS_LAYER_INFO`) tagged block. The block SHALL be read
as a `DescriptorBlock2` (a `u32` version and a `u32` data version, then a
version-16 descriptor body) and the top-level `IrGl` object SHALL be decoded. The
decoder SHALL read `enab` as `enabled`, `present` as `present`, `Md  ` (typeID
`BlnM`, a normal Photoshop blend-mode key) as `blend_mode`, `Clr ` (an `RGBC`
object whose `Rd `/`Grn `/`Bl  ` values are on the `0..=255` scale) as `color`,
`Opct` as `opacity` in percent, `Ckmt` as `choke` in percent, `blur` as `size` in
pixels, `GlwT` (typeID `BETE`, values `SfBL` Softer or `PrBL` Precise) as
`technique`, and `glwS` (typeID `IGSr`, values `SrcE` Edge or `SrcC` Center) as
`source`. The decoder SHALL also accept the legacy/mis-authored `IGsr` typeID
for `glwS` leniently. Numeric values SHALL be accepted as either a unit float or
a `doub`. A
missing `Md  ` SHALL decode to `Screen`, a missing colour SHALL decode to white,
a missing `GlwT` or an unknown technique value SHALL decode to `Softer`, and a
missing `glwS` or an unknown source value SHALL decode to `Edge`. An unknown
blend-mode key SHALL decode to `Screen`. Absent numeric keys SHALL take the
Photoshop defaults (opacity 75, choke 0, size 5). A finite value outside its
documented range SHALL be clamped (`opacity` and `choke` `0..=100`, `size`
`0..=250`). A missing `lfx2` block, a missing `IrGl`, an unknown data version, a
wrong-typed or non-finite numeric value (including a finite `f64` that overflows
to infinity as an `f32`), a wrong `Md  `, `GlwT` or `glwS` typeID, a non-`RGBC`
`Clr `, or a descriptor that fails to parse SHALL be a no-op (`None`) and SHALL
NOT panic. An `IrGl` whose `present` or `enab` is false SHALL decode but SHALL
render nothing.

#### Scenario: An inner glow descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains an `IrGl` with `enab` true, `present` true, `Md  ` value `scrn`, a non-white `Clr `, `Opct` 60, `Ckmt` 20, `blur` 12, `GlwT` `BETE`/`PrBL`, and `glwS` `IGSr`/`SrcC`
- **THEN** `decode_inner_glow` returns an `InnerGlow` whose `enabled` and `present` are true, `blend_mode` is `Screen`, `color` is the stored colour, `opacity` is 60, `choke` is 20, `size` is 12, `technique` is `Precise`, and `source` is `Center`

#### Scenario: Missing inner glow keys take Photoshop defaults

- **WHEN** an `IrGl` object carries only `enab` true and `present` true
- **THEN** the decoded `InnerGlow` has blend mode `Screen`, white colour, opacity 75, choke 0, size 5, `technique` `Softer`, and `source` `Edge`

#### Scenario: The source key is decoded from the IGSr enum

- **WHEN** an `IrGl` carries `glwS` with typeID `IGSr` and value `SrcE`, and another carries value `SrcC`
- **THEN** the first decodes to `source` `Edge` and the second to `source` `Center`, and rendering the two produces different composites
- **AND WHEN** an `IrGl` carries `glwS` with the legacy typeID `IGsr`
- **THEN** it still decodes, and a wrong typeID such as `XXXX` is `None`

#### Scenario: Out-of-range numeric values are clamped or rejected

- **WHEN** an `IrGl` carries `blur` 1e30 and `Ckmt` 1e30
- **THEN** `decode_inner_glow` returns an `InnerGlow` with `size` 250 and `choke` 100, and rendering it does not panic
- **AND WHEN** an `IrGl` carries `blur` 1e300 (a finite `f64` that overflows `f32`)
- **THEN** `decode_inner_glow` returns `None` and rendering does not panic

#### Scenario: A disabled inner glow decodes but is inert

- **WHEN** an `IrGl` object has `present` true and `enab` false
- **THEN** `decode_inner_glow` returns an `InnerGlow` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Malformed or absent effects are a no-op

- **WHEN** a layer has no `lfx2` block, or its `lfx2` block lacks an `IrGl`, has an unknown data version, a wrong-typed `Md  `, a wrong `GlwT` or `glwS` typeID, a non-`RGBC` `Clr `, or fails to parse
- **THEN** `decode_inner_glow` returns `None` and does not panic

### Requirement: An inner glow composites inside the layer content

An enabled `InnerGlow` SHALL be composited inside the layer content. On a visible
non-group, non-destructive-adjustment layer with `present` true the CPU
compositor SHALL composite it into the running canvas **after** the layer's own
content, so the glow tints the content from within. It SHALL build a coverage
matte `M` from the layer's content alpha — a pixel layer's alpha channel, a solid
fill's payload alpha, an opaque gradient or embedded smart-object coverage, or a
pattern fill's resolved pattern alpha — multiplied by the layer mask. It SHALL
erode `M` by a choke radius derived from `choke` (a min filter of radius
`round(choke / 100 · size)`), Gaussian-blur the result by a radius of `size`
pixels into `B` using the crate's existing blur, and derive the interior field
`field = 1 - B` for `Source = Edge` or `field = B` for `Source = Center`, then
composite `M · field · opacity/100`. It SHALL composite with the effect's
`blend_mode` and no extra layer opacity or fill.

For `Source = Edge` the field SHALL be strongest at the content edge and fade
inward; for `Source = Center` the field SHALL be strongest away from the edge and
lower at the edge. The `Center` field is the complement of the `Edge` field, an
approximation of Photoshop's distance-from-center falloff. `technique` `Precise`
SHALL render as `Softer`. A zero-area, fully off-canvas, or empty matte, or
`opacity` 0, or (`size` 0 with `choke` 0) under `Source = Edge` SHALL be a no-op.
Every pixel outside the content coverage `M` SHALL be byte-identical to the same
document without the effect. A layer with no effect, or with a disabled or
not-present effect, SHALL composite byte-identically to the same document without
the effect. The glow SHALL be clipped to the canvas, SHALL NOT panic, and SHALL
NOT add a dependency.

#### Scenario: The glow is interior and leaves exterior pixels unchanged

- **WHEN** an opaque square layer over an opaque backdrop has an inner glow with a non-zero `size` and colour, and the composite is compared with the same document without the effect
- **THEN** pixels outside the square's coverage are byte-identical, and at least one interior pixel near an edge is tinted toward the glow colour

#### Scenario: Edge and center sources produce different fields

- **WHEN** the same inner-glowing layer is rendered once with `source` `Edge` and once with `source` `Center`
- **THEN** the two composites differ, with the Edge run tinting pixels adjacent to the content edge and the Center run tinting the interior more than the edge

#### Scenario: Choke strengthens the glow toward the edge

- **WHEN** the same layer is rendered with `choke` 0 and with a positive `choke` at a non-zero `size`
- **THEN** the positive-choke glow is at least as strong in the interior band nearest the edge as the unchoked glow

#### Scenario: Size changes the glow falloff

- **WHEN** the same layer is rendered with `size` 0 and with `size` 6 under `Source = Edge`
- **THEN** `size` 0 produces a byte-identical composite to no effect and `size` 6 tints a band of interior pixels near the edge

#### Scenario: Opacity and colour shape the glow

- **WHEN** an inner glow with `opacity` 50 and a red `color` is composited over an opaque white content layer
- **THEN** the tinted interior pixels are a partially transparent red rather than white, and lowering `opacity` toward 0 approaches the unglowed content

#### Scenario: The layer mask shapes the matte

- **WHEN** an inner-glowing layer's mask hides half the canvas
- **THEN** the glow is absent where the mask is zero and present where the mask is 255

#### Scenario: The glow is bounded to the content rect

- **WHEN** a small content layer with a large `size` is composited on a much larger canvas
- **THEN** every pixel outside the content rect is byte-identical to the same document without the effect

#### Scenario: A disabled or absent effect is a no-op

- **WHEN** a layer carries no `lfx2` block, or an `IrGl` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect

### Requirement: The object-based effects descriptor decodes a stroke

`pictura-render` SHALL derive a typed `Stroke { enabled, present, blend_mode,
color, opacity, size, position }` from a layer's `lfx2`
(`OBJECT_BASED_EFFECTS_LAYER_INFO`) tagged block. The block SHALL be read as a
`DescriptorBlock2` (a `u32` version and a `u32` data version, then a version-16
descriptor body) and the top-level `FrFX` object SHALL be decoded. The decoder
SHALL read `enab` as `enabled`, `present` as `present`, `Md  ` (typeID `BlnM`,
a normal Photoshop blend-mode key) as `blend_mode`, `Clr ` (an `RGBC` object
whose `Rd `/`Grn `/`Bl  ` values are on the `0..=255` scale) as `color`,
`Opct` as `opacity` in percent, `Sz  ` as `size` in pixels, `Styl` (typeID
`FStl`, values `OutF` Outside, `InsF` Inside, `CtrF` Center) as `position`, and
`PntT` (typeID `FrFl`) as the fill type. Numeric values SHALL be accepted as
either a unit float or a `doub`. A missing `Md  ` SHALL decode to `Normal`, a
missing colour SHALL decode to black, an unknown blend-mode key SHALL decode to
`Normal`, and a missing `Styl` or an unknown position value SHALL decode to
Outside. Absent numeric keys SHALL take the Photoshop defaults (opacity 100,
size 3). `size` SHALL be an integer: a finite decoded value SHALL be rounded to
the nearest integer and clamped to `1..=250`, and `opacity` SHALL be clamped to
`0..=100`. `PntT` SHALL default to `SClr`; the value `SClr` SHALL decode as a
solid-colour fill, and the values `GrFl` (gradient) or `Ptrn` (pattern), or any
unknown value, SHALL make the decoder return `None` (a documented ceiling: only
the solid-colour stroke is decoded in this slice). A missing `lfx2` block, a
missing `FrFX`, an unknown data version, a wrong-typed or non-finite numeric
value (including a finite `f64` that overflows to infinity as an `f32`), a wrong
`Md  `, `Styl` or `PntT` typeID, a non-`RGBC` `Clr `, or a descriptor that fails
to parse SHALL be a no-op (`None`) and SHALL NOT panic. An `FrFX` whose `present`
or `enab` is false SHALL decode but SHALL render nothing.

#### Scenario: A stroke descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains an `FrFX` with `enab` true, `present` true, `Md  ` value `mul `, a non-black `Clr `, `Opct` 60, `Sz  ` 12, `Styl` `FStl`/`InsF`, and `PntT` `FrFl`/`SClr`
- **THEN** `decode_stroke` returns a `Stroke` whose `enabled` and `present` are true, `blend_mode` is `Multiply`, `color` is the stored colour, `opacity` is 60, `size` is 12, and `position` is `Inside`

#### Scenario: Missing stroke keys take Photoshop defaults

- **WHEN** an `FrFX` object carries only `enab` true and `present` true
- **THEN** the decoded `Stroke` has blend mode `Normal`, black colour, opacity 100, size 3, and position `Outside`

#### Scenario: The position and fill-type keys are decoded from their enums

- **WHEN** an `FrFX` carries `Styl` typeID `FStl` with value `OutF`, another with `InsF`, and another with `CtrF`
- **THEN** the first decodes to `position` `Outside`, the second to `Inside`, and the third to `Center`
- **AND WHEN** an `FrFX` carries `Styl` with a wrong typeID such as `XXXX`, or `PntT` with a wrong typeID such as `XXXX`
- **THEN** `decode_stroke` returns `None`

#### Scenario: A non-solid fill type is deferred

- **WHEN** an `FrFX` carries `PntT` typeID `FrFl` with value `GrFl` or `Ptrn`
- **THEN** `decode_stroke` returns `None`, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Out-of-range numeric values are clamped or rejected

- **WHEN** an `FrFX` carries `Sz  ` 1e30 and `Opct` 1e30
- **THEN** `decode_stroke` returns a `Stroke` with `size` 250 and `opacity` 100, and rendering it does not panic
- **AND WHEN** an `FrFX` carries `Sz  ` 1e300 (a finite `f64` that overflows `f32`)
- **THEN** `decode_stroke` returns `None` and rendering does not panic

#### Scenario: A disabled stroke decodes but is inert

- **WHEN** an `FrFX` object has `present` true and `enab` false
- **THEN** `decode_stroke` returns a `Stroke` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Malformed or absent effects are a no-op

- **WHEN** a layer has no `lfx2` block, or its `lfx2` block lacks an `FrFX`, has an unknown data version, a wrong-typed `Md  `, a non-`RGBC` `Clr `, or fails to parse
- **THEN** `decode_stroke` returns `None` and does not panic

### Requirement: A stroke composites as a band at the content edge

An enabled `Stroke` SHALL be composited as a band at the layer's content edge,
above the layer. On a visible non-group, non-destructive-adjustment layer with
`present` true the CPU compositor SHALL composite it into the running canvas
**after** the layer's own content, so the band is drawn on top of the content.
It SHALL build a coverage matte `M` from the layer's content alpha — a pixel
layer's alpha channel, a solid fill's payload alpha, an opaque gradient or
embedded smart-object coverage, or a pattern fill's resolved pattern alpha —
multiplied by the layer mask. Let `n` be the integer `size`; the band SHALL be
formed from exact integer max/min filters with no blur:

- `position` **Outside**: `band(x, y) = dilate(M, n) − M`;
- `position` **Inside**: `band(x, y) = M − erode(M, n)`;
- `position` **Center**: with `out_r = ceil(n/2)` and `in_r = floor(n/2)`,
  `band(x, y) = dilate(M, out_r) − erode(M, in_r)`.

`dilate` is a separable max filter of the given integer radius and `erode` is a
separable min filter of the given radius; the band value SHALL be clamped to
`0..=1`. The compositor SHALL composite `band · opacity/100` tinted by `color`
with the effect's own `blend_mode` and no extra layer opacity or fill. Every
pixel outside the content rect padded by `n` on each side SHALL be byte-identical
to the same document without the effect. The Outside band SHALL be zero where
`M` is 1, so it SHALL NOT alter the content interior; the Inside band SHALL be
zero where `M` is 0, so it SHALL NOT alter the exterior beyond the content; the
Center band SHALL have pixels both outside and inside the content edge. A
zero-area or fully off-canvas rect, an empty matte, `opacity` 0, or a hand-built
`size` of 0 SHALL be a no-op (a decoded `Sz  ` of 0 clamps to the minimum size
of 1, so it renders a 1-pixel band rather than a no-op). A layer with no effect, or with a disabled or not-present
effect, SHALL composite byte-identically to the same document without the effect.
The stroke SHALL be clipped to the canvas, SHALL NOT panic, and SHALL NOT add a
dependency.

#### Scenario: The outside band lies outside the content and leaves the interior unchanged

- **WHEN** an opaque square layer over an opaque backdrop has a stroke with `position` `Outside`, a non-zero `size`, opacity 100, and a non-content-colour `color`, and the composite is compared with the same document without the effect
- **THEN** at least one pixel just outside the square is tinted toward the stroke colour, every pixel inside the square's opaque coverage is byte-identical, and pixels beyond the padded reach are byte-identical

#### Scenario: The inside band is confined to the interior

- **WHEN** an opaque square layer over an opaque backdrop has a stroke with `position` `Inside`, a non-zero `size`, opacity 100, and a non-content-colour `color`
- **THEN** at least one pixel just inside the square's edge is tinted toward the stroke colour, and every pixel outside the square's coverage is byte-identical to the same document without the effect

#### Scenario: The centre band straddles the content edge

- **WHEN** the same layer is rendered with `position` `Center` and a non-zero `size`
- **THEN** at least one pixel outside the content edge and at least one pixel inside the content edge are tinted toward the stroke colour

#### Scenario: Size changes the band width

- **WHEN** the same layer is rendered with `size` 2 and with `size` 6 under `position` `Outside`
- **THEN** the `size` 6 band extends at least as far from the content edge as the `size` 2 band, and the `size` 6 band covers no fewer tinted pixels

#### Scenario: Opacity and colour shape the stroke

- **WHEN** a stroke with `opacity` 50 and a red `color` is composited over an opaque white content layer
- **THEN** the band pixels are a partially transparent red rather than white, and lowering `opacity` toward 0 approaches the unstroked content

#### Scenario: The blend mode shapes the stroke

- **WHEN** the same stroke is rendered once with `blend_mode` `Normal` and once with a different supported blend mode
- **THEN** the two composites differ

#### Scenario: The layer mask shapes the matte

- **WHEN** a stroked layer's mask hides half the canvas
- **THEN** the band is absent where the mask is zero and present where the mask is 255

#### Scenario: The stroke is bounded to the content rect plus reach

- **WHEN** a small content layer with a large `size` is composited on a much larger canvas
- **THEN** every pixel outside the content rect padded by `size` is byte-identical to the same document without the effect

#### Scenario: A disabled or absent effect is a no-op

- **WHEN** a layer carries no `lfx2` block, or an `FrFX` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect

### Requirement: The object-based effects descriptor decodes a color overlay

`pictura-render` SHALL derive a typed `ColorOverlay { enabled, present,
blend_mode, color, opacity }` from a layer's `lfx2`
(`OBJECT_BASED_EFFECTS_LAYER_INFO`) tagged block. The block SHALL be read as a
`DescriptorBlock2` (a `u32` version and a `u32` data version, then a version-16
descriptor body) and the top-level object whose class id is **`SoFi`** SHALL be
decoded. The decoder SHALL read `enab` as `enabled`, `present` as `present`,
`Md  ` (typeID `BlnM`) as `blend_mode`, `Clr ` (an `RGBC` object whose
`Rd `/`Grn `/`Bl  ` values are on the `0..=255` scale) as `color`, and `Opct` as
`opacity` in percent. Numeric values SHALL be accepted as either a unit float or
a `doub`. A missing `Md  ` or an unknown blend-mode key SHALL decode to
`Normal`; a missing `Clr ` SHALL decode to red `[255, 0, 0]` (the Photoshop
default, recorded as `(inferred)` — a real file always carries `Clr ` and this
crate cannot resolve a document foreground); a missing `Opct` SHALL decode to
100; `opacity` SHALL be clamped to `0..=100`. A missing `lfx2` block, a missing
`SoFi`, an unknown data version, a wrong-typed or non-finite numeric value
(including a finite `f64` that overflows to infinity as an `f32`), a wrong
`Md  ` typeID, a non-`RGBC` `Clr `, or a descriptor that fails to parse SHALL be
a no-op (`None`) and SHALL NOT panic. A `SoFi` whose `present` or `enab` is
false SHALL decode but SHALL render nothing.

#### Scenario: A color-overlay descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains a `SoFi` object with `enab` true, `present` true, `Md  ` `BlnM`/`mul `, a non-black `Clr `, and `Opct` 60
- **THEN** `decode_color_overlay` returns a `ColorOverlay` whose `enabled` and `present` are true, `blend_mode` is `Multiply`, `color` is the stored colour, and `opacity` is 60

#### Scenario: Missing color-overlay keys take Photoshop defaults

- **WHEN** a `SoFi` object carries only `enab` true and `present` true
- **THEN** the decoded `ColorOverlay` has blend mode `Normal`, red `[255, 0, 0]`, and opacity 100

#### Scenario: Out-of-range or non-finite color-overlay numerics are clamped or rejected

- **WHEN** a `SoFi` carries `Opct` 1e30
- **THEN** `decode_color_overlay` returns `opacity` 100 and rendering does not panic
- **AND WHEN** a `SoFi` carries `Opct` 1e300 (a finite `f64` that overflows `f32`)
- **THEN** `decode_color_overlay` returns `None` and rendering does not panic

#### Scenario: A disabled color overlay decodes but is inert

- **WHEN** a `SoFi` object has `present` true and `enab` false
- **THEN** `decode_color_overlay` returns a `ColorOverlay` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Malformed or absent color-overlay effects are a no-op

- **WHEN** a layer has no `lfx2` block, or its `lfx2` block lacks a `SoFi`, has an unknown data version, a `SoFi` with a wrong class id, a wrong-typed `Md  `, a non-`RGBC` `Clr `, or fails to parse
- **THEN** `decode_color_overlay` returns `None` and does not panic

### Requirement: A color overlay fills the content coverage above the layer

An enabled `ColorOverlay` SHALL be composited as a flat colour over the layer's
content coverage, above the layer. On a visible non-group,
non-destructive-adjustment layer with `present` true the CPU compositor SHALL
composite it **after** the layer's own content. It SHALL build a coverage matte
`M` from the layer's content alpha — a pixel layer's alpha channel, a solid
fill's payload alpha, an opaque gradient or embedded smart-object coverage, or a
pattern fill's resolved pattern alpha — multiplied by the layer mask, and SHALL
composite, for each pixel with `M > 0`, the colour `color` with alpha
`M · opacity/100` using the overlay's own `blend_mode` and no extra layer opacity
or fill. Every pixel outside the content rect SHALL be byte-identical to the same
document without the effect. A zero-area or fully off-canvas rect, an empty
matte, or `opacity` 0 SHALL be a no-op. A layer with no effect, or with a
disabled or not-present effect, SHALL composite byte-identically to the same
document without the effect. The overlay SHALL be clipped to the canvas, SHALL
NOT panic, and SHALL NOT add a dependency.

#### Scenario: The color overlay is confined to the content

- **WHEN** an opaque square layer over an opaque backdrop has an enabled and present color overlay with a non-content colour and opacity 100, and the composite is compared with the same document without the effect
- **THEN** every pixel inside the square's opaque coverage is the overlay colour and every pixel outside it is byte-identical

#### Scenario: Opacity and blend mode shape the color overlay

- **WHEN** the same layer is rendered with `opacity` 50 and with `opacity` 100, and once with `blend_mode` `Normal` and once with a different supported blend mode
- **THEN** the 50 % composite is a partial blend toward the content, the 100 % composite is the overlay colour, and the two blend-mode composites differ

#### Scenario: The layer mask shapes the color overlay

- **WHEN** a layer's mask hides half the canvas
- **THEN** the overlay is absent where the mask is zero and present where the mask is 255

#### Scenario: The color overlay is bounded to the content rect

- **WHEN** a small content layer is composited on a much larger canvas
- **THEN** every pixel outside the content rect is byte-identical to the same document without the effect

#### Scenario: A disabled or absent color overlay is a no-op

- **WHEN** a layer carries no `lfx2` block, or a `SoFi` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect

### Requirement: The object-based effects descriptor decodes a gradient overlay

`pictura-render` SHALL derive a typed `GradientOverlay { enabled, present,
blend_mode, opacity, stops, reverse, kind, angle_deg, scale, align_with_layer }`
from a layer's `lfx2` block, reusing `pictura-adjust`'s `GradientStop` and
`GradientKind`. The decoder SHALL find the top-level object whose class id is
**`GrFl`** and read `enab`, `present`, `Md  ` (typeID `BlnM`) as `blend_mode`
(default/unknown → `Normal`), `Opct` as `opacity` in percent (default 100,
clamped `0..=100`), `Grad` (a `Grdn` object carrying `GrdF` `CstS` and a `Clrs`
list of custom stops) as `stops`, `Angl` as `angle_deg` (default 0), `Type`
(typeID `GrdT`) as `kind` (default `Linear`), `Rvrs` as `reverse` (default
false), `Scl ` as `scale` in percent (default 100), and `Algn` as
`align_with_layer` (default true). `Ofst`, noise and `Dthr` SHALL be ignored.
Numeric values SHALL be accepted as a unit float or a `doub`; a finite
out-of-range `opacity` SHALL be clamped; a non-finite value (including a finite
`f64` that overflows `f32`) SHALL reject the effect. A missing `lfx2`, a missing
`GrFl`, an unknown data version, a wrong class id, a wrong `Md  `/`Type`/`GrdF`
typeID, a `Grad` that is not an object, a stop list with fewer than two or
non-increasing locations, or a parse error SHALL be `None`, and the decoder
SHALL NOT panic. A `GrFl` whose `present` or `enab` is false SHALL decode but
SHALL render nothing.

#### Scenario: A gradient-overlay descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains a `GrFl` with `enab` true, `present` true, `Md  ` `BlnM`/`Nrml`, `Opct` 80, a custom two-stop black-to-white `Grad`, `Angl` 45, `Type` `GrdT`/`Lnr `, `Rvrs` true, `Scl ` 150, and `Algn` false
- **THEN** the decoded `GradientOverlay` has `enabled`/`present` true, `blend_mode` `Normal`, `opacity` 80, two stops, `reverse` true, `kind` `Linear`, `angle_deg` 45, `scale` 150, and `align_with_layer` false

#### Scenario: Missing gradient-overlay keys take Photoshop defaults

- **WHEN** a `GrFl` carries a valid `Grad` and omits `Rvrs`, `Scl `, `Algn`, `Md  ` and `Opct`
- **THEN** the decoded `GradientOverlay` has `reverse` false, `scale` 100, `align_with_layer` true, blend mode `Normal`, and opacity 100

#### Scenario: A malformed gradient overlay is a no-op

- **WHEN** a `GrFl` has a wrong `Type` typeID, a `Grad` that is not an object, a `GrdF` that is not `CstS`, a stop list with fewer than two stops or non-increasing locations, or a non-finite `Angl`
- **THEN** `decode_gradient_overlay` returns `None` and does not panic

#### Scenario: A disabled gradient overlay decodes but is inert

- **WHEN** a `GrFl` has `present` true and `enab` false
- **THEN** `decode_gradient_overlay` returns a `GradientOverlay` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

### Requirement: A gradient overlay fills the content coverage above the layer

An enabled `GradientOverlay` SHALL be composited as the gradient source over the
layer's content coverage, above the layer. On a visible non-group,
non-destructive-adjustment layer with `present` true the CPU compositor SHALL
generate the gradient with the shipped gradient geometry (respecting `kind`,
`angle_deg`, `scale` and `reverse`) and composite it **after** the layer's own
content. The gradient SHALL be generated over the **layer rect** when
`align_with_layer` is true (pixel `(x, y)` samples local `(x - left, y - top)`)
and over the **canvas** otherwise (pixel `(x, y)` samples `(x, y)`). Using the
masked content matte `M`, the compositor SHALL composite, for each pixel with
`M > 0`, the sampled gradient colour with alpha `M · opacity/100` using the
overlay's own `blend_mode` and no extra layer opacity or fill. Every pixel
outside the content rect SHALL be byte-identical to the same document without the
effect. A zero-area or fully off-canvas rect, an empty matte, or `opacity` 0
SHALL be a no-op. A layer with no effect, or with a disabled or not-present
effect, SHALL composite byte-identically to the same document without the effect.
The overlay SHALL be clipped to the canvas, SHALL NOT panic, and SHALL NOT add a
dependency.

#### Scenario: The gradient overlay is confined to the content

- **WHEN** a small opaque square layer over a large opaque backdrop has an enabled and present gradient overlay with opacity 100
- **THEN** every pixel outside the square's content rect is byte-identical to the same document without the effect, and at least one pixel inside varies with the gradient

#### Scenario: The gradient geometry follows the overlay parameters

- **WHEN** the same gradient overlay is rendered with two different `angle_deg`, `scale`, `kind` or `reverse` values
- **THEN** the composites differ

#### Scenario: Align with layer moves the gradient origin

- **WHEN** the same gradient overlay is rendered once with `align_with_layer` true and once false on a layer smaller than the canvas
- **THEN** the two composites differ, showing that the gradient geometry is anchored to the layer rect or the canvas respectively

#### Scenario: Opacity and blend mode shape the gradient overlay

- **WHEN** the same layer is rendered with `opacity` 50 and 100, and once with `blend_mode` `Normal` and once with a different supported blend mode
- **THEN** lowering opacity approaches the content, and the two blend-mode composites differ

#### Scenario: A disabled or absent gradient overlay is a no-op

- **WHEN** a layer carries no `lfx2` block, or a `GrFl` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect

### Requirement: The object-based effects descriptor decodes a pattern overlay

`pictura-render` SHALL derive a typed `PatternOverlay { enabled, present,
blend_mode, opacity, pattern_id, scale, angle_deg, align_with_layer }` from a
layer's `lfx2` block. The decoder SHALL find the top-level object whose class id
is **`patternFill`** and read `enab` and `present`, `Md  ` (typeID `BlnM`) as
`blend_mode` (default/unknown → `Normal`), `Opct` as `opacity` in percent
(default 100, clamped `0..=100`), `Ptrn` (a `Ptrn` object carrying `Nm  ` and a
non-empty `Idnt`) as `pattern_id`, `Scl ` as `scale` in percent (default 100),
`Algn` as `align_with_layer` (default true), and `Angl` as `angle_deg` (default
0, decoded for symmetry but not applied). An optional `phase` `Pnt ` object's
`Hrzn`/`Vrtc` SHALL supply the tile origin, defaulting to `(0, 0)`. Numeric
values SHALL be accepted as a unit float or a `doub`; a non-finite value
(including a finite `f64` that overflows `f32`) SHALL reject the effect. A
missing `lfx2`, a missing `patternFill`, an unknown data version, a wrong class
id, a wrong `Md  ` typeID, a `Ptrn` that is not an object, a missing `Idnt`, or
a parse error SHALL be `None`, and the decoder SHALL NOT panic. A `patternFill`
whose `present` or `enab` is false SHALL decode but SHALL render nothing.

#### Scenario: A pattern-overlay descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains a `patternFill` with `enab` true, `present` true, `Md  ` `BlnM`/`Nrml`, `Opct` 80, a `Ptrn` object with `Nm  ` and `Idnt` `pictura-pattern`, `Scl ` 50, `Angl` 30, and `Algn` true
- **THEN** the decoded `PatternOverlay` has `enabled`/`present` true, `blend_mode` `Normal`, `opacity` 80, `pattern_id` `pictura-pattern`, `scale` 50, `angle_deg` 30, and `align_with_layer` true

#### Scenario: Missing pattern-overlay keys take Photoshop defaults

- **WHEN** a `patternFill` carries a valid `Ptrn` and omits `Scl `, `Algn`, `Md  ` and `Opct`
- **THEN** the decoded `PatternOverlay` has `scale` 100, `align_with_layer` true, blend mode `Normal`, and opacity 100

#### Scenario: A malformed pattern overlay is a no-op

- **WHEN** a `patternFill` has a `Ptrn` that is not an object, a missing `Idnt`, a wrong-typed `Md  `, or a non-finite `Scl `
- **THEN** `decode_pattern_overlay` returns `None` and does not panic

#### Scenario: A disabled pattern overlay decodes but is inert

- **WHEN** a `patternFill` has `present` true and `enab` false
- **THEN** `decode_pattern_overlay` returns a `PatternOverlay` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

### Requirement: A pattern overlay fills the content coverage above the layer

An enabled `PatternOverlay` SHALL be composited as the pattern source over the
layer's content coverage, above the layer. On a visible non-group,
non-destructive-adjustment layer with `present` true the CPU compositor SHALL
resolve the pattern from the document's decoded pattern library and tile it with
the shipped tiling (respecting `scale`), and composite it **after** the layer's
own content. The tile SHALL be anchored to the layer rect when
`align_with_layer` is true and to the document origin otherwise, offset by the
`phase` origin when present; a pattern id that is not in the library SHALL render
the shipped grey placeholder rather than a no-op. Using the masked content matte
`M`, the compositor SHALL composite, for each pixel with `M > 0`, the pattern
colour with alpha `M · pattern_alpha/255 · opacity/100` using the overlay's own
`blend_mode` and no extra layer opacity or fill. `angle_deg` SHALL NOT be
applied. Every pixel outside the content rect SHALL be byte-identical to the same
document without the effect. A zero-area or fully off-canvas rect, an empty
matte, or `opacity` 0 SHALL be a no-op. A layer with no effect, or with a
disabled or not-present effect, SHALL composite byte-identically to the same
document without the effect. The overlay SHALL be clipped to the canvas, SHALL
NOT panic, and SHALL NOT add a dependency.

#### Scenario: The pattern overlay is confined to the content

- **WHEN** a small opaque square layer over a large opaque backdrop has an enabled and present pattern overlay with opacity 100
- **THEN** every pixel outside the square's content rect is byte-identical to the same document without the effect, and at least one pixel inside shows the pattern colour

#### Scenario: A missing pattern falls back to the placeholder

- **WHEN** a pattern overlay references a pattern id that is not in the document's pattern library
- **THEN** the composite inside the content shows the grey placeholder rather than being a no-op, and does not panic

#### Scenario: Scale and align with layer shape the pattern overlay

- **WHEN** the same pattern overlay is rendered with `scale` 100 and 200, and once with `align_with_layer` false instead of true
- **THEN** the tiled composites differ

#### Scenario: Opacity and blend mode shape the pattern overlay

- **WHEN** the same layer is rendered with `opacity` 50 and 100, and once with `blend_mode` `Normal` and once with a different supported blend mode
- **THEN** lowering opacity approaches the content, and the two blend-mode composites differ

#### Scenario: A disabled or absent pattern overlay is a no-op

- **WHEN** a layer carries no `lfx2` block, or a `patternFill` with `enab` false
- **THEN** the composite is byte-identical to the same document without the effect

### Requirement: The object-based effects descriptor decodes a satin

`pictura-render` SHALL derive a typed `Satin { enabled, present, blend_mode,
color, opacity, angle_deg, distance, size, invert }` from a layer's `lfx2`
(`OBJECT_BASED_EFFECTS_LAYER_INFO`) tagged block. The block SHALL be read as a
`DescriptorBlock2` (a `u32` version and a `u32` data version, then a version-16
descriptor body) and the top-level object whose class id is **`ChFX`** SHALL be
decoded. The decoder SHALL read `enab` as `enabled`, `present` as `present`,
`Md  ` (typeID `BlnM`) as `blend_mode`, `Clr ` (an `RGBC` object whose
`Rd `/`Grn `/`Bl  ` values are on the `0..=255` scale) as `color`, `Opct` as
`opacity` in percent, `lagl` as `angle_deg` in degrees, `Dstn` as `distance` in
pixels, `blur` as `size` in pixels, and `Invr` as `invert`. Numeric values SHALL
be accepted as either a unit float or a `doub`. A missing `Md  ` or an unknown
blend-mode key SHALL decode to `Multiply`; a missing `Clr ` SHALL decode to black
`[0, 0, 0]` (the Photoshop default, recorded as `(inferred)` — a real file always
carries `Clr ` and this crate cannot resolve a document foreground); absent
numeric keys SHALL take the Photoshop UI defaults (opacity 50, angle 19,
distance 11, size 14) and a missing `Invr` SHALL decode to false. A finite value
outside its documented range SHALL be clamped (`opacity` `0..=100`, `distance`
`0..=30000`, `size` `0..=250`). The contour (`MpgS`), anti-alias (`AntA`),
global-light flag (`uglg`) and `showInDialog` keys SHALL be ignored; the
effective angle SHALL be the stored `lagl`. A missing `lfx2` block, a missing
`ChFX`, an unknown data version, a wrong class id, a wrong-typed or non-finite
numeric value (including a finite `f64` that overflows to infinity as an `f32`),
a wrong `Md  ` typeID, a non-`RGBC` `Clr `, or a descriptor that fails to parse
SHALL be a no-op (`None`) and SHALL NOT panic. A `ChFX` whose `present` or `enab`
is false SHALL decode but SHALL render nothing.

#### Scenario: A satin descriptor decodes to typed parameters

- **WHEN** an `lfx2` block contains a `ChFX` object with `enab` true, `present` true, `Md  ` `BlnM`/`mul `, a non-black `Clr `, `Opct` 60, `lagl` 45, `Dstn` 8, `blur` 12, and `Invr` true
- **THEN** `decode_satin` returns a `Satin` whose `enabled` and `present` are true, `blend_mode` is `Multiply`, `color` is the stored colour, and `opacity`, `angle_deg`, `distance`, `size` and `invert` are the stored values

#### Scenario: Missing satin keys take Photoshop defaults

- **WHEN** a `ChFX` object carries only `enab` true and `present` true
- **THEN** the decoded `Satin` has blend mode `Multiply`, black colour, opacity 50, angle 19, distance 11, size 14, and `invert` false

#### Scenario: Out-of-range numeric values are clamped or rejected

- **WHEN** a `ChFX` carries `blur` 1e30, `Dstn` 1e30, and `Opct` 1e30
- **THEN** `decode_satin` returns a `Satin` with `size` 250, `distance` 30000, and `opacity` 100, and rendering it does not panic
- **AND WHEN** a `ChFX` carries `blur` 1e300 (a finite `f64` that overflows `f32`)
- **THEN** `decode_satin` returns `None` and rendering does not panic

#### Scenario: A disabled satin decodes but is inert

- **WHEN** a `ChFX` object has `present` true and `enab` false
- **THEN** `decode_satin` returns a `Satin` with `enabled` false, and compositing the layer leaves the output byte-identical to the same document without the effect

#### Scenario: Malformed or absent satin effects are a no-op

- **WHEN** a layer has no `lfx2` block, or its `lfx2` block lacks a `ChFX`, has an unknown data version, a `ChFX` with a wrong class id, a wrong-typed `Md  `, a non-`RGBC` `Clr `, or fails to parse
- **THEN** `decode_satin` returns `None` and does not panic

### Requirement: A satin composites inside the layer content

An enabled `Satin` SHALL be composited inside the layer content as a directional
interior band. On a visible non-group, non-destructive-adjustment layer with
`present` true the CPU compositor SHALL composite it **after** the layer's own
content. It SHALL build a coverage matte `M` from the layer's content alpha — a
pixel layer's alpha channel, a solid fill's payload alpha, an opaque gradient or
embedded smart-object coverage, or a pattern fill's resolved pattern alpha —
multiplied by the layer mask, and a Gaussian blur `B = blur(M, size)` of radius
`size` pixels using the crate's existing blur. With the clamped `distance`, the
finite `angle`, and screen coordinates (y down), it SHALL shift the field by
`dx = -round(distance·cos(angle))` and `dy = +round(distance·sin(angle))` and
form

```
band(x, y)  = |B(x - dx, y - dy) - B(x + dx, y + dy)|   clamped to 0..=1
field(x, y) = invert ? 1 - band : band
satin(x, y) = M(x, y) · field(x, y) · opacity/100
```

then composite `satin` tinted by `color` with the effect's own `blend_mode` and
no extra layer opacity or fill. The contour (`MpgS`), anti-alias (`AntA`) and the
global-light resource SHALL NOT change the render. `invert` false SHALL leave the
band; `invert` true SHALL use its complement. Every pixel outside the content
coverage `M` SHALL be byte-identical to the same document without the effect. A
zero-area or fully off-canvas rect, an empty matte, or `opacity` 0 SHALL be a
no-op; a zero `distance` with `invert` false SHALL be a no-op. A layer with no
effect, or with a disabled or not-present effect, SHALL composite byte-identically
to the same document without the effect. The satin SHALL be clipped to the canvas,
SHALL NOT panic, and SHALL NOT add a dependency.

#### Scenario: The satin is interior and leaves exterior pixels unchanged

- **WHEN** an opaque square layer over an opaque backdrop has an enabled and present satin with a non-zero `distance`, `size` and colour, and the composite is compared with the same document without the effect
- **THEN** every pixel outside the square's coverage is byte-identical, and at least one interior pixel differs from the un-satined content

#### Scenario: Angle and distance shape the satin band

- **WHEN** the same satin is rendered with two different `angle_deg` values, and separately with two different `distance` values at a fixed angle
- **THEN** the composites differ, showing that the band direction and width follow `angle_deg` and `distance`

#### Scenario: Size softens the satin band

- **WHEN** the same satin is rendered with `size` 0 and with a larger `size`
- **THEN** the larger-`size` transition spans at least as many interior pixels as the `size` 0 band

#### Scenario: Invert flips the satin field

- **WHEN** the same satin is rendered once with `invert` false and once with `invert` true
- **THEN** the two composites differ, with the inverted run using the complement of the band

#### Scenario: Opacity, colour and blend mode shape the satin

- **WHEN** a satin with `opacity` 50 and a red `color` is composited over an opaque white content layer, and once more with `opacity` 100 and a different supported `blend_mode`
- **THEN** lowering opacity approaches the unsatined content, and the two blend-mode composites differ

#### Scenario: The layer mask shapes the matte

- **WHEN** a satined layer's mask hides half the canvas
- **THEN** the satin is absent where the mask is zero and present where the mask is 255

#### Scenario: The satin is bounded to the content rect plus reach

- **WHEN** a small content layer with a large `size` and `distance` is composited on a much larger canvas
- **THEN** every pixel outside the content rect padded by the distance and blur reach is byte-identical to the same document without the effect

#### Scenario: A disabled or absent satin is a no-op

- **WHEN** a layer carries no `lfx2` block, or a `ChFX` with `enab` false, or a `ChFX` with `distance` 0 and `invert` false
- **THEN** the composite is byte-identical to the same document without the effect

