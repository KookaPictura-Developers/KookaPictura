## ADDED Requirements

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
