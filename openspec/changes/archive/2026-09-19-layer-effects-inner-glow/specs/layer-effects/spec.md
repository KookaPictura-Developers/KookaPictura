## ADDED Requirements

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
