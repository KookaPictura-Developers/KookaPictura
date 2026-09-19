## MODIFIED Requirements

### Requirement: Committed adjustment payloads decode to typed parameters

`decode_adjustment` SHALL decode the committed PSD adjustment payloads into the
matching `pictura_adjust::Adjustment` variant: `expA` to
`Adjustment::Exposure(ExposureParams)`, `vibA` to
`Adjustment::Vibrance(VibranceParams)`, `blwh` to
`Adjustment::BlackWhite(BlackWhiteParams)`, `phfl` to
`Adjustment::PhotoFilter(PhotoFilterParams)`, `grdm` to
`Adjustment::GradientMap(GradientMapParams)`, and `SoCo` to
`Adjustment::SolidFill([u8; 4])`. The fixed structs `expA` and
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
four stored components.

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

### Requirement: Deferred adjustment payloads remain no-ops

The renderer SHALL return `None` from `decode_adjustment` for the deferred
payloads `curv`, `selc`, `clrL`, `mixr`, and a version-3 `phfl`. These payloads
SHALL remain preserved on disk and their layers SHALL leave the backdrop
unchanged.

#### Scenario: Deferred keys return None

- **WHEN** a `curv`, `selc`, `clrL`, `mixr`, or version-3 `phfl` payload is decoded
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Deferred layer does not change the composite

- **WHEN** a document contains an adjustment layer with a deferred payload over a backdrop
- **THEN** the composite equals the backdrop-only composite

## ADDED Requirements

### Requirement: Solid-color fill payloads encode and round-trip

`pictura-render` SHALL expose `encode_solid_color_fill(color: [u8; 3]) ->
AdjustmentData` that builds the standard Photoshop `SoCo` block: a version-16
`DescriptorBlock` whose `Clr ` item is a descriptor object carrying `Rd  `,
`Grn `, and `Bl  ` `doub` values on the `0..=255` scale. `decode_adjustment` on
the encoder's output SHALL equal `Adjustment::SolidFill([r, g, b, 255])`.

#### Scenario: Encoded solid fill decodes back

- **WHEN** `encode_solid_color_fill([10, 20, 30])` is passed to `decode_adjustment`
- **THEN** it returns `Adjustment::SolidFill([10, 20, 30, 255])`

#### Scenario: The encoded block is a version-16 descriptor

- **WHEN** the output of `encode_solid_color_fill` is parsed by `pictura_codec::read_descriptor`
- **THEN** it is an object whose `Clr ` item is an object with the three double components

### Requirement: Solid-color fill layers author the standard descriptor

The app SHALL author a new solid-color fill layer with the standard `SoCo`
descriptor rather than the in-house 4-byte form, while the 4-byte payload SHALL
remain readable for backward compatibility. Because the standard descriptor
carries no alpha, an authored fill layer SHALL be opaque.

#### Scenario: A new fill layer carries the standard descriptor

- **WHEN** the app adds a solid-color fill layer
- **THEN** its `SoCo` payload is a version-16 descriptor and it composites to the requested colour at full alpha

#### Scenario: An in-house 4-byte fill still decodes

- **WHEN** a layer carries a 4-byte `SoCo` payload
- **THEN** `decode_adjustment` returns `Adjustment::SolidFill` with its four stored components
