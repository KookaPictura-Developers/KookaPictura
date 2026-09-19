## MODIFIED Requirements

### Requirement: Committed adjustment payloads decode to typed parameters

`decode_adjustment` SHALL decode the committed PSD adjustment payloads into the
matching `pictura_adjust::Adjustment` variant: `expA` to
`Adjustment::Exposure(ExposureParams)`, `vibA` to
`Adjustment::Vibrance(VibranceParams)`, `blwh` to
`Adjustment::BlackWhite(BlackWhiteParams)`, `phfl` to
`Adjustment::PhotoFilter(PhotoFilterParams)`, and `grdm` to
`Adjustment::GradientMap(GradientMapParams)`. The fixed structs `expA` and
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
fourth component, and every field after the colour stops SHALL be ignored.

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

#### Scenario: Photo Filter version 3 is deferred

- **WHEN** a `phfl` payload has version 3
- **THEN** `decode_adjustment` returns `None`

### Requirement: Deferred adjustment payloads remain no-ops

The renderer SHALL return `None` from `decode_adjustment` for the deferred
payloads `curv`, `selc`, `clrL`, `mixr`, a version-3 `phfl`, and a real
Photoshop `SoCo` descriptor. These payloads SHALL remain preserved on disk and
their layers SHALL leave the backdrop unchanged.

#### Scenario: Deferred keys return None

- **WHEN** a `curv`, `selc`, `clrL`, `mixr`, or version-3 `phfl` payload is decoded
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Real solid-color descriptor still returns None

- **WHEN** a `SoCo` payload is a Photoshop descriptor rather than the 4-byte in-house form
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Deferred layer does not change the composite

- **WHEN** a document contains an adjustment layer with a deferred payload over a backdrop
- **THEN** the composite equals the backdrop-only composite

## ADDED Requirements

### Requirement: Gradient Map payloads encode and round-trip

`pictura-render` SHALL expose `encode_gradient_map(stops: &[GradientStop],
reverse: bool) -> AdjustmentData` that builds the version-1 `grdm` block
`decode_adjustment` reads. The encoder SHALL write the reverse flag, a unicode
name, the supplied colour stops with their 8-bit colours scaled to the 16-bit
storage scale, zero transparency stops, and the trailing gradient fields with
psd-tools' defaults, padded to a 4-byte boundary. `decode_adjustment` on the
encoder's output for a valid stop list SHALL equal
`Adjustment::GradientMap` with the same stops and `reverse` flag.

#### Scenario: Encoded Gradient Map decodes back

- **WHEN** `encode_gradient_map` is called with stops at location 0 colour `[0, 0, 0]` and location 4096 colour `[255, 255, 255]` and `reverse` false
- **THEN** `decode_adjustment` returns `Adjustment::GradientMap` with those two stops and `reverse` false

#### Scenario: The reverse flag round-trips

- **WHEN** `encode_gradient_map` is called with `reverse` true
- **THEN** the decoded `GradientMapParams.reverse` is true

### Requirement: The app can create a Gradient Map adjustment layer

The app SHALL map the adjustment kind `gradient-map` to a `grdm` adjustment
layer carrying a black-to-white gradient with reverse off, and the Adjustments
panel menu SHALL offer a `Gradient Map` entry that dispatches
`adjustment:gradient-map`. Compositing such a layer over a non-uniform backdrop
SHALL change the result by mapping the backdrop's luminance through the
gradient.

#### Scenario: The gradient-map kind becomes an adjustment layer

- **WHEN** the app adds an adjustment layer of kind `gradient-map`
- **THEN** the new layer carries a `grdm` block and is reported as an adjustment layer

#### Scenario: The panel menu offers Gradient Map

- **WHEN** the Adjustments panel menu is built
- **THEN** it contains a `Gradient Map` row dispatching `adjustment:gradient-map`

#### Scenario: A Gradient Map layer changes the composite

- **WHEN** a black-to-white Gradient Map adjustment layer is composited over a non-uniform backdrop
- **THEN** the result differs from the backdrop-only composite
