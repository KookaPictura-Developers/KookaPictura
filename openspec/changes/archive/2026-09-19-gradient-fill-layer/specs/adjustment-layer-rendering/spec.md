## MODIFIED Requirements

### Requirement: Committed adjustment payloads decode to typed parameters

`decode_adjustment` SHALL decode the committed PSD adjustment payloads into the
matching `pictura_adjust::Adjustment` variant: `expA` to
`Adjustment::Exposure(ExposureParams)`, `vibA` to
`Adjustment::Vibrance(VibranceParams)`, `blwh` to
`Adjustment::BlackWhite(BlackWhiteParams)`, `phfl` to
`Adjustment::PhotoFilter(PhotoFilterParams)`, `grdm` to
`Adjustment::GradientMap(GradientMapParams)`, `SoCo` to
`Adjustment::SolidFill([u8; 4])`, and `GdFl` to
`Adjustment::GradientFill(GradientFillParams)`. The fixed structs `expA` and
`phfl` SHALL be read as a `u16` version followed by fixed-width fields. The
descriptor payloads `vibA` and `blwh` SHALL be read as a 4-byte descriptor
version equal to 16 followed by a descriptor object whose keys carry the slider
values. A version-2 `phfl` payload SHALL be read as a `u16` colour space, four
`u16` colour components, a `u32` density, and a `u8` luminosity flag; the first
three components SHALL be the filter colour, each in `0..=255`, and the fourth
and the colour space SHALL be ignored. A `grdm` payload SHALL be read as a `u16`
version (`1` or `3`, a `4`-byte method following the two flag bytes when the
version is 3), a `u8` reverse flag, a `u8` dither flag, a unicode gradient name,
and a colour-stop list; each stop SHALL be read as a `u32` location, a `u32`
midpoint, a `u16` mode, and four `u16` colour components whose first three are
reduced from the 16-bit scale to the stop's RGB colour, and the midpoint, mode,
fourth component, and every field after the colour stops SHALL be ignored. A
`SoCo` payload SHALL be read in one of two forms: a 4-byte straight-alpha RGBA
tuple, or a version-16 descriptor whose `Clr ` object carries `Rd  `, `Grn `,
and `Bl  ` `doub` values on the `0..=255` scale. The descriptor form SHALL
decode to `Adjustment::SolidFill([r, g, b, 255])`, rounding and clamping each
component to `0..=255` and forcing alpha to 255; the 4-byte form SHALL keep its
four stored components. A `GdFl` payload SHALL be read as a version-16
descriptor whose `Angl` `doub` is the angle in degrees, whose `Type` enum (typeID
`GrdT`) is the gradient kind (`Lnr ` Linear, `Rdl ` Radial, `Angl` Angle, `Rflc`
Reflected, `Dmnd` Diamond), and whose `Grad` object (class `Grdn`) carries the
gradient stops. The `Grad` object's `GrdF` enum (typeID `GrdF`) SHALL be the
custom-stops form (`CstS`); a colour-noise form (`ClNs`) SHALL NOT decode. Each
stop in the `Clrs` list SHALL be an object whose `Clr ` `RGBC` object carries
`Rd  `, `Grn `, and `Bl  ` finite `doub` values on the `0..=255` scale and whose
`Lctn` value is a `doub` or `long` in `0..=4096`. Stops SHALL be kept in stored
order. When present, the top-level `Rvrs` `bool` SHALL be the reverse flag and
the `Scl ` `doub` SHALL be the scale percent; absent, they SHALL default to false
and 100.

#### Scenario: Exposure payload decodes to ExposureParams

- **WHEN** an `expA` payload has version 1 and finite exposure, offset, and gamma values
- **THEN** `decode_adjustment` returns `Adjustment::Exposure` carrying those three values

#### Scenario: Vibrance payload decodes to VibranceParams

- **WHEN** a `vibA` descriptor contains the `vibrance` and `Strt` integer keys in range
- **THEN** `decode_adjustment` returns `Adjustment::Vibrance` with those vibrance and saturation values

#### Scenario: Black and white payload decodes to BlackWhiteParams

- **WHEN** a `blwh` descriptor contains the `Rd  `, `Yllw`, `Grn `, `Cyn `, `Bl  `, `Mgnt`, `useTint`, and `tintColor` keys
- **THEN** `decode_adjustment` returns `Adjustment::BlackWhite` with the six channel percentages, the tint flag, and the tint colour

#### Scenario: Photo Filter payload decodes to PhotoFilterParams

- **WHEN** a `phfl` payload has version 2, four colour components whose first three are each `0..=255`, and a density in `0..=100`
- **THEN** `decode_adjustment` returns `Adjustment::PhotoFilter` whose `color` is the first three components, whose `density` is the stored percent, and whose `preserve_luminosity` is the luminosity byte interpreted as a boolean

#### Scenario: Gradient Map payload decodes to GradientMapParams

- **WHEN** a `grdm` payload has version 1 or 3, a two-flag header, a unicode name, and at least two colour stops whose locations strictly increase within `0..=4096`
- **THEN** `decode_adjustment` returns `Adjustment::GradientMap` whose `stops` carry those locations and the 16-bit-reduced RGB colours and whose `reverse` is the reverse flag interpreted as a boolean

#### Scenario: Gradient Map version 3 skips its method

- **WHEN** a `grdm` payload has version 3 and a `4`-byte method before the name
- **THEN** `decode_adjustment` returns `Adjustment::GradientMap` with the same stops as the equivalent version-1 payload

#### Scenario: Solid-color fill payload decodes to SolidFill

- **WHEN** a `SoCo` payload is a 4-byte RGBA tuple, or a version-16 descriptor whose `Clr ` object carries `Rd  `/`Grn `/`Bl  ` doubles on the `0..=255` scale
- **THEN** `decode_adjustment` returns `Adjustment::SolidFill` with those components; the descriptor form forces alpha to 255

#### Scenario: Gradient fill payload decodes to GradientFill

- **WHEN** a `GdFl` descriptor has a finite `Angl`, a `Type` kind, and a `Grad` `Grdn` object whose `GrdF` is `CstS` and whose `Clrs` list holds two `RGBC` stops with strictly increasing `Lctn` in `0..=4096`
- **THEN** `decode_adjustment` returns `Adjustment::GradientFill` whose `kind` is the decoded kind, whose `angle_deg` is the `Angl` value, whose `reverse` is false and `scale` is 100 when absent, and whose `stops` carry those locations and colours

#### Scenario: Every gradient fill kind decodes

- **WHEN** the `Type` enum is `Lnr `, `Rdl `, `Angl`, `Rflc`, or `Dmnd`
- **THEN** the decoded `GradientKind` is Linear, Radial, Angle, Reflected, or Diamond respectively

#### Scenario: Malformed gradient fill descriptor is a no-op

- **WHEN** a `GdFl` payload does not parse as a descriptor object, carries a colour-noise `GrdF` (`ClNs`), lacks `Angl`/`Type`/`Grad`/`Clrs`, has fewer than two stops, has non-increasing or out-of-range stop locations, or carries a non-finite angle or colour component
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed exposure payload is a no-op

- **WHEN** an `expA` payload is truncated, has a version other than 1, carries a non-finite value, or has gamma at or below zero
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed vibrance payload is a no-op

- **WHEN** a `vibA` payload is truncated, does not parse as a descriptor object, or carries an out-of-range or wrong-typed key
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed black and white payload is a no-op

- **WHEN** a `blwh` payload is truncated, does not parse as a descriptor object, or carries an out-of-range or wrong-typed key
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed Photo Filter payload is a no-op

- **WHEN** a `phfl` payload is truncated, has a version other than 2, has a colour component above 255, or has a density above 100
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed Gradient Map payload is a no-op

- **WHEN** a `grdm` payload is truncated, has a version other than 1 or 3, has fewer than two colour stops, or has non-increasing or out-of-range stop locations
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Malformed solid-color descriptor is a no-op

- **WHEN** a `SoCo` payload is neither a 4-byte RGBA tuple nor a version-16 descriptor object whose `Clr ` object carries `Rd  `, `Grn `, and `Bl  ` finite doubles
- **THEN** `decode_adjustment` returns `None` and does not panic

#### Scenario: Photo Filter version 3 is deferred

- **WHEN** a `phfl` payload has version 3
- **THEN** `decode_adjustment` returns `None`

## ADDED Requirements

### Requirement: Gradient fill layers render generatively

The renderer SHALL composite a layer whose adjustment decodes to
`Adjustment::GradientFill` by generating an opaque colour for every pixel inside
the layer's rectangle (clamped to the canvas) and blending it through the normal
layer path, so the layer's mask, opacity, fill, and blend mode apply and pixels
outside the rectangle are untouched. Generation SHALL follow the psd-tools
`draw_gradient_fill` geometry for all five kinds — Linear, Radial, Angle,
Reflected, and Diamond — using the params' `angle_deg`, `scale`, and `reverse`
and linearly interpolating the `stops` (clamped outside the stop range). The
generated output alpha SHALL be 255.

#### Scenario: A gradient fill layer changes the composite

- **WHEN** a black-to-white Linear gradient fill layer is composited over a backdrop
- **THEN** the result differs from the backdrop-only composite and the in-rect pixels follow the black-to-white ramp

#### Scenario: Every geometry kind renders

- **WHEN** a gradient fill layer of kind Linear, Radial, Angle, Reflected, or Diamond is composited
- **THEN** each produces its own non-empty in-rect gradient rather than a no-op

#### Scenario: Reverse flips the ramp

- **WHEN** the same black-to-white gradient fill is composited with `reverse` true
- **THEN** the direction of the ramp is inverted relative to the forward case

#### Scenario: Masked-out layer stays a no-op

- **WHEN** a gradient fill layer's mask hides the whole canvas
- **THEN** the composite equals the backdrop-only composite

### Requirement: Gradient fill payloads encode and round-trip

`pictura-render` SHALL expose `encode_gradient_fill(kind: GradientKind, stops:
&[GradientStop], angle_deg: f32) -> AdjustmentData` that builds the standard
Photoshop `GdFl` block: a version-16 `DescriptorBlock` whose `Angl` is the angle,
whose `Type` enum (typeID `GrdT`) is the kind, and whose `Grad` object (class
`Grdn`) carries `GrdF` custom stops (`CstS`), an `Intr` interpolation, and a
`Clrs` list of `RGBC` stops with `Lctn` on the `0..=4096` scale. The encoder
SHALL write only custom-stop gradients and SHALL NOT write `Rvrs`/`Scl `.
`decode_adjustment` on the encoder's output SHALL equal
`Adjustment::GradientFill` with the input kind, stops, and angle (reverse false,
scale 100).

#### Scenario: Encoded gradient fill decodes back

- **WHEN** `encode_gradient_fill(GradientKind::Linear, &[black@0, white@4096], 0.0)` is passed to `decode_adjustment`
- **THEN** it returns `Adjustment::GradientFill` with those stops, kind Linear, `angle_deg` 0.0, `reverse` false, and `scale` 100

#### Scenario: The encoded block is a version-16 descriptor

- **WHEN** the output of `encode_gradient_fill` is parsed by `pictura_codec::read_descriptor`
- **THEN** it is an object whose `Type` enum is the kind and whose `Grad` object carries a `Clrs` list of `RGBC` stops

### Requirement: The app can create a gradient fill layer

The app SHALL create a gradient fill layer for the default kind `gradient-fill`
(black-to-white Linear at angle 0) through a document-sized `GdFl` layer, and
`Layer > New Fill Layer > Gradient…` and the Layers-panel fill menu SHALL both
offer an enabled `Gradient…` entry that creates one. Compositing such a layer
over a backdrop SHALL change the result.

#### Scenario: The gradient-fill kind becomes a fill layer

- **WHEN** the app adds a gradient fill layer
- **THEN** the new layer carries a `GdFl` block, is a fill-content layer, and composites to a black-to-white ramp

#### Scenario: The menus offer Gradient

- **WHEN** the Layer menu and the Layers-panel fill menu are built
- **THEN** each offers an enabled `Gradient…` entry that creates a gradient fill layer
