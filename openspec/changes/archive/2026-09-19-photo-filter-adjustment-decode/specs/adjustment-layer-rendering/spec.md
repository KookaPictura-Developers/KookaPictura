## MODIFIED Requirements

### Requirement: Committed adjustment payloads decode to typed parameters

`decode_adjustment` SHALL decode the committed PSD adjustment payloads into the
matching `pictura_adjust::Adjustment` variant: `expA` to
`Adjustment::Exposure(ExposureParams)`, `vibA` to
`Adjustment::Vibrance(VibranceParams)`, `blwh` to
`Adjustment::BlackWhite(BlackWhiteParams)`, and `phfl` to
`Adjustment::PhotoFilter(PhotoFilterParams)`. The fixed structs `expA` and
`phfl` SHALL be read as a `u16` version followed by fixed-width fields. The
descriptor payloads `vibA` and `blwh` SHALL be read as a 4-byte descriptor
version equal to 16 followed by a descriptor object whose keys carry the slider
values. A version-2 `phfl` payload SHALL be read as a `u16` colour space, four
`u16` colour components, a `u32` density, and a `u8` luminosity flag; the first
three components SHALL be the filter colour, each in `0..=255`, and the fourth
and the colour space SHALL be ignored.

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

#### Scenario: Photo Filter version 3 is deferred

- **WHEN** a `phfl` payload has version 3
- **THEN** `decode_adjustment` returns `None`

### Requirement: Deferred adjustment payloads remain no-ops

The renderer SHALL return `None` from `decode_adjustment` for the deferred
payloads `curv`, `selc`, `clrL`, `gdrm`, `mixr`, a version-3 `phfl`, and a real
Photoshop `SoCo` descriptor. These payloads SHALL remain preserved on disk and
their layers SHALL leave the backdrop unchanged.

#### Scenario: Deferred keys return None

- **WHEN** a `curv`, `selc`, `clrL`, `gdrm`, `mixr`, or version-3 `phfl` payload is decoded
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Real solid-color descriptor still returns None

- **WHEN** a `SoCo` payload is a Photoshop descriptor rather than the 4-byte in-house form
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Deferred layer does not change the composite

- **WHEN** a document contains an adjustment layer with a deferred payload over a backdrop
- **THEN** the composite equals the backdrop-only composite

## ADDED Requirements

### Requirement: Photo Filter payloads encode and round-trip

`pictura-render` SHALL expose `encode_photo_filter(color: [u8; 3], density:
f64, preserve_luminosity: bool) -> AdjustmentData` that builds the version-2
`phfl` block `decode_adjustment` reads. The encoder SHALL clamp `density` to
`0..=100` and clamp each colour component to `0..=255`, so its output always
decodes. `decode_adjustment` on the encoder's output SHALL equal
`Adjustment::PhotoFilter` with the input colour, the clamped density, and the
input luminosity flag.

#### Scenario: Encoded Photo Filter decodes back

- **WHEN** `encode_photo_filter([255, 180, 80], 25.0, true)` is passed to `decode_adjustment`
- **THEN** it returns `Adjustment::PhotoFilter` with `color` `[255, 180, 80]`, `density` `25.0`, and `preserve_luminosity` true

#### Scenario: Out-of-range density is clamped

- **WHEN** `encode_photo_filter` is called with a density above 100 or below 0
- **THEN** the encoded density is clamped into `0..=100` and the block still decodes

### Requirement: The app can create a Photo Filter adjustment layer

The app SHALL map the adjustment kind `photo-filter` to a `phfl` adjustment
layer carrying a warming filter at density 25 with luminosity preservation, and
the Adjustments panel menu SHALL offer a `Photo Filter` entry that dispatches
`adjustment:photo-filter`. Compositing such a layer over a non-uniform backdrop
SHALL change the result and SHALL warm it (red above blue) while keeping the
per-pixel luminance close to the backdrop when luminosity is preserved.

#### Scenario: The photo-filter kind becomes an adjustment layer

- **WHEN** the app adds an adjustment layer of kind `photo-filter`
- **THEN** the new layer carries a `phfl` block and is reported as an adjustment layer

#### Scenario: The panel menu offers Photo Filter

- **WHEN** the Adjustments panel menu is built
- **THEN** it contains a `Photo Filter` row dispatching `adjustment:photo-filter`

#### Scenario: A Photo Filter layer warms and preserves luminosity

- **WHEN** a well-formed Photo Filter adjustment layer is composited over a non-uniform backdrop
- **THEN** the result differs from the backdrop-only composite, red exceeds blue, and each pixel's luminance is within tolerance of the backdrop
