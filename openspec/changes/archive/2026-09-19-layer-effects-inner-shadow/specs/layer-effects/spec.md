## ADDED Requirements

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
