## ADDED Requirements

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
