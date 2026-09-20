## ADDED Requirements

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
